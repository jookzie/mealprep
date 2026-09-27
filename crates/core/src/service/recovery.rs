use std::collections::BTreeMap;

use time::{Date, Duration};

use super::{Mealprep, stats};
use crate::{
    Result,
    domain::{
        Band, BaselineStatus, Behaviour, HealthDay, Impact, Macros, Marker, MarkerPoint,
        MarkerSeries, Recovery,
    },
    store::{
        CalendarStore, CategoryStore, DayPlanStore, HealthStore, MealStore, ProductStore,
        TargetsStore,
    },
};

/// The rolling average is the signal: a single morning's HRV swings by 5–13 % on its own,
/// which is noise to anyone deciding anything from it.
const AVERAGE_DAYS: i64 = 7;
const MIN_AVERAGE_READINGS: usize = 3;

/// The normal range is the last two months: long enough to be the user's own, short enough
/// to follow a change in fitness. Plews and Buchheit's "smallest worthwhile change" sets its
/// width at half a standard deviation either side of the mean.
const BAND_DAYS: i64 = 60;
const MIN_BAND_READINGS: usize = 14;
const BAND_WIDTH: f64 = 0.5;

/// Behaviours are compared over 90 days and need five mornings on each side, as WHOOP's
/// Journal does before it shows an impact at all.
const IMPACT_DAYS: i64 = 90;
const MIN_SIDE_DAYS: u32 = 5;
/// The fewest marker readings in the window that make it worth comparing on.
const MIN_IMPACT_READINGS: usize = 2 * MIN_SIDE_DAYS as usize;
/// A difference smaller than this many standard deviations is day-to-day variation.
const CLEAR_DIFFERENCE: f64 = 0.5;

const SEVEN_HOURS: f64 = 7.0 * 60.0;
/// The fewest nights a median bedtime is taken from.
const MIN_BEDTIME_NIGHTS: usize = 10;

impl<S, C> Mealprep<S, C>
where
    S: HealthStore
        + CalendarStore
        + CategoryStore
        + DayPlanStore
        + MealStore
        + ProductStore
        + TargetsStore,
{
    /// HRV and resting heart rate against their normal ranges, and what went with better or
    /// worse mornings.
    pub async fn recovery(&self, today: Date) -> Result<Recovery> {
        let days = self.store.list_health_days().await?;
        let from = today - Duration::days(IMPACT_DAYS);
        let planned = self.planned_macros(from, today).await?;
        let targets = self
            .store
            .get_targets()
            .await?
            .map(|targets| targets.macros);
        Ok(recovery(&days, &planned, targets, today))
    }
}

fn recovery(
    days: &[HealthDay],
    planned: &BTreeMap<Date, Macros>,
    targets: Option<Macros>,
    today: Date,
) -> Recovery {
    let by_date: BTreeMap<Date, &HealthDay> = days.iter().map(|day| (day.date, day)).collect();
    let hrv_values = values(days, |day| day.hrv_ms);
    let rhr_values = values(days, |day| day.resting_heart_rate);

    let window = today - Duration::days(IMPACT_DAYS - 1)..=today;
    let in_window = |values: &BTreeMap<Date, f64>| values.range(window.clone()).count();
    let impact_marker = if in_window(&hrv_values) >= MIN_IMPACT_READINGS {
        Some((Marker::Hrv, &hrv_values))
    } else if in_window(&rhr_values) >= MIN_IMPACT_READINGS {
        Some((Marker::RestingHeartRate, &rhr_values))
    } else {
        None
    };

    let context = Context {
        days: &by_date,
        planned,
        targets,
        typical_bedtime: typical_bedtime(&by_date, *window.start(), today),
    };
    let impacts = impact_marker
        .map(|(_, values)| impacts(values, &context, *window.start(), today))
        .unwrap_or_default();

    Recovery {
        hrv: marker_series(Marker::Hrv, &hrv_values, today),
        resting_heart_rate: marker_series(Marker::RestingHeartRate, &rhr_values, today),
        impact_marker: impact_marker.map(|(marker, _)| marker),
        impacts,
    }
}

fn values(days: &[HealthDay], figure: impl Fn(&HealthDay) -> Option<f64>) -> BTreeMap<Date, f64> {
    days.iter()
        .filter_map(|day| Some((day.date, figure(day)?)))
        .collect()
}

fn dates(from: Date, to: Date) -> impl Iterator<Item = Date> {
    (0..=(to - from).whole_days()).map(move |offset| from + Duration::days(offset))
}

fn window_values(values: &BTreeMap<Date, f64>, date: Date, days: i64) -> Vec<f64> {
    values
        .range(date - Duration::days(days - 1)..=date)
        .map(|(_, value)| *value)
        .collect()
}

fn average_at(values: &BTreeMap<Date, f64>, date: Date) -> Option<f64> {
    let week = window_values(values, date, AVERAGE_DAYS);
    if week.len() < MIN_AVERAGE_READINGS {
        return None;
    }
    stats::mean(&week)
}

fn band_at(values: &BTreeMap<Date, f64>, date: Date) -> Option<Band> {
    let recent = window_values(values, date, BAND_DAYS);
    if recent.len() < MIN_BAND_READINGS {
        return None;
    }
    let mean = stats::mean(&recent)?;
    let half = stats::standard_deviation(&recent)? * BAND_WIDTH;
    Some(Band {
        low: mean - half,
        high: mean + half,
    })
}

fn marker_series(
    marker: Marker,
    values: &BTreeMap<Date, f64>,
    today: Date,
) -> Option<MarkerSeries> {
    let (first, _) = values.range(..=today).next()?;
    let points: Vec<MarkerPoint> = dates(*first, today)
        .map(|date| MarkerPoint {
            date,
            value: values.get(&date).copied(),
            average: average_at(values, date),
            band: band_at(values, date),
        })
        .collect();
    let latest = points.last()?;
    let status = match (latest.average, latest.band) {
        (Some(average), Some(band)) if average < band.low => Some(BaselineStatus::Below),
        (Some(average), Some(band)) if average > band.high => Some(BaselineStatus::Above),
        (Some(_), Some(_)) => Some(BaselineStatus::Within),
        _ => None,
    };
    Some(MarkerSeries {
        marker,
        average: latest.average,
        band: latest.band,
        status,
        points,
    })
}

/// What the behaviours are read from.
struct Context<'a> {
    days: &'a BTreeMap<Date, &'a HealthDay>,
    planned: &'a BTreeMap<Date, Macros>,
    targets: Option<Macros>,
    typical_bedtime: Option<f64>,
}

impl Context<'_> {
    /// Whether the behaviour happened for the given morning, or `None` when there is no way
    /// to tell: a morning without the data is neither side of the comparison.
    fn happened(&self, behaviour: Behaviour, morning: Date) -> Option<bool> {
        let day_before = morning - Duration::days(1);
        match behaviour {
            Behaviour::SleptSevenHours => {
                let sleep = self.days.get(&morning)?.sleep?;
                Some(sleep.asleep_minutes >= SEVEN_HOURS)
            }
            Behaviour::EarlierBedtime => {
                let typical = self.typical_bedtime?;
                let sleep = self.days.get(&morning)?.sleep?;
                Some(f64::from(sleep.bed_minute) < typical)
            }
            Behaviour::TrainedDayBefore => {
                let day = self.days.get(&day_before)?;
                // A day the source reported no activity for at all may be a day the strap
                // was off, which says nothing about training.
                let reported = day.active_kcal.is_some()
                    || day.total_kcal.is_some()
                    || day.exercise_minutes.is_some();
                reported.then(|| day.exercise_minutes.unwrap_or(0.0) > 0.0)
            }
            Behaviour::EnergyOverTargetDayBefore => {
                let (plan, targets) = (self.planned.get(&day_before)?, self.targets?);
                Some(plan.energy_kcal > targets.energy_kcal)
            }
            Behaviour::ProteinAtTargetDayBefore => {
                let (plan, targets) = (self.planned.get(&day_before)?, self.targets?);
                Some(plan.protein_g >= targets.protein_g)
            }
        }
    }
}

const BEHAVIOURS: [Behaviour; 5] = [
    Behaviour::SleptSevenHours,
    Behaviour::EarlierBedtime,
    Behaviour::TrainedDayBefore,
    Behaviour::EnergyOverTargetDayBefore,
    Behaviour::ProteinAtTargetDayBefore,
];

fn typical_bedtime(days: &BTreeMap<Date, &HealthDay>, from: Date, to: Date) -> Option<f64> {
    let bedtimes: Vec<f64> = days
        .range(from..=to)
        .filter_map(|(_, day)| Some(f64::from(day.sleep?.bed_minute)))
        .collect();
    if bedtimes.len() < MIN_BEDTIME_NIGHTS {
        return None;
    }
    stats::median(&bedtimes)
}

fn impacts(
    values: &BTreeMap<Date, f64>,
    context: &Context<'_>,
    from: Date,
    to: Date,
) -> Vec<Impact> {
    let mornings: Vec<(Date, f64)> = values
        .range(from..=to)
        .map(|(date, value)| (*date, *value))
        .collect();
    let all: Vec<f64> = mornings.iter().map(|(_, value)| *value).collect();
    let Some(deviation) = stats::standard_deviation(&all) else {
        return Vec::new();
    };

    BEHAVIOURS
        .into_iter()
        .filter_map(|behaviour| {
            let (mut with, mut without) = (Vec::new(), Vec::new());
            for (date, value) in &mornings {
                match context.happened(behaviour, *date) {
                    Some(true) => with.push(*value),
                    Some(false) => without.push(*value),
                    None => {}
                }
            }
            let (with_days, without_days) = (with.len() as u32, without.len() as u32);
            if with_days < MIN_SIDE_DAYS || without_days < MIN_SIDE_DAYS {
                return None;
            }
            let (with_mean, without_mean) = (stats::mean(&with)?, stats::mean(&without)?);
            Some(Impact {
                behaviour,
                with_days,
                without_days,
                with_mean,
                without_mean,
                change_percent: (with_mean - without_mean) / without_mean * 100.0,
                clear: (with_mean - without_mean).abs() >= deviation * CLEAR_DIFFERENCE,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;
    use crate::domain::NightSleep;

    const TODAY: Date = date!(2026 - 09 - 30);

    fn day(date: Date) -> HealthDay {
        HealthDay::empty(date)
    }

    fn night(asleep_minutes: f64, bed_minute: i32) -> NightSleep {
        NightSleep {
            bed_minute,
            wake_minute: bed_minute + asleep_minutes as i32,
            asleep_minutes,
            light_minutes: None,
            deep_minutes: None,
            rem_minutes: None,
            awake_minutes: None,
        }
    }

    /// Sixty mornings of HRV alternating around 60 ms, ending today.
    fn steady_hrv() -> Vec<HealthDay> {
        (0..60)
            .map(|offset| {
                let mut day = day(TODAY - Duration::days(59 - offset));
                day.hrv_ms = Some(if offset % 2 == 0 { 55.0 } else { 65.0 });
                day
            })
            .collect()
    }

    #[test]
    fn nothing_imported_is_nothing_to_show() {
        let recovery = recovery(&[], &BTreeMap::new(), None, TODAY);

        assert_eq!(recovery.hrv, None);
        assert_eq!(recovery.resting_heart_rate, None);
        assert_eq!(recovery.impact_marker, None);
        assert!(recovery.impacts.is_empty());
    }

    #[test]
    fn a_steady_marker_sits_within_its_own_range() {
        let recovery = recovery(&steady_hrv(), &BTreeMap::new(), None, TODAY);

        let hrv = recovery.hrv.expect("sixty mornings of HRV");
        assert_eq!(hrv.points.len(), 60);
        assert_eq!(hrv.status, Some(BaselineStatus::Within));
        let band = hrv.band.expect("a band");
        assert!(band.low < 60.0 && 60.0 < band.high);
    }

    #[test]
    fn a_week_of_low_readings_falls_below_the_range() {
        let mut days = steady_hrv();
        for day in days.iter_mut().rev().take(7) {
            day.hrv_ms = Some(40.0);
        }

        let recovery = recovery(&days, &BTreeMap::new(), None, TODAY);

        assert_eq!(
            recovery.hrv.expect("a series").status,
            Some(BaselineStatus::Below)
        );
    }

    #[test]
    fn a_short_history_has_readings_but_no_range() {
        let days: Vec<_> = steady_hrv().into_iter().rev().take(5).collect();

        let hrv = recovery(&days, &BTreeMap::new(), None, TODAY)
            .hrv
            .expect("a series");

        assert!(hrv.average.is_some());
        assert_eq!(hrv.band, None);
        assert_eq!(hrv.status, None);
    }

    #[test]
    fn the_series_runs_to_today_even_when_the_last_reading_is_older() {
        let days: Vec<_> = steady_hrv()
            .into_iter()
            .filter(|day| day.date <= TODAY - Duration::days(3))
            .collect();

        let hrv = recovery(&days, &BTreeMap::new(), None, TODAY)
            .hrv
            .expect("a series");

        assert_eq!(hrv.points.last().map(|point| point.date), Some(TODAY));
        assert_eq!(hrv.points.last().and_then(|point| point.value), None);
    }

    #[test]
    fn long_nights_that_go_with_higher_hrv_are_a_clear_impact() {
        let days: Vec<_> = (0..30)
            .map(|offset| {
                let long = offset % 2 == 0;
                let mut day = day(TODAY - Duration::days(offset));
                day.hrv_ms = Some(if long { 70.0 } else { 50.0 });
                day.sleep = Some(night(if long { 480.0 } else { 360.0 }, -60));
                day
            })
            .collect();

        let recovery = recovery(&days, &BTreeMap::new(), None, TODAY);

        assert_eq!(recovery.impact_marker, Some(Marker::Hrv));
        let sleep = recovery
            .impacts
            .iter()
            .find(|impact| impact.behaviour == Behaviour::SleptSevenHours)
            .expect("enough nights on both sides");
        assert_eq!((sleep.with_days, sleep.without_days), (15, 15));
        assert!((sleep.change_percent - 40.0).abs() < 1e-9);
        assert!(sleep.clear);
    }

    #[test]
    fn a_behaviour_with_fewer_than_five_mornings_on_a_side_is_not_shown() {
        let days: Vec<_> = (0..30)
            .map(|offset| {
                let mut day = day(TODAY - Duration::days(offset));
                day.hrv_ms = Some(60.0 + offset as f64 % 3.0);
                day.sleep = Some(night(if offset < 4 { 300.0 } else { 480.0 }, -60));
                day
            })
            .collect();

        let recovery = recovery(&days, &BTreeMap::new(), None, TODAY);

        assert!(
            recovery
                .impacts
                .iter()
                .all(|impact| impact.behaviour != Behaviour::SleptSevenHours)
        );
    }

    #[test]
    fn resting_heart_rate_stands_in_when_hrv_is_missing() {
        let days: Vec<_> = (0..20)
            .map(|offset| {
                let mut day = day(TODAY - Duration::days(offset));
                day.resting_heart_rate = Some(52.0 + (offset % 4) as f64);
                day
            })
            .collect();

        let recovery = recovery(&days, &BTreeMap::new(), None, TODAY);

        assert_eq!(recovery.hrv, None);
        assert_eq!(recovery.impact_marker, Some(Marker::RestingHeartRate));
    }

    #[test]
    fn the_plan_for_the_day_before_is_what_counts() {
        let targets = Macros {
            energy_kcal: 2500.0,
            protein_g: 150.0,
            ..Macros::ZERO
        };
        let mut planned = BTreeMap::new();
        let days: Vec<_> = (0..20)
            .map(|offset| {
                let date = TODAY - Duration::days(offset);
                let over = offset % 2 == 0;
                planned.insert(
                    date - Duration::days(1),
                    Macros {
                        energy_kcal: if over { 3000.0 } else { 2000.0 },
                        protein_g: 150.0,
                        ..Macros::ZERO
                    },
                );
                let mut day = day(date);
                day.hrv_ms = Some(if over { 50.0 } else { 60.0 });
                day
            })
            .collect();

        let recovery = recovery(&days, &planned, Some(targets), TODAY);

        let energy = recovery
            .impacts
            .iter()
            .find(|impact| impact.behaviour == Behaviour::EnergyOverTargetDayBefore)
            .expect("ten planned days on each side");
        assert_eq!(energy.with_mean, 50.0);
        assert_eq!(energy.without_mean, 60.0);
        // Every day met the protein target, so there is nothing to compare it against.
        assert!(
            recovery
                .impacts
                .iter()
                .all(|impact| impact.behaviour != Behaviour::ProteinAtTargetDayBefore)
        );
    }
}
