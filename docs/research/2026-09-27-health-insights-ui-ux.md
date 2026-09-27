# UI/UX Research → Body measurements and Health Connect insights

> **Status: research deliverable, 2026-09-27.** Part 4 (decisions) is implemented. Part 5 is
> what was rejected or deferred, and why.

## Context

The app plans food against targets and trends the weight those targets exist to move. The
brief adds two data sources: physical measurements typed in by hand, and what a WHOOP strap
writes to Android's Health Connect. The question is which insights from those sources are
established enough to show — the ones other apps have converged on and that the research
backs — and how to show them without cluttering an app that already has five tabs.

## 1. What WHOOP actually puts in Health Connect

WHOOP's integration is two-way, but what it *writes* is narrower than what it measures:

- **Written**: sleep sessions (with stages), workouts, heart rate, energy burned. Resting
  heart rate and respiratory rate come with the Health Monitor features.
- **Not written**: Recovery, Strain and Sleep Performance scores are WHOOP's own and stay in
  WHOOP. HRV is reported inconsistently: some sources say it is written, others that it is
  not, and the answer has changed between app versions.
- **Read** by WHOOP: weight and height, to improve its calorie estimates.

Consequence for the design: **every series is optional.** A screen that assumes HRV exists
breaks on the day WHOOP stops writing it. Each insight names the data it needs and simply
does not appear without it; where HRV is missing, resting heart rate stands in, because it
is the other half of every recovery score (WHOOP, Oura, Garmin all combine the two).

## 2. The insights other services have converged on

Ordered by how strong the evidence is and how directly it can be computed from this data.

### 2.1 Recovery: today against your own normal range

Every recovery product compares the latest HRV and resting heart rate against **the user's
own baseline**, never against a population. Absolute HRV differs by device (Oura's member
average is ~41 ms, WHOOP's ~62 ms, because they sample different windows), by age and by
person, so a population chart is meaningless to a single user.

The method with research behind it is the one HRV4Training popularised from Plews and
Buchheit:

- a **7-day rolling average** as the signal (a single morning is noise; WHOOP HRV's
  day-to-day coefficient of variation is roughly 5–13 %),
- a **normal range** from the last ~60 days: mean ± 0.5 × standard deviation (the
  "smallest worthwhile change"),
- the verdict is where the 7-day average sits: inside the band, below it, or above it.

For resting heart rate the reading is inverted — above the band is the warning sign.

### 2.2 Sleep: duration and regularity

WHOOP (Sleep Consistency), Oura (bedtime regularity) and the sleep literature all treat
**timing regularity** as its own dimension next to duration: irregular bed and wake times are
associated with worse cardiometabolic markers independently of total sleep. The honest
measure for one person is the variability of bedtime and wake time; research on WHOOP data
shows one or two weeks is too few nights to judge variability, so a four-week window is used.

Sleep stages are shown per night but not charted: consumer stage classification agrees with
polysomnography only moderately, and a stage chart is the kind of detail that invites
over-reading.

### 2.3 What moves your recovery

WHOOP's Journal ("Recovery Impact") and Oura's tags answer "which of my habits go with
better or worse recovery?" by comparing the mornings after a behaviour with the mornings
after its absence. WHOOP requires **at least five "yes" and five "no" days within 90 days**
before it shows anything, and labels the result as association, not causation.

This app can do the same without asking the user to journal, because it already knows the
behaviours:

| Behaviour | Source |
| --- | --- |
| Slept at least 7 hours | Health Connect sleep |
| Went to bed earlier than usual | Health Connect sleep (bedtime vs the 90-day median) |
| Trained the day before | Health Connect workouts |
| Planned energy over target the day before | the calendar and the targets |
| Planned protein at target the day before | the calendar and the targets |

The same guard applies: five of each side within 90 days, and a difference smaller than
half a standard deviation is reported as "no clear difference" rather than as a percentage
someone would act on.

### 2.4 Energy balance: what the weight trend says you burn

MacroFactor's core idea: over about three weeks, intake minus the energy stored or released
(read off the **weight trend**, not the scale) is the expenditure. It is more reliable than
a wrist estimate, which is typically off by tens of percent. This app knows intake — from
the plan, not a food log, so that caveat must travel with the figure — and it knows the
trend. WHOOP's own burned-calories figure is shown next to it as a comparison, not as truth.

Energy density of body mass change: 7,700 kcal per kilogram, the usual approximation for
mixed fat-and-lean change.

### 2.5 Body measurements

- **Waist is the measurement with the strongest evidence** among tape measures. Waist-to-
  height ratio is adopted by NICE (2022): below 0.5 is healthy central adiposity, 0.5–0.59
  increased, 0.6 and over high. It needs one division and no age or sex tables.
- Waist changes by fractions of a centimetre a week even on a good cut, so it is a
  **weekly-to-monthly** signal; the chart draws the measurements as they are, with no daily
  smoothing.
- The established "recomposition" read is **waist against weight over the same weeks**:
  weight holding while the waist shrinks is the pattern people look for. The screen puts the
  two changes side by side and lets the user read them, rather than naming a verdict.

## 3. UI patterns

- **Bottom bar stays at five.** Both platforms cap it there; the precedent is weight, which
  lives behind a summary card rather than taking a tab.
- **A hub of one-line cards.** Oura's and WHOOP's home screens lead with a handful of
  headline figures, each tapping through to one detail screen. Each card here carries one
  figure and one qualifier ("58 ms · within your normal range"), no chart.
- **One chart per detail screen**, the same visual language as the weight chart: ink for
  the figure worth reading, muted for the raw points, a tinted band for the normal range.
  No new hues and no red: status is a word, not a colour, as the macro meters already do.
- **Nothing appears before it means something.** A card with too little data says how
  much more it needs ("a week of readings gives a baseline") rather than drawing a line
  through three points.
- **Health Connect is opt-in from one card.** Before access is granted there is exactly one
  row about it. Sync runs when the hub opens, at most hourly, plus a manual button.

## 4. Decisions taken

| Decision | Why |
| --- | --- |
| The Targets tab becomes **Body**, a hub | Targets, weight, measurements and health are all "what the plan is measured against". A hub keeps each one a tap away without a sixth tab. |
| Imported health data is stored as **one row per day** | Health Connect is read, summarised to the morning it belongs to, and kept as a snapshot, the way catalogue imports are. Every insight then reads local data and works offline. |
| A night belongs to **the morning it ends** | The calendar and the macros are per day; joining "last night" to "yesterday's food" needs one rule. Readings taken from 18:00 onwards count towards the next morning. |
| Only the **longest sleep of a night** is used | WHOOP writes naps as separate sessions, and a second app can write the same night again. The longest session is the main sleep in both cases. |
| **Energy per day is aggregated by Health Connect**, not summed here | Health Connect de-duplicates overlapping sources by the user's priority order; summing raw records would double count a phone and a strap. |
| Recovery marker is **HRV, else resting heart rate** | Section 1: HRV may not be written. |
| Baselines, impacts and energy balance are **computed in Rust** | Same reason as the weight trend: rules are tested without a browser and no view can disagree. |
| Measurements are **manual only**, one per kind per date | Health Connect has no record for waist, hips or arms. Height is a measurement kind so the waist-to-height ratio needs nothing else. |

## 5. Rejected and deferred

**Rejected.** A composite "readiness" score: it would be a fourth proprietary number next to
WHOOP's, Oura's and Garmin's, and it hides which input moved. Population HRV charts
(section 2.1). Sleep stage charts (section 2.2). Red and green status colours.

**Deferred** (see `docs/todo.md`): reading weight and body fat from a smart scale through
Health Connect; writing weigh-ins back to Health Connect; background sync.

## Sources

- [Health Connect integration for Android — WHOOP support](https://support.whoop.com/s/article/Google-Health-Integration-For-Android?language=en_US)
- [WHOOP introduces Health Connect integration — Gadgets & Wearables](https://gadgetsandwearables.com/2023/07/14/whoop-health-connect-integration/)
- [What is the average HRV? — WHOOP](https://www.whoop.com/us/en/thelocker/average-hrv-normal-heart-rate-variability/)
- [Day-to-day variability of WHOOP-derived HRV (PMC9505647)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9505647/)
- [HRV-guided endurance training: strengths and weaknesses — Frontiers, 2026](https://www.frontiersin.org/journals/sports-and-active-living/articles/10.3389/fspor.2026.1858271/full)
- [A brief history of HRV-guided training — Marco Altini](https://marcoaltini.substack.com/p/a-brief-history-of-heart-rate-variability)
- [Sleep Consistency: why we track it — WHOOP](https://www.whoop.com/us/en/thelocker/new-feature-sleep-consistency-why-we-track-it/)
- [Avoiding variability in sleep variability assessments — SLEEP](https://pmc.ncbi.nlm.nih.gov/articles/PMC13266552/)
- [Regularity of bedtime and wake-up time and cardiometabolic markers (PMC11960235)](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11960235/)
- [A new way to see which behaviours affect your recovery — WHOOP](https://www.whoop.com/us/en/thelocker/a-new-way-to-see-insights-on-which-behaviors-affect-your-recovery/)
- [WHOOP Journal overview](https://support.whoop.com/s/article/WHOOP-Journal-Overview?language=en_US)
- [MacroFactor's algorithms and core philosophy](https://macrofactor.com/macrofactors-algorithms-and-core-philosophy/)
- [How accurate is MacroFactor's expenditure algorithm?](https://macrofactor.com/algorithm-accuracy/)
- [NICE NG246: identifying and assessing central adiposity](https://www.nice.org.uk/guidance/ng246/chapter/Identifying-and-assessing-overweight-obesity-and-central-adiposity)
- [Waist-to-height ratio — Wikipedia](https://en.wikipedia.org/wiki/Waist-to-height_ratio)
- [Get started with Health Connect — Android Developers](https://developer.android.com/health-and-fitness/health-connect/get-started)
