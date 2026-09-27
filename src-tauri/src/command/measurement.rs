use mealprep_core::domain::{self, Measurement, MeasurementKind};
use tauri::State;

use crate::{result::Result, setup::AppState};

/// Lists every body measurement of every kind, in date order.
#[tauri::command]
pub async fn list_measurements(state: State<'_, AppState>) -> Result<Vec<Measurement>> {
    let measurements = state.list_measurements().await?;
    Ok(measurements)
}

/// Records a measurement for the given `YYYY-MM-DD` date, replacing that kind's value there.
#[tauri::command]
pub async fn set_measurement(
    state: State<'_, AppState>,
    date: String,
    kind: MeasurementKind,
    value: f64,
) -> Result<Measurement> {
    let date = domain::parse_iso_date(&date)?;
    let measurement = state.set_measurement(date, kind, value).await?;
    Ok(measurement)
}

#[tauri::command]
pub async fn delete_measurement(
    state: State<'_, AppState>,
    date: String,
    kind: MeasurementKind,
) -> Result<()> {
    let date = domain::parse_iso_date(&date)?;
    state.delete_measurement(date, kind).await?;
    Ok(())
}
