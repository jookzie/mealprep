//! SQLite implementation of the Mealprep storage contracts.
//!
//! One database file holds every entity. The schema lives in `migrations/` and is brought
//! up to date whenever a store is opened.

mod calendar;
mod category;
mod day_plan;
mod health;
mod meal;
mod measurement;
mod product;
mod row;
mod targets;
mod weight;

use std::{path::Path, str::FromStr};

use mealprep_core::Result;
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use self::row::internal;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Stores every Mealprep entity in one SQLite database.
#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Opens the database file at the given path, creating it when missing.
    pub async fn open(path: &Path) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(internal)?;
        Self::migrate(pool).await
    }

    /// Opens a private database that lives in memory for as long as this store does.
    pub async fn open_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(internal)?
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(options)
            .await
            .map_err(internal)?;
        Self::migrate(pool).await
    }

    async fn migrate(pool: SqlitePool) -> Result<Self> {
        MIGRATOR.run(&pool).await.map_err(internal)?;
        Ok(Self { pool })
    }
}
