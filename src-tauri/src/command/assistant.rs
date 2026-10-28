use mealprep_core::domain::{AssistantKeyStatus, AssistantRecord, ChatTurn, Preview, Proposal};
use tauri::State;

use crate::{result::Result, setup::AppState};

/// Asks the assistant what the request means, in the conversation so far, and returns the
/// preview, writing nothing.
#[tauri::command]
pub async fn propose_assistant_edit(
    state: State<'_, AppState>,
    request: String,
    history: Vec<ChatTurn>,
) -> Result<Preview> {
    let preview = state.propose_assistant_edit(request, history).await?;
    Ok(preview)
}

/// Applies the operations the user confirmed, re-checked from scratch.
#[tauri::command]
pub async fn apply_assistant_edit(
    state: State<'_, AppState>,
    proposal: Proposal,
) -> Result<AssistantRecord> {
    let record = state.apply_assistant_edit(proposal).await?;
    Ok(record)
}

#[tauri::command]
pub async fn list_assistant_requests(state: State<'_, AppState>) -> Result<Vec<AssistantRecord>> {
    let records = state.list_assistant_requests().await?;
    Ok(records)
}

/// Reports whether the assistant has a key. The key itself is never returned in full.
#[tauri::command]
pub async fn assistant_key_status(state: State<'_, AppState>) -> Result<AssistantKeyStatus> {
    let status = state.assistant_key_status().await?;
    Ok(status)
}

#[tauri::command]
pub async fn set_assistant_key(
    state: State<'_, AppState>,
    key: String,
) -> Result<AssistantKeyStatus> {
    let status = state.set_assistant_key(key).await?;
    Ok(status)
}

#[tauri::command]
pub async fn clear_assistant_key(state: State<'_, AppState>) -> Result<()> {
    state.clear_assistant_key().await?;
    Ok(())
}
