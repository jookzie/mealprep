//! What crosses the bridge from the Kotlin plugin, and its conversion into the domain.
//!
//! Instants travel as epoch milliseconds with the offset they were recorded in, in seconds.
//! That is unambiguous in both languages, where a formatted timestamp would depend on how
//! each side prints and parses seconds and zones.

// Only the Android source constructs and reads these; the conversion is tested everywhere.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use mealprep_core::{
    Error, Result,
    domain::{
        self, ActivityDay, HealthAvailability, HealthImport, Reading, SleepSession, SleepStage,
        SleepStageKind,
    },
};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

#[derive(Serialize)]
pub(crate) struct ReadArgs {
    pub(crate) from: String,
    pub(crate) to: String,
}

#[derive(Deserialize)]
pub(crate) struct Status {
    availability: String,
    pub(crate) connected: bool,
}

impl Status {
    pub(crate) fn availability(&self) -> HealthAvailability {
        match self.availability.as_str() {
            "available" => HealthAvailability::Available,
            "update-required" => HealthAvailability::UpdateRequired,
            _ => HealthAvailability::Unsupported,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Connected {
    pub(crate) connected: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadResponse {
    #[serde(default)]
    sleep: Vec<Session>,
    #[serde(default)]
    resting_heart_rate: Vec<Sample>,
    #[serde(default)]
    heart_rate_variability: Vec<Sample>,
    #[serde(default)]
    respiratory_rate: Vec<Sample>,
    #[serde(default)]
    activity: Vec<Activity>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    start_ms: i64,
    start_offset: i32,
    end_ms: i64,
    end_offset: i32,
    #[serde(default)]
    stages: Vec<Stage>,
}

/// A stage carries no offset of its own; it takes the session's start offset.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Stage {
    start_ms: i64,
    end_ms: i64,
    stage: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sample {
    at_ms: i64,
    offset: i32,
    value: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Activity {
    date: String,
    active_kcal: Option<f64>,
    total_kcal: Option<f64>,
    exercise_minutes: Option<f64>,
}

impl ReadResponse {
    pub(crate) fn into_import(self) -> Result<HealthImport> {
        let readings = |samples: Vec<Sample>| -> Result<Vec<Reading>> {
            samples
                .into_iter()
                .map(|sample| {
                    Ok(Reading {
                        at: instant(sample.at_ms, sample.offset)?,
                        value: sample.value,
                    })
                })
                .collect()
        };

        Ok(HealthImport {
            sleep: self
                .sleep
                .into_iter()
                .map(Session::into_domain)
                .collect::<Result<_>>()?,
            resting_heart_rate: readings(self.resting_heart_rate)?,
            heart_rate_variability: readings(self.heart_rate_variability)?,
            respiratory_rate: readings(self.respiratory_rate)?,
            activity: self
                .activity
                .into_iter()
                .map(Activity::into_domain)
                .collect::<Result<_>>()?,
        })
    }
}

impl Session {
    fn into_domain(self) -> Result<SleepSession> {
        let stages = self
            .stages
            .into_iter()
            .map(|stage| {
                Ok(SleepStage {
                    start: instant(stage.start_ms, self.start_offset)?,
                    end: instant(stage.end_ms, self.start_offset)?,
                    kind: stage_kind(stage.stage),
                })
            })
            .collect::<Result<_>>()?;
        Ok(SleepSession {
            start: instant(self.start_ms, self.start_offset)?,
            end: instant(self.end_ms, self.end_offset)?,
            stages,
        })
    }
}

impl Activity {
    fn into_domain(self) -> Result<ActivityDay> {
        Ok(ActivityDay {
            date: domain::parse_iso_date(&self.date)?,
            active_kcal: self.active_kcal,
            total_kcal: self.total_kcal,
            exercise_minutes: self.exercise_minutes,
        })
    }
}

/// Health Connect's `SleepSessionRecord.STAGE_TYPE_*` constants.
fn stage_kind(stage: i32) -> SleepStageKind {
    match stage {
        1 | 7 => SleepStageKind::Awake,
        2 => SleepStageKind::Sleeping,
        3 => SleepStageKind::OutOfBed,
        4 => SleepStageKind::Light,
        5 => SleepStageKind::Deep,
        6 => SleepStageKind::Rem,
        _ => SleepStageKind::Unknown,
    }
}

fn instant(epoch_ms: i64, offset_seconds: i32) -> Result<OffsetDateTime> {
    let malformed = |reason: String| Error::HealthUnavailable { reason };
    let utc = OffsetDateTime::from_unix_timestamp_nanos(i128::from(epoch_ms) * 1_000_000)
        .map_err(|source| malformed(format!("cannot read instant {epoch_ms}: {source}")))?;
    let offset = UtcOffset::from_whole_seconds(offset_seconds)
        .map_err(|source| malformed(format!("cannot read offset {offset_seconds}: {source}")))?;
    Ok(utc.to_offset(offset))
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;

    #[test]
    fn reads_what_the_kotlin_side_sends() {
        let json = r#"{
            "sleep": [{
                "startMs": 1788300000000, "startOffset": 7200,
                "endMs": 1788328800000, "endOffset": 7200,
                "stages": [{ "startMs": 1788300000000, "endMs": 1788303600000, "stage": 5 }]
            }],
            "heartRateVariability": [{ "atMs": 1788320000000, "offset": 7200, "value": 61.5 }],
            "activity": [{ "date": "2026-09-02", "activeKcal": 420.0, "totalKcal": null }]
        }"#;

        let response: ReadResponse = serde_json::from_str(json).unwrap();
        let import = response.into_import().unwrap();

        let session = &import.sleep[0];
        assert_eq!(session.start, datetime!(2026-09-02 00:00 +02:00));
        assert_eq!(session.end, datetime!(2026-09-02 08:00 +02:00));
        assert_eq!(session.stages[0].kind, SleepStageKind::Deep);
        assert_eq!(import.heart_rate_variability[0].value, 61.5);
        assert!(import.resting_heart_rate.is_empty());
        assert_eq!(import.activity[0].date, date!(2026 - 09 - 02));
        assert_eq!(import.activity[0].total_kcal, None);
        assert_eq!(import.activity[0].exercise_minutes, None);
    }

    #[test]
    fn an_unknown_availability_is_unsupported() {
        let status: Status =
            serde_json::from_str(r#"{ "availability": "later", "connected": false }"#).unwrap();

        assert_eq!(status.availability(), HealthAvailability::Unsupported);
    }
}
