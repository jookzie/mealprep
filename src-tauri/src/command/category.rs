use mealprep_core::domain::{Category, CategoryDraft, CategoryScope};
use tauri::State;
use uuid::Uuid;

use crate::{result::Result, setup::AppState};

#[tauri::command]
pub async fn list_categories(
    state: State<'_, AppState>,
    scope: CategoryScope,
) -> Result<Vec<Category>> {
    let categories = state.list_categories(scope).await?;
    Ok(categories)
}

#[tauri::command]
pub async fn create_category(state: State<'_, AppState>, draft: CategoryDraft) -> Result<Category> {
    let category = state.create_category(draft).await?;
    Ok(category)
}

#[tauri::command]
pub async fn rename_category(
    state: State<'_, AppState>,
    category_id: Uuid,
    name: String,
) -> Result<Category> {
    let category = state.rename_category(category_id, &name).await?;
    Ok(category)
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, category_id: Uuid) -> Result<()> {
    state.delete_category(category_id).await?;
    Ok(())
}
