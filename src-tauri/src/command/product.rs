use mealprep_core::domain::{CatalogueEntry, Product, ProductDraft};
use tauri::State;
use uuid::Uuid;

use crate::{result::Result, setup::AppState};

#[tauri::command]
pub async fn list_products(state: State<'_, AppState>) -> Result<Vec<Product>> {
    let products = state.list_products().await?;
    Ok(products)
}

#[tauri::command]
pub async fn get_product(state: State<'_, AppState>, product_id: Uuid) -> Result<Product> {
    let product = state.get_product(product_id).await?;
    Ok(product)
}

#[tauri::command]
pub async fn create_product(state: State<'_, AppState>, draft: ProductDraft) -> Result<Product> {
    let product = state.create_product(draft).await?;
    Ok(product)
}

#[tauri::command]
pub async fn update_product(
    state: State<'_, AppState>,
    product_id: Uuid,
    draft: ProductDraft,
) -> Result<Product> {
    let product = state.update_product(product_id, draft).await?;
    Ok(product)
}

#[tauri::command]
pub async fn delete_product(state: State<'_, AppState>, product_id: Uuid) -> Result<()> {
    state.delete_product(product_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn search_catalogue(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<CatalogueEntry>> {
    let entries = state.search_catalogue(&query).await?;
    Ok(entries)
}

#[tauri::command]
pub async fn get_catalogue_entry(
    state: State<'_, AppState>,
    code: String,
) -> Result<CatalogueEntry> {
    let entry = state.get_catalogue_entry(&code).await?;
    Ok(entry)
}

#[tauri::command]
pub async fn import_product(
    state: State<'_, AppState>,
    code: String,
    draft: ProductDraft,
) -> Result<Product> {
    let product = state.import_product(&code, draft).await?;
    Ok(product)
}
