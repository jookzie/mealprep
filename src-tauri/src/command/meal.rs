use mealprep_core::domain::{Derived, Meal, MealDraft};
use tauri::State;
use uuid::Uuid;

use crate::{result::Result, setup::AppState};

#[tauri::command]
pub async fn list_meals(state: State<'_, AppState>) -> Result<Vec<Derived<Meal>>> {
    let meals = state.list_meals().await?;
    Ok(meals)
}

#[tauri::command]
pub async fn get_meal(state: State<'_, AppState>, meal_id: Uuid) -> Result<Derived<Meal>> {
    let meal = state.get_meal(meal_id).await?;
    Ok(meal)
}

#[tauri::command]
pub async fn create_meal(state: State<'_, AppState>, draft: MealDraft) -> Result<Derived<Meal>> {
    let meal = state.create_meal(draft).await?;
    Ok(meal)
}

#[tauri::command]
pub async fn update_meal(
    state: State<'_, AppState>,
    meal_id: Uuid,
    draft: MealDraft,
) -> Result<Derived<Meal>> {
    let meal = state.update_meal(meal_id, draft).await?;
    Ok(meal)
}

#[tauri::command]
pub async fn delete_meal(state: State<'_, AppState>, meal_id: Uuid) -> Result<()> {
    state.delete_meal(meal_id).await?;
    Ok(())
}
