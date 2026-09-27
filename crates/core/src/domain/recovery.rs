use serde::{Deserialize, Serialize};
use time::Date;

/// An overnight figure recovery is read from.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Marker {
    /// RMSSD in milliseconds; higher is better recovered.
    Hrv,
    /// Beats per minute; lower is better recovered.
    RestingHeartRate,
}

/// The user's own normal range for a marker: the mean of the last two months, plus or
/// minus half a standard deviation.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Band {
    pub low: f64,
    pub high: f64,
}

/// Where the 7-day average sits against the normal range.
///
/// Neutral words on purpose: above the range is good for HRV and bad for resting heart
/// rate, and the marker says which.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BaselineStatus {
    Below,
    Within,
    Above,
}

/// One morning of a marker's series.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerPoint {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    /// What was read that morning, absent when nothing was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// The 7-day average ending that morning, absent when the week has too few readings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub average: Option<f64>,
    /// The normal range as of that morning, absent until there are enough readings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band: Option<Band>,
}

/// A marker as a daily series with its baseline, derived on read.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerSeries {
    pub marker: Marker,
    /// One per morning from the first reading to today, in date order.
    pub points: Vec<MarkerPoint>,
    /// The 7-day average as of today, which is the figure to read rather than last night's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub average: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub band: Option<Band>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<BaselineStatus>,
}

/// Something done or planned that may go with better or worse recovery the next morning.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Behaviour {
    /// The night before the morning was seven hours or more.
    SleptSevenHours,
    /// Bedtime was earlier than the median of the window.
    EarlierBedtime,
    /// A workout was recorded the day before.
    TrainedDayBefore,
    /// The plan for the day before was over the energy target.
    EnergyOverTargetDayBefore,
    /// The plan for the day before reached the protein target.
    ProteinAtTargetDayBefore,
}

/// How a marker compared on mornings with and without a behaviour.
///
/// An association over the window, never a cause; the counts travel with it so no view can
/// show the difference without how little it may rest on.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Impact {
    pub behaviour: Behaviour,
    pub with_days: u32,
    pub without_days: u32,
    pub with_mean: f64,
    pub without_mean: f64,
    /// `with_mean` against `without_mean`, in percent.
    pub change_percent: f64,
    /// Whether the difference is at least half a standard deviation of the marker. Below
    /// that it is within ordinary day-to-day variation and reads as no clear difference.
    pub clear: bool,
}

/// Everything the recovery screen shows.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recovery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hrv: Option<MarkerSeries>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resting_heart_rate: Option<MarkerSeries>,
    /// The marker the impacts are measured on: HRV when there is enough of it, otherwise
    /// resting heart rate, because some sources do not write HRV at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_marker: Option<Marker>,
    /// Only the behaviours with enough mornings on both sides.
    pub impacts: Vec<Impact>,
}
