use time::{Date, Duration};

use super::{Mealprep, stats};
use crate::{
    Result,
    domain::{HealthDay, SleepNight, SleepSummary},
    store::HealthStore,
};

/// Duration is read over a week, like every other rolling figure here.
const AVERAGE_DAYS: i64 = 7;
const MIN_AVERAGE_NIGHTS: usize = 3;

/// Regularity needs more nights than duration does: research on a year of WHOOP data finds
/// one or two weeks too few to judge sleep variability, so it is read over four and not
/// at all below two.
const REGULARITY_DAYS: i64 = 28;
const MIN_REGULARITY_NIGHTS: usize = 14;

impl<S, C> Mealprep<S, C>
where
    S: HealthStore,
{
    /// Every imported night, with how long and how regularly they were slept.
    pub async fn sleep_summary(&self, today: Date) -> Result<SleepSummary> {
        let days = self.store.list_health_days().await?;
        Ok(summary(&days, today))
    }
}

fn summary(days: &[HealthDay], today: Date) -> SleepSummary {
    let nights: Vec<SleepNight> = days
        .iter()
        .filter(|day| day.date <= today)
        .filter_map(|day| {
            Some(SleepNight {
                date: day.date,
                night: day.sleep?,
            })
        })
        .collect();

    let since = |days: i64| {
        let from = today - Duration::days(days - 1);
        nights.iter().filter(move |night| night.date >= from)
    };

    let week: Vec<f64> = since(AVERAGE_DAYS)
        .map(|night| night.night.asleep_minutes)
        .collect();
    let bedtimes: Vec<f64> = since(REGULARITY_DAYS)
        .map(|night| f64::from(night.night.bed_minute))
        .collect();
    let wakes: Vec<f64> = since(REGULARITY_DAYS)
        .map(|night| f64::from(night.night.wake_minute))
        .collect();
    let regular = bedtimes.len() >= MIN_REGULARITY_NIGHTS;

    SleepSummary {
        average_minutes: (week.len() >= MIN_AVERAGE_NIGHTS)
            .then(|| stats::mean(&week))
            .flatten(),
        typical_bed_minute: stats::median(&bedtimes),
        typical_wake_minute: stats::median(&wakes),
        bedtime_spread_minutes: regular
            .then(|| stats::standard_deviation(&bedtimes))
            .flatten(),
        wake_spread_minutes: regular.then(|| stats::standard_deviation(&wakes)).flatten(),
        nights,
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;
    use crate::domain::NightSleep;

    const TODAY: Date = date!(2026 - 09 - 30);

    fn day(offset: i64, bed_minute: i32, asleep_minutes: f64) -> HealthDay {
        let mut day = HealthDay::empty(TODAY - Duration::days(offset));
        day.sleep = Some(NightSleep {
            bed_minute,
            wake_minute: bed_minute + asleep_minutes as i32,
            asleep_minutes,
            light_minutes: None,
            deep_minutes: None,
            rem_minutes: None,
            awake_minutes: None,
        });
        day
    }

    #[test]
    fn a_few_nights_give_an_average_but_no_regularity() {
        let days: Vec<_> = (0..5).map(|offset| day(offset, -60, 420.0)).collect();

        let summary = summary(&days, TODAY);

        assert_eq!(summary.nights.len(), 5);
        assert_eq!(summary.average_minutes, Some(420.0));
        assert_eq!(summary.typical_bed_minute, Some(-60.0));
        assert_eq!(summary.bedtime_spread_minutes, None);
    }

    #[test]
    fn identical_bedtimes_have_no_spread_and_varying_ones_do() {
        let steady: Vec<_> = (0..20).map(|offset| day(offset, -30, 450.0)).collect();
        let shifting: Vec<_> = (0..20)
            .map(|offset| day(offset, if offset % 2 == 0 { -120 } else { 0 }, 450.0))
            .collect();

        assert_eq!(summary(&steady, TODAY).bedtime_spread_minutes, Some(0.0));
        let spread = summary(&shifting, TODAY)
            .bedtime_spread_minutes
            .expect("twenty nights");
        assert!(spread > 55.0, "spread was {spread}");
    }

    #[test]
    fn a_day_without_a_night_is_not_a_night_of_no_sleep() {
        let mut days: Vec<_> = (0..3).map(|offset| day(offset, -60, 480.0)).collect();
        days.push(HealthDay {
            resting_heart_rate: Some(50.0),
            ..HealthDay::empty(TODAY - Duration::days(4))
        });

        let summary = summary(&days, TODAY);

        assert_eq!(summary.nights.len(), 3);
        assert_eq!(summary.average_minutes, Some(480.0));
    }
}
