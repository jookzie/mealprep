use mealprep_core::domain::{self, WeightEntry, WeightSeries};
use tauri::State;

use crate::{result::Result, setup::AppState};

/// Lists every weigh-in, most recent first.
#[tauri::command]
pub async fn list_weight_entries(state: State<'_, AppState>) -> Result<Vec<WeightEntry>> {
    let entries = state.list_weight_entries().await?;
    Ok(entries)
}

/// The weigh-ins as a daily series, with the smoothed trend and its rate of change.
#[tauri::command]
pub async fn weight_series(state: State<'_, AppState>) -> Result<WeightSeries> {
    let series = state.weight_series().await?;
    Ok(series)
}

/// Records the weight for the given `YYYY-MM-DD` date, replacing whatever was there.
#[tauri::command]
pub async fn set_weight_entry(
    state: State<'_, AppState>,
    date: String,
    kilograms: f64,
) -> Result<WeightEntry> {
    let date = domain::parse_iso_date(&date)?;
    let entry = state.set_weight_entry(date, kilograms).await?;
    Ok(entry)
}

#[tauri::command]
pub async fn delete_weight_entry(state: State<'_, AppState>, date: String) -> Result<()> {
    let date = domain::parse_iso_date(&date)?;
    state.delete_weight_entry(date).await?;
    Ok(())
}
