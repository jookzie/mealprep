use std::{fs, path::PathBuf};

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

/// Where the database lives, creating the application's data directory when missing.
pub fn database_path(app: &AppHandle<impl Runtime>) -> Result<PathBuf, SetupError> {
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
    Ok(directory.join(DATABASE_FILE))
}

/// Opens the database in the application's data directory, creating both when missing.
///
/// The store comes back beside the rules too: exporting and importing move the whole file,
/// which is the store's business rather than the rules'.
pub async fn open(app: &AppHandle<impl Runtime>) -> Result<(AppState, SqliteStore), SetupError> {
    let path = database_path(app)?;

    // The acceptance environment starts from the demo dataset rather than from an empty
    // database, so a fresh install has something to look at. Before the open, because
    // `SqliteStore::open` creates the file it does not find.
    #[cfg(feature = "acceptance")]
    crate::acceptance::seed(&path).map_err(|source| SetupError::SeedDemoDatabase {
        path: path.clone(),
        source,
    })?;

    let store = SqliteStore::open(&path).await?;
    let catalogue = OpenFoodFacts::new()?;
    // The assistant holds no key: the rules read the user's own out of the store and hand
    // it over per call, so there is no credential in the application's state.
    let assistant = Anthropic::new()?;
    Ok((Mealprep::new(store.clone(), catalogue, assistant), store))
}
