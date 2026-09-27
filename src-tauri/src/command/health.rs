//! Health Connect and what is derived from the days imported from it.
//!
//! Every command takes the frontend's `today`: the device's local date is what the user
//! means by today, and the webview already knows it.

use mealprep_core::domain::{self, EnergyBalance, HealthStatus, Recovery, SleepSummary};
use mealprep_health_connect::HealthConnectExt;
use tauri::{AppHandle, State};
use time::OffsetDateTime;

use crate::{result::Result, setup::AppState};

/// Whether Health Connect can be used, whether access was granted, and when it was last read.
#[tauri::command]
pub async fn health_status(app: AppHandle, state: State<'_, AppState>) -> Result<HealthStatus> {
    let source = app.health_connect();
    let availability = source.availability().await?;
    let connected = source.connected().await?;
    let synced_at = state.last_health_sync().await?.map(|sync| sync.synced_at);
    Ok(HealthStatus {
        availability,
        connected,
        synced_at,
    })
}

/// Shows Health Connect's permission screen; answers whether access was granted.
#[tauri::command]
pub async fn connect_health(app: AppHandle) -> Result<bool> {
    let connected = app.health_connect().connect().await?;
    Ok(connected)
}

/// Opens the Play Store on Health Connect.
#[tauri::command]
pub async fn install_health_connect(app: AppHandle) -> Result<()> {
    app.health_connect().install().await?;
    Ok(())
}

/// Reads Health Connect from where the last import left off and stores the result.
///
/// The one command that joins two calls: the rules decide the range and summarise, and only
/// the plugin can read, so the reading has to happen between them.
#[tauri::command]
pub async fn sync_health(app: AppHandle, state: State<'_, AppState>, today: String) -> Result<()> {
    let today = domain::parse_iso_date(&today)?;
    let from = state.health_import_start(today).await?;
    let import = app.health_connect().read(from, today).await?;
    state
        .import_health(from, today, import, OffsetDateTime::now_utc())
        .await?;
    Ok(())
}

/// HRV and resting heart rate against the user's normal range, and what went with better or
/// worse mornings.
#[tauri::command]
pub async fn recovery(state: State<'_, AppState>, today: String) -> Result<Recovery> {
    let today = domain::parse_iso_date(&today)?;
    let recovery = state.recovery(today).await?;
    Ok(recovery)
}

#[tauri::command]
pub async fn sleep_summary(state: State<'_, AppState>, today: String) -> Result<SleepSummary> {
    let today = domain::parse_iso_date(&today)?;
    let summary = state.sleep_summary(today).await?;
    Ok(summary)
}

/// What the plan and the weight trend say about expenditure over the last three weeks.
#[tauri::command]
pub async fn energy_balance(state: State<'_, AppState>, today: String) -> Result<EnergyBalance> {
    let today = domain::parse_iso_date(&today)?;
    let balance = state.energy_balance(today).await?;
    Ok(balance)
}
