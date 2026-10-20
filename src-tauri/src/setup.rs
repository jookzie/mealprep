use std::fs;

use mealprep_anthropic::Anthropic;
use mealprep_core::service::Mealprep;
use mealprep_openfoodfacts::OpenFoodFacts;
use mealprep_sqlite::SqliteStore;
use tauri::{AppHandle, Manager, Runtime};

use crate::error::SetupError;

/// The business rules as the application runs them: SQLite on the device, Open Food Facts
/// as the catalogue, Claude as the assistant.
///
/// Swapping in another database means another store implementing the `mealprep_core::store`
/// traits and changing this alias; no command changes. The same holds for the assistant: a
/// model that runs on the device would be another crate implementing `store::Assistant`.
pub type AppState = Mealprep<SqliteStore, OpenFoodFacts, Anthropic>;

const DATABASE_FILE: &str = "mealprep.db";

/// Opens the database in the application's data directory, creating both when missing.
pub async fn open(app: &AppHandle<impl Runtime>) -> Result<AppState, SetupError> {
    let directory =
        app.path()
            .app_data_dir()
            .map_err(|source| SetupError::LocateDataDirectory {
                reason: source.to_string(),
            })?;
    fs::create_dir_all(&directory).map_err(|source| SetupError::CreateDataDirectory {
        path: directory.clone(),
        source,
    })?;

    let store = SqliteStore::open(&directory.join(DATABASE_FILE)).await?;
    let catalogue = OpenFoodFacts::new()?;
    // The assistant holds no key: the rules read the user's own out of the store and hand
    // it over per call, so there is no credential in the application's state.
    let assistant = Anthropic::new()?;
    Ok(Mealprep::new(store, catalogue, assistant))
}
