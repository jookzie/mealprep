use std::{fs, path::PathBuf};

use mealprep_core::service::Mealprep;
use mealprep_openfoodfacts::OpenFoodFacts;
use mealprep_sqlite::SqliteStore;
use tauri::{AppHandle, Manager, Runtime};

use crate::error::SetupError;

/// The business rules as the application runs them: SQLite on the device, Open Food Facts
/// as the catalogue.
///
/// Swapping in another database means another store implementing the `mealprep_core::store`
/// traits and changing this alias; no command changes.
pub type AppState = Mealprep<SqliteStore, OpenFoodFacts>;

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
pub async fn open(app: &AppHandle<impl Runtime>) -> Result<AppState, SetupError> {
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
    Ok(Mealprep::new(store, catalogue))
}
