//! Moving the whole database in and out of the application as one file.
//!
//! The file is the only copy of the user's data, so an import is checked here before it
//! replaces anything: a file this build cannot open would stop the application starting,
//! and the only way back from that is an uninstall, which takes the data with it.

use std::{collections::HashMap, path::Path};

use mealprep_core::{Entity, Error, Result};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};

use crate::{MIGRATOR, SqliteStore, row::internal};

impl SqliteStore {
    /// Writes a consistent copy of the database to `path`, which must not exist yet.
    ///
    /// `VACUUM INTO` copies what is committed as one self-contained file, with whatever
    /// the write-ahead log still holds folded in, so the copy can be opened on its own.
    pub async fn snapshot_to(&self, path: &Path) -> Result<()> {
        let target = path.to_str().ok_or_else(|| Error::Internal {
            reason: format!("{} is not a UTF-8 path", path.display()),
        })?;
        sqlx::query("VACUUM INTO ?")
            .bind(target)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        Ok(())
    }
}

/// Checks that the file at `path` is a Mealprep database this build can open, and folds
/// any write-ahead log beside it into the file so the file can be moved on its own.
///
/// It writes to the file, so it is given a staged copy rather than the user's original.
/// A database in WAL mode cannot be read through a read-only connection at all, and a file
/// taken off a device usually is one.
///
/// Can be opened means an intact SQLite file whose applied migrations are all ones this build
/// carries, unchanged. Migrations it has not applied yet are fine — they run on the first
/// open, as after any update — but one this build does not know means a newer build wrote
/// the file, and opening it would fail.
pub async fn prepare_import(path: &Path) -> Result<()> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false);
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(|source| invalid(format!("cannot be opened as SQLite: {source}")))?;

    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut connection)
        .await
        .map_err(|source| invalid(format!("cannot be read as SQLite: {source}")))?;
    if integrity != "ok" {
        return Err(invalid(format!("is damaged: {integrity}")));
    }

    let applied: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations WHERE success")
            .fetch_all(&mut connection)
            .await
            .map_err(|_| invalid("is not a Mealprep database".to_owned()))?;
    if applied.is_empty() {
        return Err(invalid("is not a Mealprep database".to_owned()));
    }

    let known: HashMap<i64, &[u8]> = MIGRATOR
        .iter()
        .map(|migration| (migration.version, migration.checksum.as_ref()))
        .collect();
    for (version, checksum) in &applied {
        match known.get(version) {
            None => {
                return Err(invalid(format!(
                    "was written by a newer version of Mealprep (migration {version}); update the app first"
                )));
            }
            Some(expected) if *expected != checksum.as_slice() => {
                return Err(invalid(format!(
                    "has a migration {version} that differs from this version's"
                )));
            }
            Some(_) => {}
        }
    }

    // Rollback-journal mode on the way out removes the log once it is folded in, so the
    // file is complete by itself; the store switches back to WAL when it opens it.
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&mut connection)
        .await
        .map_err(internal)?;
    sqlx::query("PRAGMA journal_mode = DELETE")
        .execute(&mut connection)
        .await
        .map_err(internal)?;
    connection.close().await.map_err(internal)?;
    Ok(())
}

fn invalid(reason: String) -> Error {
    Error::Invalid {
        entity: Entity::Database,
        reason: format!("the file {reason}"),
    }
}
