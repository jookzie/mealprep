//! Moving the whole database in and out of the application as one file.
//!
//! Both go through the system's file picker, so the file can be saved to or read from
//! anywhere the phone offers — Downloads, a cloud drive, a USB stick. On Android the
//! picker answers with a content URI rather than a path, which the fs plugin opens through
//! the platform; nothing here reads a path the user typed.
//!
//! Before an import replaces anything, what is there now is kept in the application's own
//! folder in shared storage, `Android/data/<application id>/files/Download` on a phone,
//! which needs no permission and which a computer reaches over USB or adb.

use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use mealprep_core::Error as CoreError;
use mealprep_sqlite::{SqliteStore, prepare_import};
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_fs::{FsExt, OpenOptions};
use time::{OffsetDateTime, macros::format_description};

use crate::{error::Error, result::Result, setup};

/// Asks where to save a copy of everything and writes it there. Answers `false` when the
/// user closed the picker without choosing.
#[tauri::command]
pub async fn export_database(app: AppHandle, store: State<'_, SqliteStore>) -> Result<bool> {
    let name = format!("mealprep-{}.db", stamp());
    let dialog = app.dialog().file().set_file_name(&name);
    // Android picks the kind of document to create by MIME type, and has none for `.db`.
    #[cfg(target_os = "android")]
    let dialog = dialog.add_filter("Mealprep database", &["application/octet-stream"]);
    #[cfg(not(target_os = "android"))]
    let dialog = dialog.add_filter("Mealprep database", &["db"]);
    let Some(target) = choose(move || dialog.blocking_save_file()).await? else {
        return Ok(false);
    };

    // Taken after the choice rather than before, so the copy is as of the moment it lands.
    let snapshot = scratch(&app, &name)?;
    store.snapshot_to(&snapshot).await?;
    let written = copy_to(&app, &snapshot, target);
    fs::remove_file(&snapshot).ok();
    written.map_err(internal)?;
    Ok(true)
}

/// Asks for a database file and replaces everything with it, then closes the application,
/// which has to be reopened to read it. Answers `false` when the user chose nothing.
///
/// What is there now is kept first, so an import is never the only copy of anything. The
/// file is checked on a staged copy before anything is replaced, because a database this
/// build cannot open would stop the application starting. Closing is part of the operation
/// for the reason `reset_demo_data` gives: the open store holds the old file by descriptor,
/// so only a restart reads the new one.
#[tauri::command]
pub async fn import_database(app: AppHandle, store: State<'_, SqliteStore>) -> Result<bool> {
    let dialog = app.dialog().file();
    let Some(source) = choose(move || dialog.blocking_pick_file()).await? else {
        return Ok(false);
    };

    let database = setup::database_path(&app).map_err(internal)?;
    let staged = database.with_extension("db.importing");
    remove_with_sidecars(&staged).map_err(internal)?;
    copy_from(&app, source, &staged).map_err(internal)?;
    if let Err(error) = prepare_import(&staged).await {
        remove_with_sidecars(&staged).ok();
        return Err(error.into());
    }

    let backups = backup_folder(&app)?;
    store
        .snapshot_to(&backups.join(format!("mealprep-before-import-{}.db", stamp())))
        .await?;

    // Renamed into place, so an interruption leaves either the old database or the new
    // one where the application looks, never half of either. The old log and its index
    // describe the replaced file and would be replayed over the new one at the next open.
    fs::rename(&staged, &database).map_err(internal)?;
    remove_sidecars(&database).map_err(internal)?;
    remove_sidecars(&staged).map_err(internal)?;

    app.exit(0);
    Ok(true)
}

/// Runs a blocking picker off the async runtime: on Android it waits for an activity
/// result, which must not hold a runtime thread.
async fn choose(
    pick: impl FnOnce() -> Option<FilePath> + Send + 'static,
) -> Result<Option<FilePath>> {
    tauri::async_runtime::spawn_blocking(pick)
        .await
        .map_err(internal)
}

fn copy_to(app: &AppHandle<impl Runtime>, from: &Path, to: FilePath) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    let mut target = app.fs().open(to, options)?;
    io::copy(&mut File::open(from)?, &mut target)?;
    target.sync_all()
}

fn copy_from(app: &AppHandle<impl Runtime>, from: FilePath, to: &Path) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.read(true);
    let mut source = app.fs().open(from, options)?;
    io::copy(&mut source, &mut File::create(to)?)?;
    Ok(())
}

/// A file in the application's cache, for a snapshot on its way to where the user chose.
fn scratch(app: &AppHandle<impl Runtime>, name: &str) -> Result<PathBuf> {
    let folder = app.path().app_cache_dir().map_err(internal)?;
    fs::create_dir_all(&folder).map_err(internal)?;
    let path = folder.join(name);
    remove_with_sidecars(&path).map_err(internal)?;
    Ok(path)
}

fn backup_folder(app: &AppHandle<impl Runtime>) -> Result<PathBuf> {
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

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
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
