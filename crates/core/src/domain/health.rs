//! What Health Connect supplies, and the one row per day it is summarised into.
//!
//! The raw records only exist on the way in: [`HealthImport`] is what a source reads, and
//! [`HealthDay`] is what is stored and what every insight reads, so the insights work offline
//! and do not depend on what the source still holds.

use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

/// Everything a source read for a range of days.
///
/// Times carry the offset the record was taken in, so a night in another time zone still
/// ends on the morning it ended on there.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HealthImport {
    pub sleep: Vec<SleepSession>,
    pub resting_heart_rate: Vec<Reading>,
    pub heart_rate_variability: Vec<Reading>,
    pub respiratory_rate: Vec<Reading>,
    /// Already summed per day by the source, which de-duplicates overlapping apps by the
    /// priority the user set; summing raw records here would count a phone and a strap twice.
    pub activity: Vec<ActivityDay>,
}

/// One sleep session as the source recorded it.
#[derive(Clone, Debug, PartialEq)]
pub struct SleepSession {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    /// Empty when the source only knows the session's bounds.
    pub stages: Vec<SleepStage>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SleepStage {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    pub kind: SleepStageKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SleepStageKind {
    Awake,
    OutOfBed,
    /// Asleep, with no stage given.
    Sleeping,
    Light,
    Deep,
    Rem,
    Unknown,
}

/// A single timed value: a resting heart rate, an HRV or a respiratory rate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Reading {
    pub at: OffsetDateTime,
    pub value: f64,
}

/// Activity the source summed for one local calendar day.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ActivityDay {
    pub date: Date,
    pub active_kcal: Option<f64>,
    pub total_kcal: Option<f64>,
    pub exercise_minutes: Option<f64>,
}

/// The night that ended on a [`HealthDay`]'s morning.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NightSleep {
    /// Minutes from the morning's midnight, in the session's own offset. Negative for a
    /// bedtime on the evening before, which is the usual case: 23:30 is -30.
    pub bed_minute: i32,
    pub wake_minute: i32,
    pub asleep_minutes: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deep_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rem_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awake_minutes: Option<f64>,
}

/// One morning's summary: the night that ended on it, the readings taken for it, and the
/// day's activity. Every figure is optional because every source is.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthDay {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sleep: Option<NightSleep>,
    /// Beats per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resting_heart_rate: Option<f64>,
    /// RMSSD in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hrv_ms: Option<f64>,
    /// Breaths per minute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub respiratory_rate: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_kcal: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_kcal: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exercise_minutes: Option<f64>,
}

impl HealthDay {
    pub fn empty(date: Date) -> Self {
        Self {
            date,
            sleep: None,
            resting_heart_rate: None,
            hrv_ms: None,
            respiratory_rate: None,
            active_kcal: None,
            total_kcal: None,
            exercise_minutes: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::empty(self.date)
    }
}

/// Whether the device can supply health data at all.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HealthAvailability {
    Available,
    /// Health Connect exists but must be installed or updated from the Play Store first.
    UpdateRequired,
    /// No Health Connect on this device or platform: a desktop build, or Android before 9.
    Unsupported,
}

/// What the health card needs to decide what to show.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub availability: HealthAvailability,
    /// Whether every permission the import asks for has been granted.
    pub connected: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub synced_at: Option<OffsetDateTime>,
}

/// When the last import ran, and the last day it covered.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HealthSync {
    pub synced_at: OffsetDateTime,
    pub through: Date,
}
