use serde::{Deserialize, Serialize};
use time::Date;

use super::NightSleep;

/// One night, named by the morning it ended on.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepNight {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    #[serde(flatten)]
    pub night: NightSleep,
}

/// Sleep duration and regularity, derived on read.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepSummary {
    /// Every imported night, in date order.
    pub nights: Vec<SleepNight>,
    /// Mean time asleep over the last week, absent below three nights.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub average_minutes: Option<f64>,
    /// Median bedtime over the last four weeks, in minutes from midnight (negative before).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typical_bed_minute: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typical_wake_minute: Option<f64>,
    /// Standard deviation of bedtime over the last four weeks, in minutes. Absent below two
    /// weeks of nights, which is too few to say anything about regularity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bedtime_spread_minutes: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake_spread_minutes: Option<f64>,
}
