use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

/// One day plan assigned to one date; a date holds at most one.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDay {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    /// Kept even when the plan has since been deleted, so the day can still be cleared.
    pub day_plan_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
