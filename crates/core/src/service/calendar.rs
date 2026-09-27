use time::Date;
use uuid::Uuid;

use super::Mealprep;
use crate::{
    Entity, Error, Result,
    domain::CalendarDay,
    store::{CalendarStore, DayPlanStore},
};

/// The longest range a calendar read may span; longer is an export, not a view.
const MAX_RANGE_DAYS: i64 = 366;

impl<S, C> Mealprep<S, C>
where
    S: CalendarStore + DayPlanStore,
{
    /// Lists the days from `from` to `to` inclusive that have a plan, in date order.
    pub async fn list_calendar_days(&self, from: Date, to: Date) -> Result<Vec<CalendarDay>> {
        if to < from {
            return Err(Error::Invalid {
                entity: Entity::CalendarDay,
                reason: format!("the range ends on {to}, before it starts on {from}"),
            });
        }
        if (to - from).whole_days() > MAX_RANGE_DAYS {
            return Err(Error::Invalid {
                entity: Entity::CalendarDay,
                reason: format!("the range spans more than {MAX_RANGE_DAYS} days"),
            });
        }
        self.store.list_calendar_days(from, to).await
    }

    /// Puts the given day plan on the given date, replacing whatever was there.
    pub async fn assign_calendar_day(&self, date: Date, day_plan_id: Uuid) -> Result<CalendarDay> {
        match self.store.get_day_plan(day_plan_id).await {
            Ok(_) => {}
            Err(Error::NotFound { .. }) => {
                return Err(Error::Invalid {
                    entity: Entity::CalendarDay,
                    reason: format!("day plan {day_plan_id} does not exist"),
                });
            }
            Err(error) => return Err(error),
        }
        self.store.assign_calendar_day(date, day_plan_id).await
    }

    pub async fn unassign_calendar_day(&self, date: Date) -> Result<()> {
        self.store.unassign_calendar_day(date).await?;
        Ok(())
    }
}
