use mealprep_core::{
    Entity, Result,
    domain::{self, CalendarDay},
    store::CalendarStore,
};
use time::Date;
use uuid::Uuid;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const ASSIGN: &str = "INSERT INTO calendar_days (date, day_plan_id, created_at, updated_at) \
    VALUES (?, ?, ?, ?) \
    ON CONFLICT (date) DO UPDATE \
    SET day_plan_id = excluded.day_plan_id, updated_at = excluded.updated_at, deleted_at = NULL \
    RETURNING *";
const LIST: &str = "SELECT * FROM calendar_days \
    WHERE date >= ? AND date <= ? AND deleted_at IS NULL ORDER BY date";
const UNASSIGN: &str = "UPDATE calendar_days SET deleted_at = ?, updated_at = ? \
    WHERE date = ? AND deleted_at IS NULL";

impl CalendarStore for SqliteStore {
    async fn assign_calendar_day(&self, date: Date, day_plan_id: Uuid) -> Result<CalendarDay> {
        let now = row::now();
        let day: CalendarRow = sqlx::query_as(ASSIGN)
            .bind(domain::format_iso_date(date))
            .bind(day_plan_id.to_string())
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        day.into_calendar_day()
    }

    async fn list_calendar_days(&self, from: Date, to: Date) -> Result<Vec<CalendarDay>> {
        let days: Vec<CalendarRow> = sqlx::query_as(LIST)
            .bind(domain::format_iso_date(from))
            .bind(domain::format_iso_date(to))
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        days.into_iter()
            .map(CalendarRow::into_calendar_day)
            .collect()
    }

    async fn unassign_calendar_day(&self, date: Date) -> Result<()> {
        let now = row::now();
        let date = domain::format_iso_date(date);
        let cleared = sqlx::query(UNASSIGN)
            .bind(&now)
            .bind(&now)
            .bind(&date)
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if cleared.rows_affected() == 0 {
            return Err(row::not_found(Entity::CalendarDay, date));
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct CalendarRow {
    date: String,
    day_plan_id: String,
    created_at: String,
    updated_at: String,
}

impl CalendarRow {
    fn into_calendar_day(self) -> Result<CalendarDay> {
        let Self {
            date,
            day_plan_id,
            created_at,
            updated_at,
        } = self;

        let date = domain::parse_iso_date(&date).map_err(internal)?;
        let day_plan_id = row::parse_id(&day_plan_id)?;
        let created_at = row::parse_timestamp(&created_at)?;
        let updated_at = row::parse_timestamp(&updated_at)?;

        Ok(CalendarDay {
            date,
            day_plan_id,
            created_at,
            updated_at,
        })
    }
}
