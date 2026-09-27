use mealprep_core::domain::{DayPlanDraft, ResolvedDayPlan};
use tauri::State;
use uuid::Uuid;

use crate::{result::Result, setup::AppState};

#[tauri::command]
pub async fn list_day_plans(state: State<'_, AppState>) -> Result<Vec<ResolvedDayPlan>> {
    let plans = state.list_day_plans().await?;
    Ok(plans)
}

#[tauri::command]
pub async fn get_day_plan(
    state: State<'_, AppState>,
    day_plan_id: Uuid,
) -> Result<ResolvedDayPlan> {
    let plan = state.get_day_plan(day_plan_id).await?;
    Ok(plan)
}

#[tauri::command]
pub async fn create_day_plan(
    state: State<'_, AppState>,
    draft: DayPlanDraft,
) -> Result<ResolvedDayPlan> {
    let plan = state.create_day_plan(draft).await?;
    Ok(plan)
}

#[tauri::command]
pub async fn update_day_plan(
    state: State<'_, AppState>,
    day_plan_id: Uuid,
    draft: DayPlanDraft,
) -> Result<ResolvedDayPlan> {
    let plan = state.update_day_plan(day_plan_id, draft).await?;
    Ok(plan)
}

#[tauri::command]
pub async fn delete_day_plan(state: State<'_, AppState>, day_plan_id: Uuid) -> Result<()> {
    state.delete_day_plan(day_plan_id).await?;
    Ok(())
}
