use mealprep_core::domain::{self, CalendarDay};
use tauri::State;
use uuid::Uuid;

use crate::{result::Result, setup::AppState};

/// Lists the assigned days between the given `YYYY-MM-DD` dates, both inclusive.
#[tauri::command]
pub async fn list_calendar_days(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<CalendarDay>> {
    let from = domain::parse_iso_date(&from)?;
    let to = domain::parse_iso_date(&to)?;
    let days = state.list_calendar_days(from, to).await?;
    Ok(days)
}

#[tauri::command]
pub async fn assign_calendar_day(
    state: State<'_, AppState>,
    date: String,
    day_plan_id: Uuid,
) -> Result<CalendarDay> {
    let date = domain::parse_iso_date(&date)?;
    let day = state.assign_calendar_day(date, day_plan_id).await?;
    Ok(day)
}

#[tauri::command]
pub async fn unassign_calendar_day(state: State<'_, AppState>, date: String) -> Result<()> {
    let date = domain::parse_iso_date(&date)?;
    state.unassign_calendar_day(date).await?;
    Ok(())
}
