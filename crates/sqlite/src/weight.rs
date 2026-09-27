use mealprep_core::{
    Entity, Result,
    domain::{self, WeightEntry},
    store::WeightStore,
};
use time::Date;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const SET: &str = "INSERT INTO weight_entries (date, kilograms, created_at, updated_at) \
    VALUES (?, ?, ?, ?) \
    ON CONFLICT (date) DO UPDATE \
    SET kilograms = excluded.kilograms, updated_at = excluded.updated_at, deleted_at = NULL \
    RETURNING *";
const LIST: &str = "SELECT * FROM weight_entries WHERE deleted_at IS NULL ORDER BY date";
const DELETE: &str = "UPDATE weight_entries SET deleted_at = ?, updated_at = ? \
    WHERE date = ? AND deleted_at IS NULL";

impl WeightStore for SqliteStore {
    async fn set_weight_entry(&self, date: Date, kilograms: f64) -> Result<WeightEntry> {
        let now = row::now();
        let entry: WeightRow = sqlx::query_as(SET)
            .bind(domain::format_iso_date(date))
            .bind(kilograms)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        entry.into_weight_entry()
    }

    async fn list_weight_entries(&self) -> Result<Vec<WeightEntry>> {
        let entries: Vec<WeightRow> = sqlx::query_as(LIST)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        entries
            .into_iter()
            .map(WeightRow::into_weight_entry)
            .collect()
    }

    async fn delete_weight_entry(&self, date: Date) -> Result<()> {
        let now = row::now();
        let date = domain::format_iso_date(date);
        let deleted = sqlx::query(DELETE)
            .bind(&now)
            .bind(&now)
            .bind(&date)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if deleted.rows_affected() == 0 {
            return Err(row::not_found(Entity::WeightEntry, date));
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct WeightRow {
    date: String,
    kilograms: f64,
    created_at: String,
    updated_at: String,
}

impl WeightRow {
    fn into_weight_entry(self) -> Result<WeightEntry> {
        let Self {
            date,
            kilograms,
            created_at,
            updated_at,
        } = self;

        let date = domain::parse_iso_date(&date).map_err(internal)?;
        let created_at = row::parse_timestamp(&created_at)?;
        let updated_at = row::parse_timestamp(&updated_at)?;

        Ok(WeightEntry {
            date,
            kilograms,
            created_at,
            updated_at,
        })
    }
}
