use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

/// One weigh-in. A date holds at most one, because a day has one number worth keeping.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightEntry {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    pub kilograms: f64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// One day of the series the graph draws.
///
/// The trend has a value on every day between the first and last weigh-in, including the
/// days that were skipped; `kilograms` is what the scale actually said, and is absent on
/// those days. Drawing the two together is the point of the graph: a measurement below the
/// line is pulling the trend down, which a lone number cannot show.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightPoint {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kilograms: Option<f64>,
    pub trend: f64,
}

/// The weigh-ins as a daily series, with what the trend implies.
///
/// Derived on read and never stored, the way macros and cost are.
#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightSeries {
    /// One entry per day from the first weigh-in to the last, in date order.
    pub points: Vec<WeightPoint>,
    /// Where the trend stands today, which is the figure to read rather than the last
    /// measurement. Absent when nothing has been logged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trend: Option<f64>,
    /// Kilograms per week, from the slope of the trend. Absent until the series is long
    /// enough for the figure to mean anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_per_week: Option<f64>,
}
