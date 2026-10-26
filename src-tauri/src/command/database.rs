//! Moving the whole database in and out of the application as one file.
//!
//! Both go through one folder: the application's own directory in shared storage,
//! `Android/data/<application id>/files/Download` on a phone. The application needs no
//! permission for it, and a computer reaches it over USB or adb. Android removes it with the
//! application, so an export only counts as a backup once it is copied off the phone.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use mealprep_core::{Entity, Error as CoreError};
use mealprep_sqlite::{SqliteStore, prepare_import};
use tauri::{AppHandle, Manager, Runtime, State};
use time::{OffsetDateTime, macros::format_description};

use crate::{error::Error, result::Result, setup};

/// The file an import reads. One fixed name, so an import never picks up an older export
/// that happens to sit in the same folder.
const IMPORT_FILE: &str = "mealprep-import.db";

/// Where exports land and the import is read from, as a path to show the user.
#[tauri::command]
pub async fn database_folder(app: AppHandle) -> Result<String> {
    Ok(folder(&app)?.display().to_string())
}

/// Writes a copy of everything to the folder and answers the file's name.
#[tauri::command]
pub async fn export_database(app: AppHandle, store: State<'_, SqliteStore>) -> Result<String> {
    let path = folder(&app)?.join(format!("mealprep-export-{}.db", stamp()));
    store.snapshot_to(&path).await?;
    Ok(file_name(&path))
}

/// Replaces the database with `mealprep-import.db` from the folder and closes the
/// application, which has to be reopened to read it.
///
/// What is there now is exported first, so an import is never the only copy of anything.
/// The file is checked on a staged copy before anything is replaced, because a database
/// this build cannot open would stop the application starting. Closing is part of the
/// operation for the reason `reset_demo_data` gives: the open store holds the old file by
/// descriptor, so only a restart reads the new one.
#[tauri::command]
pub async fn import_database(app: AppHandle, store: State<'_, SqliteStore>) -> Result<()> {
    let folder = folder(&app)?;
    let source = folder.join(IMPORT_FILE);
    if !source.exists() {
        return Err(CoreError::Invalid {
            entity: Entity::Database,
            reason: format!("there is no {IMPORT_FILE} in {}", folder.display()),
        }
        .into());
    }

    let database = setup::database_path(&app).map_err(internal)?;
    let staged = database.with_extension("db.importing");
    remove_with_sidecars(&staged).map_err(internal)?;
    copy_with_sidecars(&source, &staged).map_err(internal)?;
    if let Err(error) = prepare_import(&staged).await {
        remove_with_sidecars(&staged).ok();
        return Err(error.into());
    }

    let stamp = stamp();
    store
        .snapshot_to(&folder.join(format!("mealprep-before-import-{stamp}.db")))
        .await?;

    // Renamed into place, so an interruption leaves either the old database or the new
    // one where the application looks, never half of either. The old log and its index
    // describe the replaced file and would be replayed over the new one at the next open.
    fs::rename(&staged, &database).map_err(internal)?;
    remove_sidecars(&database).map_err(internal)?;
    remove_sidecars(&staged).map_err(internal)?;

    // Out of the way, so the same file cannot be imported twice by accident.
    fs::rename(
        &source,
        folder.join(format!("mealprep-imported-{stamp}.db")),
    )
    .map_err(internal)?;
    remove_sidecars(&source).map_err(internal)?;

    app.exit(0);
    Ok(())
}

fn folder(app: &AppHandle<impl Runtime>) -> Result<PathBuf> {
    let folder = app.path().download_dir().map_err(internal)?;
    fs::create_dir_all(&folder).map_err(internal)?;
    Ok(folder)
}

/// UTC to the second, so two exports a minute apart do not collide and the names sort in
/// the order they were taken. UTC because reading the local offset needs a `time` feature
/// that is unsound in a multi-threaded process, which this is.
fn stamp() -> String {
    OffsetDateTime::now_utc()
        .format(format_description!(
            "[year][month][day]-[hour][minute][second]Z"
        ))
        .expect("internal error: a timestamp always formats")
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// A file copied off a device may come with the write-ahead log that holds its newest
/// writes; the log travels with it and `prepare_import` folds it in.
fn copy_with_sidecars(from: &Path, to: &Path) -> io::Result<()> {
    fs::copy(from, to)?;
    for suffix in ["-wal", "-shm"] {
        let log = sidecar(from, suffix);
        if log.exists() {
            fs::copy(&log, sidecar(to, suffix))?;
        }
    }
    Ok(())
}

fn remove_sidecars(path: &Path) -> io::Result<()> {
    for suffix in ["-wal", "-shm"] {
        let log = sidecar(path, suffix);
        if log.exists() {
            fs::remove_file(log)?;
        }
    }
    Ok(())
}

fn remove_with_sidecars(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    remove_sidecars(path)
}

fn internal(reason: impl std::fmt::Display) -> Error {
    CoreError::Internal {
        reason: reason.to_string(),
    }
    .into()
}
