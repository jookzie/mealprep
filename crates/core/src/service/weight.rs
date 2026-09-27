use time::{Date, Duration};

use super::{Mealprep, validate};
use crate::{
    Result,
    domain::{WeightEntry, WeightPoint, WeightSeries},
    store::WeightStore,
};

/// How much of a new measurement the trend takes on each day.
///
/// A tenth, from the Hacker's Diet: it lags like a twenty-day moving average while needing
/// no window and discarding no history. Day-to-day weight is mostly water, gut contents and
/// hormonal noise, so the raw series carries little signal on its own; this is the figure
/// worth reading.
const SMOOTHING: f64 = 0.1;

/// The span the rate of change is measured over.
const RATE_WINDOW_DAYS: i64 = 14;

/// The shortest history that gets a rate at all. Below this the slope is noise wearing a
/// number, and a wrong figure here is one someone would act on.
const MIN_RATE_DAYS: i64 = 7;

impl<S, C> Mealprep<S, C>
where
    S: WeightStore,
{
    /// Lists every weigh-in, most recent first, for the history to read.
    pub async fn list_weight_entries(&self) -> Result<Vec<WeightEntry>> {
        let mut entries = self.store.list_weight_entries().await?;
        entries.reverse();
        Ok(entries)
    }

    /// Records the weight for the given date, replacing whatever was there.
    pub async fn set_weight_entry(&self, date: Date, kilograms: f64) -> Result<WeightEntry> {
        let kilograms = validate::weight(kilograms)?;
        self.store.set_weight_entry(date, kilograms).await
    }

    pub async fn delete_weight_entry(&self, date: Date) -> Result<()> {
        self.store.delete_weight_entry(date).await
    }

    /// The weigh-ins as a daily series, with the trend and what it implies.
    pub async fn weight_series(&self) -> Result<WeightSeries> {
        let entries = self.store.list_weight_entries().await?;
        Ok(series(&entries))
    }
}

/// Builds the daily series from the weigh-ins, in date order.
fn series(entries: &[WeightEntry]) -> WeightSeries {
    let (Some(first), Some(last)) = (entries.first(), entries.last()) else {
        return WeightSeries::default();
    };

    let points = smooth(&fill(entries));
    let trend = points.last().map(|point| point.trend);
    let span = (last.date - first.date).whole_days();

    WeightSeries {
        rate_per_week: rate_per_week(&points, span),
        trend,
        points,
    }
}

/// Expands the weigh-ins to one value per day, interpolating the days that were skipped.
///
/// Without this a gap would smooth as though it were a single day, so three weeks away from
/// the scale would move the trend as fast as three days of weighing in. The interpolated
/// days carry no measurement, so the graph draws no point for them.
fn fill(entries: &[WeightEntry]) -> Vec<(Date, Option<f64>, f64)> {
    let mut filled = Vec::new();
    for pair in entries.windows(2) {
        let (from, to) = (&pair[0], &pair[1]);
        let span = (to.date - from.date).whole_days();
        filled.push((from.date, Some(from.kilograms), from.kilograms));
        for step in 1..span {
            let fraction = step as f64 / span as f64;
            filled.push((
                from.date + Duration::days(step),
                None,
                from.kilograms + (to.kilograms - from.kilograms) * fraction,
            ));
        }
    }
    if let Some(last) = entries.last() {
        filled.push((last.date, Some(last.kilograms), last.kilograms));
    }
    filled
}

/// Runs the exponentially smoothed average over the filled days.
///
/// The trend starts at the first measurement rather than at zero, so the line begins where
/// the user does instead of climbing out of the floor for the first month.
fn smooth(filled: &[(Date, Option<f64>, f64)]) -> Vec<WeightPoint> {
    let Some((_, _, seed)) = filled.first() else {
        return Vec::new();
    };
    let mut trend = *seed;
    filled
        .iter()
        .map(|(date, kilograms, value)| {
            trend += (value - trend) * SMOOTHING;
            WeightPoint {
                date: *date,
                kilograms: *kilograms,
                trend,
            }
        })
        .collect()
}

/// The slope of the trend over the last fortnight, in kilograms per week.
fn rate_per_week(points: &[WeightPoint], span: i64) -> Option<f64> {
    if span < MIN_RATE_DAYS {
        return None;
    }
    let window = RATE_WINDOW_DAYS.min(span);
    let last = points.last()?;
    let earlier = points.get(points.len().checked_sub(1 + window as usize)?)?;
    Some((last.trend - earlier.trend) / window as f64 * 7.0)
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, macros::date};

    use super::*;

    fn entry(date: Date, kilograms: f64) -> WeightEntry {
        let now = OffsetDateTime::UNIX_EPOCH;
        WeightEntry {
            date,
            kilograms,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn an_empty_history_has_no_series() {
        let series = series(&[]);

        assert!(series.points.is_empty());
        assert_eq!(series.trend, None);
        assert_eq!(series.rate_per_week, None);
    }

    #[test]
    fn a_single_weigh_in_is_its_own_trend() {
        let series = series(&[entry(date!(2026 - 09 - 01), 80.0)]);

        assert_eq!(series.points.len(), 1);
        assert_eq!(series.trend, Some(80.0));
        assert_eq!(series.rate_per_week, None);
    }

    #[test]
    fn the_trend_lags_behind_a_jump() {
        let entries = [
            entry(date!(2026 - 09 - 01), 80.0),
            entry(date!(2026 - 09 - 02), 90.0),
        ];

        let series = series(&entries);

        // A ten-kilogram jump moves the trend by a tenth of it, not by all of it.
        let trend = series.trend.expect("two weigh-ins have a trend");
        assert!((trend - 81.0).abs() < 1e-9, "trend was {trend}");
    }

    #[test]
    fn a_skipped_day_is_interpolated_and_carries_no_measurement() {
        let entries = [
            entry(date!(2026 - 09 - 01), 80.0),
            entry(date!(2026 - 09 - 03), 82.0),
        ];

        let series = series(&entries);

        assert_eq!(series.points.len(), 3);
        assert_eq!(series.points[1].date, date!(2026 - 09 - 02));
        assert_eq!(series.points[1].kilograms, None);
        assert_eq!(series.points[0].kilograms, Some(80.0));
        assert_eq!(series.points[2].kilograms, Some(82.0));
    }

    #[test]
    fn a_gap_smooths_day_by_day_rather_than_all_at_once() {
        let apart = [
            entry(date!(2026 - 09 - 01), 80.0),
            entry(date!(2026 - 09 - 21), 90.0),
        ];
        let adjacent = [
            entry(date!(2026 - 09 - 01), 80.0),
            entry(date!(2026 - 09 - 02), 90.0),
        ];

        let over_a_gap = series(&apart).trend.expect("a trend");
        let in_one_day = series(&adjacent).trend.expect("a trend");

        assert!(
            over_a_gap > in_one_day,
            "three weeks of climbing ({over_a_gap}) should reach further than one day ({in_one_day})"
        );
    }

    #[test]
    fn a_short_history_gets_no_rate() {
        let entries = [
            entry(date!(2026 - 09 - 01), 80.0),
            entry(date!(2026 - 09 - 05), 79.0),
        ];

        assert_eq!(series(&entries).rate_per_week, None);
    }

    #[test]
    fn steady_loss_reports_a_negative_rate() {
        let entries: Vec<_> = (0..28)
            .map(|day| {
                entry(
                    date!(2026 - 09 - 01) + Duration::days(day),
                    80.0 - 0.1 * day as f64,
                )
            })
            .collect();

        let rate = series(&entries)
            .rate_per_week
            .expect("four weeks of weigh-ins have a rate");

        // Losing 0.1 kg a day is 0.7 a week; the trend lags, so it reads a little under.
        assert!(
            (-0.75..=-0.5).contains(&rate),
            "rate was {rate} kg/week, expected about -0.7"
        );
    }
}
