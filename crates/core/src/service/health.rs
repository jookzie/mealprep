use std::collections::BTreeMap;

use time::{Date, Duration, OffsetDateTime};

use super::Mealprep;
use crate::{
    Entity, Error, Result,
    domain::{
        HealthDay, HealthImport, HealthSync, NightSleep, Reading, SleepSession, SleepStageKind,
    },
    store::HealthStore,
};

/// How far back the first import asks for.
///
/// Health Connect answers with at most 30 days from before access was granted unless the
/// history permission was granted too, so asking for a year costs nothing when it was not.
const FIRST_IMPORT_DAYS: i64 = 365;

/// How many already-imported days every later import reads again.
///
/// A strap uploads when it next reaches the phone, so a night can land in Health Connect a
/// day or two after it happened; re-reading a few days picks it up.
const OVERLAP_DAYS: i64 = 3;

/// The longest range one import may replace.
const MAX_IMPORT_DAYS: i64 = 400;

/// The hour from which a reading counts towards the next morning.
///
/// Recovery readings are taken overnight, and a night belongs to the morning it ends on.
/// A reading stamped late in the evening is the start of that night, not the end of the
/// day before.
const EVENING_HOUR: u8 = 18;

impl<S, C> Mealprep<S, C>
where
    S: HealthStore,
{
    /// The first day the next import should cover.
    pub async fn health_import_start(&self, today: Date) -> Result<Date> {
        let start = match self.store.last_health_sync().await? {
            Some(sync) => sync.through - Duration::days(OVERLAP_DAYS),
            None => today - Duration::days(FIRST_IMPORT_DAYS),
        };
        Ok(start.min(today))
    }

    /// Summarises what the source read for `from` to `today` and stores it, replacing the
    /// days it covers.
    pub async fn import_health(
        &self,
        from: Date,
        today: Date,
        import: HealthImport,
        now: OffsetDateTime,
    ) -> Result<HealthSync> {
        if today < from {
            return Err(Error::Invalid {
                entity: Entity::HealthDay,
                reason: format!("the import ends on {today}, before it starts on {from}"),
            });
        }
        if (today - from).whole_days() > MAX_IMPORT_DAYS {
            return Err(Error::Invalid {
                entity: Entity::HealthDay,
                reason: format!("an import spans at most {MAX_IMPORT_DAYS} days"),
            });
        }
        let days = summarise(&import, from, today);
        let sync = HealthSync {
            synced_at: now,
            through: today,
        };
        self.store.replace_health_days(from, &days, sync).await?;
        Ok(sync)
    }

    /// Every imported day, in date order.
    pub async fn list_health_days(&self) -> Result<Vec<HealthDay>> {
        self.store.list_health_days().await
    }

    pub async fn last_health_sync(&self) -> Result<Option<HealthSync>> {
        self.store.last_health_sync().await
    }
}

/// Folds the raw records into one row per morning from `from` to `through`, in date order.
/// Mornings with nothing on them are left out.
pub(super) fn summarise(import: &HealthImport, from: Date, through: Date) -> Vec<HealthDay> {
    let mut days: BTreeMap<Date, HealthDay> = BTreeMap::new();

    for (date, session) in main_sleep(&import.sleep) {
        day(&mut days, date).sleep = Some(night(date, session));
    }
    for (date, value) in morning_means(&import.resting_heart_rate) {
        day(&mut days, date).resting_heart_rate = Some(value);
    }
    for (date, value) in morning_means(&import.heart_rate_variability) {
        day(&mut days, date).hrv_ms = Some(value);
    }
    for (date, value) in morning_means(&import.respiratory_rate) {
        day(&mut days, date).respiratory_rate = Some(value);
    }
    for activity in &import.activity {
        let entry = day(&mut days, activity.date);
        entry.active_kcal = activity.active_kcal;
        entry.total_kcal = activity.total_kcal;
        entry.exercise_minutes = activity.exercise_minutes;
    }

    days.into_values()
        .filter(|day| (from..=through).contains(&day.date) && !day.is_empty())
        .collect()
}

fn day(days: &mut BTreeMap<Date, HealthDay>, date: Date) -> &mut HealthDay {
    days.entry(date).or_insert_with(|| HealthDay::empty(date))
}

/// The morning a moment counts towards.
fn morning_of(at: OffsetDateTime) -> Date {
    if at.hour() >= EVENING_HOUR {
        at.date() + Duration::days(1)
    } else {
        at.date()
    }
}

/// The longest session ending on each morning.
///
/// Naps are separate sessions, and a second app can record the same night again; in both
/// cases the longest session is the night's main sleep, and adding them up would count
/// the same hours twice.
fn main_sleep(sessions: &[SleepSession]) -> BTreeMap<Date, &SleepSession> {
    let mut main: BTreeMap<Date, &SleepSession> = BTreeMap::new();
    for session in sessions
        .iter()
        .filter(|session| session.end > session.start)
    {
        let date = session.end.date();
        let longer = main
            .get(&date)
            .is_none_or(|current| session.end - session.start > current.end - current.start);
        if longer {
            main.insert(date, session);
        }
    }
    main
}

fn night(morning: Date, session: &SleepSession) -> NightSleep {
    let minute_of = |at: OffsetDateTime| -> i32 {
        let days = (at.date() - morning).whole_days() as i32;
        days * 24 * 60 + i32::from(at.hour()) * 60 + i32::from(at.minute())
    };
    let bed_minute = minute_of(session.start);
    let wake_minute = minute_of(session.end);

    if session.stages.is_empty() {
        return NightSleep {
            bed_minute,
            wake_minute,
            asleep_minutes: minutes(session.end - session.start),
            light_minutes: None,
            deep_minutes: None,
            rem_minutes: None,
            awake_minutes: None,
        };
    }

    let total = |wanted: &[SleepStageKind]| -> f64 {
        session
            .stages
            .iter()
            .filter(|stage| wanted.contains(&stage.kind) && stage.end > stage.start)
            .map(|stage| minutes(stage.end - stage.start))
            .sum()
    };
    let light = total(&[SleepStageKind::Light]);
    let deep = total(&[SleepStageKind::Deep]);
    let rem = total(&[SleepStageKind::Rem]);
    let staged = light + deep + rem > 0.0;
    let stage = |value: f64| staged.then_some(value);

    NightSleep {
        bed_minute,
        wake_minute,
        asleep_minutes: light + deep + rem + total(&[SleepStageKind::Sleeping]),
        light_minutes: stage(light),
        deep_minutes: stage(deep),
        rem_minutes: stage(rem),
        awake_minutes: Some(total(&[SleepStageKind::Awake, SleepStageKind::OutOfBed])),
    }
}

/// The mean of the readings counted towards each morning.
///
/// A mean rather than the last reading, because two apps writing the same figure should
/// not have the later sync decide it.
fn morning_means(readings: &[Reading]) -> BTreeMap<Date, f64> {
    let mut sums: BTreeMap<Date, (f64, u32)> = BTreeMap::new();
    for reading in readings.iter().filter(|reading| reading.value.is_finite()) {
        let (sum, count) = sums.entry(morning_of(reading.at)).or_default();
        *sum += reading.value;
        *count += 1;
    }
    sums.into_iter()
        .map(|(date, (sum, count))| (date, sum / f64::from(count)))
        .collect()
}

fn minutes(duration: Duration) -> f64 {
    duration.whole_seconds() as f64 / 60.0
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;
    use crate::domain::{ActivityDay, SleepStage};

    fn stage(start: OffsetDateTime, end: OffsetDateTime, kind: SleepStageKind) -> SleepStage {
        SleepStage { start, end, kind }
    }

    #[test]
    fn a_night_belongs_to_the_morning_it_ends_on() {
        let import = HealthImport {
            sleep: vec![SleepSession {
                start: datetime!(2026-09-01 23:30 +02:00),
                end: datetime!(2026-09-02 07:00 +02:00),
                stages: Vec::new(),
            }],
            ..HealthImport::default()
        };

        let days = summarise(&import, date!(2026 - 09 - 01), date!(2026 - 09 - 03));

        assert_eq!(days.len(), 1);
        assert_eq!(days[0].date, date!(2026 - 09 - 02));
        let sleep = days[0].sleep.expect("the night");
        assert_eq!(sleep.bed_minute, -30);
        assert_eq!(sleep.wake_minute, 7 * 60);
        assert_eq!(sleep.asleep_minutes, 450.0);
        assert_eq!(sleep.deep_minutes, None);
    }

    #[test]
    fn stages_split_the_night_and_awake_time_is_not_sleep() {
        let import = HealthImport {
            sleep: vec![SleepSession {
                start: datetime!(2026-09-02 00:00 UTC),
                end: datetime!(2026-09-02 04:00 UTC),
                stages: vec![
                    stage(
                        datetime!(2026-09-02 00:00 UTC),
                        datetime!(2026-09-02 01:00 UTC),
                        SleepStageKind::Light,
                    ),
                    stage(
                        datetime!(2026-09-02 01:00 UTC),
                        datetime!(2026-09-02 02:00 UTC),
                        SleepStageKind::Deep,
                    ),
                    stage(
                        datetime!(2026-09-02 02:00 UTC),
                        datetime!(2026-09-02 02:30 UTC),
                        SleepStageKind::Awake,
                    ),
                    stage(
                        datetime!(2026-09-02 02:30 UTC),
                        datetime!(2026-09-02 04:00 UTC),
                        SleepStageKind::Rem,
                    ),
                ],
            }],
            ..HealthImport::default()
        };

        let days = summarise(&import, date!(2026 - 09 - 02), date!(2026 - 09 - 02));
        let sleep = days[0].sleep.expect("the night");

        assert_eq!(sleep.asleep_minutes, 210.0);
        assert_eq!(sleep.deep_minutes, Some(60.0));
        assert_eq!(sleep.rem_minutes, Some(90.0));
        assert_eq!(sleep.awake_minutes, Some(30.0));
    }

    #[test]
    fn a_nap_does_not_replace_or_add_to_the_night() {
        let night = SleepSession {
            start: datetime!(2026-09-01 23:00 UTC),
            end: datetime!(2026-09-02 07:00 UTC),
            stages: Vec::new(),
        };
        let nap = SleepSession {
            start: datetime!(2026-09-02 14:00 UTC),
            end: datetime!(2026-09-02 14:30 UTC),
            stages: Vec::new(),
        };
        let import = HealthImport {
            sleep: vec![nap, night],
            ..HealthImport::default()
        };

        let days = summarise(&import, date!(2026 - 09 - 02), date!(2026 - 09 - 02));

        assert_eq!(days[0].sleep.expect("the night").asleep_minutes, 480.0);
    }

    #[test]
    fn an_evening_reading_counts_towards_the_next_morning() {
        let import = HealthImport {
            heart_rate_variability: vec![
                Reading {
                    at: datetime!(2026-09-01 23:10 UTC),
                    value: 50.0,
                },
                Reading {
                    at: datetime!(2026-09-02 04:00 UTC),
                    value: 70.0,
                },
            ],
            ..HealthImport::default()
        };

        let days = summarise(&import, date!(2026 - 09 - 01), date!(2026 - 09 - 02));

        assert_eq!(days.len(), 1);
        assert_eq!(days[0].date, date!(2026 - 09 - 02));
        assert_eq!(days[0].hrv_ms, Some(60.0));
    }

    #[test]
    fn days_outside_the_range_and_empty_days_are_left_out() {
        let import = HealthImport {
            activity: vec![
                ActivityDay {
                    date: date!(2026 - 08 - 31),
                    active_kcal: Some(500.0),
                    total_kcal: Some(2500.0),
                    exercise_minutes: None,
                },
                ActivityDay {
                    date: date!(2026 - 09 - 01),
                    active_kcal: None,
                    total_kcal: None,
                    exercise_minutes: None,
                },
                ActivityDay {
                    date: date!(2026 - 09 - 02),
                    active_kcal: Some(300.0),
                    total_kcal: Some(2300.0),
                    exercise_minutes: Some(45.0),
                },
            ],
            ..HealthImport::default()
        };

        let days = summarise(&import, date!(2026 - 09 - 01), date!(2026 - 09 - 02));

        assert_eq!(days.len(), 1);
        assert_eq!(days[0].date, date!(2026 - 09 - 02));
        assert_eq!(days[0].exercise_minutes, Some(45.0));
    }
}
