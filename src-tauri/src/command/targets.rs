use mealprep_core::domain::{Macros, Targets};
use tauri::State;

use crate::{result::Result, setup::AppState};

/// Fetches the daily targets, answering `null` when the user has never set any.
#[tauri::command]
pub async fn get_targets(state: State<'_, AppState>) -> Result<Option<Targets>> {
    let targets = state.get_targets().await?;
    Ok(targets)
}

#[tauri::command]
pub async fn set_targets(state: State<'_, AppState>, macros: Macros) -> Result<Targets> {
    let targets = state.set_targets(macros).await?;
    Ok(targets)
}
