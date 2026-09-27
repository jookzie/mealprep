use std::collections::BTreeMap;

use time::{Date, Duration};

use super::{Mealprep, stats};
use crate::{
    Result,
    domain::{EnergyBalance, HealthDay, Macros, WeightPoint},
    store::{
        CalendarStore, CategoryStore, DayPlanStore, HealthStore, MealStore, ProductStore,
        WeightStore,
    },
};

/// The window the balance is read over. MacroFactor's expenditure looks back about three
/// weeks: shorter and water swings dominate the trend, longer and it lags a real change.
const WINDOW_DAYS: i64 = 21;

/// Two thirds of the window planned, or the mean says more about the gaps than the plan.
const MIN_PLANNED_DAYS: u32 = 14;

/// The trend's last value may be this many days short of the window's end: a scale used
/// three times a week should not lose the estimate between weigh-ins.
const TREND_SLACK_DAYS: i64 = 3;

/// Energy per kilogram of body mass gained or lost, the usual approximation for a mix of
/// fat and lean tissue.
const KCAL_PER_KG: f64 = 7700.0;

impl<S, C> Mealprep<S, C>
where
    S: CalendarStore
        + CategoryStore
        + DayPlanStore
        + HealthStore
        + MealStore
        + ProductStore
        + WeightStore,
{
    /// The expenditure the plan and the weight trend imply over the last three weeks, next
    /// to what the wearable reported. Today is left out because it is not over yet.
    pub async fn energy_balance(&self, today: Date) -> Result<EnergyBalance> {
        let to = today - Duration::days(1);
        let from = to - Duration::days(WINDOW_DAYS - 1);
        let planned = self.planned_macros(from, to).await?;
        let weight = self.weight_series().await?;
        let health = self.store.list_health_days().await?;
        Ok(balance(from, to, &planned, &weight.points, &health))
    }
}

fn balance(
    from: Date,
    to: Date,
    planned: &BTreeMap<Date, Macros>,
    weight: &[WeightPoint],
    health: &[HealthDay],
) -> EnergyBalance {
    let intake: Vec<f64> = planned
        .range(from..=to)
        .map(|(_, macros)| macros.energy_kcal)
        .collect();
    let planned_days = intake.len() as u32;
    let planned_kcal = stats::mean(&intake);

    let trend = trend_change(from, to, weight);
    let stored_kcal = trend.map(|(change, days)| change * KCAL_PER_KG / days);
    let expenditure_kcal = match (planned_kcal, stored_kcal) {
        (Some(intake), Some(stored)) if planned_days >= MIN_PLANNED_DAYS => Some(intake - stored),
        _ => None,
    };

    let measured: Vec<f64> = health
        .iter()
        .filter(|day| (from..=to).contains(&day.date))
        .filter_map(|day| day.total_kcal)
        .collect();

    EnergyBalance {
        from,
        to,
        window_days: WINDOW_DAYS as u32,
        planned_days,
        planned_kcal,
        trend_change_kg: trend.map(|(change, _)| change),
        stored_kcal,
        expenditure_kcal,
        measured_days: measured.len() as u32,
        measured_kcal: stats::mean(&measured),
    }
}

/// How far the trend moved from the day before the window to its last value inside it,
/// and over how many days.
fn trend_change(from: Date, to: Date, points: &[WeightPoint]) -> Option<(f64, f64)> {
    let before = from - Duration::days(1);
    let start = points.iter().find(|point| point.date == before)?;
    let end = points
        .iter()
        .rev()
        .find(|point| point.date <= to && point.date >= to - Duration::days(TREND_SLACK_DAYS))?;
    let days = (end.date - start.date).whole_days();
    (days > 0).then_some((end.trend - start.trend, days as f64))
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    const FROM: Date = date!(2026 - 09 - 01);
    const TO: Date = date!(2026 - 09 - 21);

    fn plan(days: i64, energy_kcal: f64) -> BTreeMap<Date, Macros> {
        (0..days)
            .map(|offset| {
                let macros = Macros {
                    energy_kcal,
                    ..Macros::ZERO
                };
                (FROM + Duration::days(offset), macros)
            })
            .collect()
    }

    /// A trend falling linearly by the given kilograms a day, from the day before the window
    /// to its end.
    fn falling(per_day: f64) -> Vec<WeightPoint> {
        (0..=WINDOW_DAYS)
            .map(|offset| WeightPoint {
                date: FROM - Duration::days(1) + Duration::days(offset),
                kilograms: None,
                trend: 80.0 - per_day * offset as f64,
            })
            .collect()
    }

    #[test]
    fn a_falling_trend_means_burning_more_than_was_planned() {
        let balance = balance(FROM, TO, &plan(21, 2000.0), &falling(0.05), &[]);

        // 0.05 kg a day is 385 kcal a day released from storage.
        let expenditure = balance.expenditure_kcal.expect("a full window");
        assert!((expenditure - 2385.0).abs() < 1e-6, "was {expenditure}");
        assert_eq!(balance.planned_days, 21);
    }

    #[test]
    fn too_few_planned_days_give_no_estimate() {
        let balance = balance(FROM, TO, &plan(10, 2000.0), &falling(0.05), &[]);

        assert_eq!(balance.planned_days, 10);
        assert_eq!(balance.planned_kcal, Some(2000.0));
        assert!(balance.stored_kcal.is_some());
        assert_eq!(balance.expenditure_kcal, None);
    }

    #[test]
    fn a_trend_that_starts_inside_the_window_gives_no_estimate() {
        let late: Vec<_> = falling(0.05).into_iter().skip(5).collect();

        let balance = balance(FROM, TO, &plan(21, 2000.0), &late, &[]);

        assert_eq!(balance.trend_change_kg, None);
        assert_eq!(balance.expenditure_kcal, None);
    }

    #[test]
    fn a_trend_ending_a_couple_of_days_early_still_counts() {
        let early: Vec<_> = falling(0.05).into_iter().take(20).collect();

        let balance = balance(FROM, TO, &plan(21, 2000.0), &early, &[]);

        let expenditure = balance.expenditure_kcal.expect("within the slack");
        assert!((expenditure - 2385.0).abs() < 1e-6, "was {expenditure}");
    }

    #[test]
    fn the_wearable_figure_is_averaged_over_the_days_it_reported() {
        let health: Vec<_> = [
            (FROM, 2600.0),
            (TO, 2800.0),
            (TO + Duration::days(1), 9999.0),
        ]
        .into_iter()
        .map(|(date, kcal)| HealthDay {
            total_kcal: Some(kcal),
            ..HealthDay::empty(date)
        })
        .collect();

        let balance = balance(FROM, TO, &BTreeMap::new(), &[], &health);

        assert_eq!(balance.measured_days, 2);
        assert_eq!(balance.measured_kcal, Some(2700.0));
    }
}
