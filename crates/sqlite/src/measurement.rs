use mealprep_core::{
    Entity, Result,
    domain::{self, Measurement, MeasurementKind},
    store::MeasurementStore,
};
use time::Date;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const SET: &str = "INSERT INTO measurements (date, kind, value, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?) \
    ON CONFLICT (date, kind) DO UPDATE \
    SET value = excluded.value, updated_at = excluded.updated_at, deleted_at = NULL \
    RETURNING date, kind, value, created_at, updated_at";
const LIST: &str = "SELECT date, kind, value, created_at, updated_at FROM measurements \
    WHERE deleted_at IS NULL ORDER BY date, kind";
const DELETE: &str = "UPDATE measurements SET deleted_at = ?, updated_at = ? \
    WHERE date = ? AND kind = ? AND deleted_at IS NULL";

impl MeasurementStore for SqliteStore {
    async fn set_measurement(
        &self,
        date: Date,
        kind: MeasurementKind,
        value: f64,
    ) -> Result<Measurement> {
        let now = row::now();
        let measurement: MeasurementRow = sqlx::query_as(SET)
            .bind(domain::format_iso_date(date))
            .bind(kind.as_str())
            .bind(value)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        measurement.into_measurement()
    }

    async fn list_measurements(&self) -> Result<Vec<Measurement>> {
        let measurements: Vec<MeasurementRow> = sqlx::query_as(LIST)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        measurements
            .into_iter()
            .map(MeasurementRow::into_measurement)
            .collect()
    }

    async fn delete_measurement(&self, date: Date, kind: MeasurementKind) -> Result<()> {
        let now = row::now();
        let date = domain::format_iso_date(date);
        let deleted = sqlx::query(DELETE)
            .bind(&now)
            .bind(&now)
            .bind(&date)
            .bind(kind.as_str())
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if deleted.rows_affected() == 0 {
            let id = format!("{} on {date}", kind.as_str());
            return Err(row::not_found(Entity::Measurement, id));
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MeasurementRow {
    date: String,
    kind: String,
    value: f64,
    created_at: String,
    updated_at: String,
}

impl MeasurementRow {
    fn into_measurement(self) -> Result<Measurement> {
        let Self {
            date,
            kind,
            value,
            created_at,
            updated_at,
        } = self;

        let date = domain::parse_iso_date(&date).map_err(internal)?;
        let kind = MeasurementKind::parse(&kind)
            .ok_or_else(|| internal(format!("unknown measurement kind {kind:?}")))?;

        Ok(Measurement {
            date,
            kind,
            value,
            created_at: row::parse_timestamp(&created_at)?,
            updated_at: row::parse_timestamp(&updated_at)?,
        })
    }
}
