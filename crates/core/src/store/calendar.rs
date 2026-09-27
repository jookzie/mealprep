use time::Date;
use uuid::Uuid;

use crate::{Result, domain::CalendarDay};

/// Persistence of which day plan is on which date.
#[trait_variant::make(Send)]
pub trait CalendarStore {
    /// Puts the given day plan on the given date, replacing whatever was there.
    async fn assign_calendar_day(&self, date: Date, day_plan_id: Uuid) -> Result<CalendarDay>;

    /// Lists the assigned days from `from` to `to` inclusive, in date order.
    async fn list_calendar_days(&self, from: Date, to: Date) -> Result<Vec<CalendarDay>>;

    async fn unassign_calendar_day(&self, date: Date) -> Result<()>;
}
