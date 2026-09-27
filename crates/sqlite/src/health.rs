use mealprep_core::{
    Result,
    domain::{self, HealthDay, HealthSync, NightSleep},
    store::HealthStore,
};
use time::Date;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const CLEAR_RANGE: &str = "UPDATE health_days SET deleted_at = ?, updated_at = ? \
    WHERE date BETWEEN ? AND ? AND deleted_at IS NULL";
const UPSERT: &str = "INSERT INTO health_days (date, bed_minute, wake_minute, asleep_minutes, \
        light_minutes, deep_minutes, rem_minutes, awake_minutes, resting_heart_rate, hrv_ms, \
        respiratory_rate, active_kcal, total_kcal, exercise_minutes, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
    ON CONFLICT (date) DO UPDATE \
    SET bed_minute = excluded.bed_minute, wake_minute = excluded.wake_minute, \
        asleep_minutes = excluded.asleep_minutes, light_minutes = excluded.light_minutes, \
        deep_minutes = excluded.deep_minutes, rem_minutes = excluded.rem_minutes, \
        awake_minutes = excluded.awake_minutes, resting_heart_rate = excluded.resting_heart_rate, \
        hrv_ms = excluded.hrv_ms, respiratory_rate = excluded.respiratory_rate, \
        active_kcal = excluded.active_kcal, total_kcal = excluded.total_kcal, \
        exercise_minutes = excluded.exercise_minutes, updated_at = excluded.updated_at, \
        deleted_at = NULL";
const LIST: &str = "SELECT date, bed_minute, wake_minute, asleep_minutes, light_minutes, \
        deep_minutes, rem_minutes, awake_minutes, resting_heart_rate, hrv_ms, respiratory_rate, \
        active_kcal, total_kcal, exercise_minutes \
    FROM health_days WHERE deleted_at IS NULL ORDER BY date";
const SET_SYNC: &str = "INSERT INTO health_sync (id, synced_at, through, created_at, updated_at) \
    VALUES (1, ?, ?, ?, ?) \
    ON CONFLICT (id) DO UPDATE \
    SET synced_at = excluded.synced_at, through = excluded.through, \
        updated_at = excluded.updated_at, deleted_at = NULL";
const GET_SYNC: &str =
    "SELECT synced_at, through FROM health_sync WHERE id = 1 AND deleted_at IS NULL";

impl HealthStore for SqliteStore {
    async fn replace_health_days(
        &self,
        from: Date,
        days: &[HealthDay],
        sync: HealthSync,
    ) -> Result<()> {
        let now = row::now();
        let mut transaction = self.pool.begin().await.map_err(internal)?;

        sqlx::query(CLEAR_RANGE)
            .bind(&now)
            .bind(&now)
            .bind(domain::format_iso_date(from))
            .bind(domain::format_iso_date(sync.through))
            .execute(&mut *transaction)
            .await
            .map_err(internal)?;

        for day in days {
            let sleep = day.sleep;
            sqlx::query(UPSERT)
                .bind(domain::format_iso_date(day.date))
                .bind(sleep.map(|sleep| sleep.bed_minute))
                .bind(sleep.map(|sleep| sleep.wake_minute))
                .bind(sleep.map(|sleep| sleep.asleep_minutes))
                .bind(sleep.and_then(|sleep| sleep.light_minutes))
                .bind(sleep.and_then(|sleep| sleep.deep_minutes))
                .bind(sleep.and_then(|sleep| sleep.rem_minutes))
                .bind(sleep.and_then(|sleep| sleep.awake_minutes))
                .bind(day.resting_heart_rate)
                .bind(day.hrv_ms)
                .bind(day.respiratory_rate)
                .bind(day.active_kcal)
                .bind(day.total_kcal)
                .bind(day.exercise_minutes)
                .bind(&now)
                .bind(&now)
                .execute(&mut *transaction)
                .await
                .map_err(internal)?;
        }

        let synced_at = row::format_timestamp(sync.synced_at);
        sqlx::query(SET_SYNC)
            .bind(&synced_at)
            .bind(domain::format_iso_date(sync.through))
            .bind(&now)
            .bind(&now)
            .execute(&mut *transaction)
            .await
            .map_err(internal)?;

        transaction.commit().await.map_err(internal)
    }

    async fn list_health_days(&self) -> Result<Vec<HealthDay>> {
        let days: Vec<HealthDayRow> = sqlx::query_as(LIST)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        days.into_iter()
            .map(HealthDayRow::into_health_day)
            .collect()
    }

    async fn last_health_sync(&self) -> Result<Option<HealthSync>> {
        let sync: Option<(String, String)> = sqlx::query_as(GET_SYNC)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        sync.map(|(synced_at, through)| {
            Ok(HealthSync {
                synced_at: row::parse_timestamp(&synced_at)?,
                through: domain::parse_iso_date(&through).map_err(internal)?,
            })
        })
        .transpose()
    }
}

#[derive(sqlx::FromRow)]
struct HealthDayRow {
    date: String,
    bed_minute: Option<i32>,
    wake_minute: Option<i32>,
    asleep_minutes: Option<f64>,
    light_minutes: Option<f64>,
    deep_minutes: Option<f64>,
    rem_minutes: Option<f64>,
    awake_minutes: Option<f64>,
    resting_heart_rate: Option<f64>,
    hrv_ms: Option<f64>,
    respiratory_rate: Option<f64>,
    active_kcal: Option<f64>,
    total_kcal: Option<f64>,
    exercise_minutes: Option<f64>,
}

impl HealthDayRow {
    fn into_health_day(self) -> Result<HealthDay> {
        let Self {
            date,
            bed_minute,
            wake_minute,
            asleep_minutes,
            light_minutes,
            deep_minutes,
            rem_minutes,
            awake_minutes,
            resting_heart_rate,
            hrv_ms,
            respiratory_rate,
            active_kcal,
            total_kcal,
            exercise_minutes,
        } = self;

        let sleep = match (bed_minute, wake_minute, asleep_minutes) {
            (Some(bed_minute), Some(wake_minute), Some(asleep_minutes)) => Some(NightSleep {
                bed_minute,
                wake_minute,
                asleep_minutes,
                light_minutes,
                deep_minutes,
                rem_minutes,
                awake_minutes,
            }),
            _ => None,
        };

        Ok(HealthDay {
            date: domain::parse_iso_date(&date).map_err(internal)?,
            sleep,
            resting_heart_rate,
            hrv_ms,
            respiratory_rate,
            active_kcal,
            total_kcal,
            exercise_minutes,
        })
    }
}
