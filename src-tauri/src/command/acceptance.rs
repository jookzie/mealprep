//! The acceptance environment's commands. Neither is registered in a production build.

use mealprep_core::Error as CoreError;
use tauri::AppHandle;

use crate::{acceptance, error::Error, result::Result, setup};

/// Answers `true` wherever it can be called at all, which is the acceptance build only.
///
/// One frontend bundle serves both environments, so it cannot be compiled against a
/// feature flag. It asks for this command instead and reads a rejected invocation — the
/// command is not registered — as "this is production".
#[tauri::command]
pub async fn acceptance_build() -> bool {
    true
}

/// Puts the demo dataset back and closes the application, which has to be reopened to
/// read it.
///
/// Closing is part of the operation rather than an inconvenience after it: the store is
/// open on the file being replaced and holds it by descriptor, so nothing short of a
/// restart reads the new one.
#[tauri::command]
pub async fn reset_demo_data(app: AppHandle) -> Result<()> {
    let path = setup::database_path(&app).map_err(internal)?;
    acceptance::restore(&path).map_err(internal)?;
    app.exit(0);
    Ok(())
}

fn internal(reason: impl std::fmt::Display) -> Error {
    CoreError::Internal {
        reason: reason.to_string(),
    }
    .into()
}
