use serde::{Deserialize, Serialize};
use time::Date;

/// What the plan and the weight trend together say about energy expenditure.
///
/// Every figure is optional and the counts travel with them: the intake is what was
/// planned, not what was eaten, and a window with few planned days or no weigh-ins at its
/// ends cannot carry an estimate.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnergyBalance {
    #[serde(with = "super::format::iso_date")]
    pub from: Date,
    #[serde(with = "super::format::iso_date")]
    pub to: Date,
    pub window_days: u32,
    /// Days in the window with a plan that still exists.
    pub planned_days: u32,
    /// Mean planned energy over the planned days, in kcal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_kcal: Option<f64>,
    /// How far the weight trend moved across the window, in kilograms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trend_change_kg: Option<f64>,
    /// Energy stored per day (negative when released), from the trend's change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stored_kcal: Option<f64>,
    /// Planned intake minus what was stored: the expenditure the trend implies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expenditure_kcal: Option<f64>,
    /// Days in the window the wearable reported total energy burned for.
    pub measured_days: u32,
    /// The wearable's own mean daily figure, as a comparison rather than as truth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_kcal: Option<f64>,
}
