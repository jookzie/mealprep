# UI/UX Research → Weight Tracker

> **Status: research deliverable, 2026-09-20.** The decisions in Part 5 were taken and are
> implemented; the rest records why, and what was rejected. Part 6 is deferred work, not a
> plan.

## Context

The application plans food against macro targets. Weight is the outcome those targets exist
to move, and it was the one figure the user could not record. The brief was "a simple weight
tracker with a graph" — so the question is not what a weight tracker could contain, but which
of its parts earn their place in an app that already has five tabs and a settled visual
language.

## 1. The graph is a trend line, not a plot of the scale

This is the finding that shaped everything else, and it inverts the naive design.

Day-to-day body weight is dominated by water retention, gut contents, sleep, stress and
hormonal cycling. A chart of raw weigh-ins is therefore mostly noise: the visible movement
between any two adjacent days is usually not fat, and reading it as progress is how a tracker
makes its user miserable. The standard answer, from the Hacker's Diet onwards, is an
exponentially smoothed moving average:

```
trend += (weight − trend) × 0.1
```

A tenth lags like a twenty-day simple moving average, but it needs no window and discards no
history — older measurements fade out on their own. MacroFactor's trend is the same idea with
recency weighting, and their guidance is that daily weigh-ins are ideal but three a week is
enough.

Two consequences for the UI:

- **Both series are drawn together.** The trend alone hides the data; the measurements alone
  hide the signal. Seeing a point sit below the line is what tells the user the trend is about
  to turn, and it is the psychological payoff the smoothing would otherwise take away.
- **The trend is the headline figure, not the last weigh-in.** The number shown large is where
  the trend stands, because that is the one that means something.

## 2. Skipped days must be interpolated, not ignored

If the smoothing runs over weigh-ins as a list, three weeks away from the scale smooths as
though it were one day, and the trend jumps on the user's return. MacroFactor fills gaps by
linear interpolation between the surrounding measurements; the series is then daily and the
smoothing is time-correct.

The interpolated days carry no measurement, so they get no mark on the chart. The user sees a
continuous trend line with a gap in the points, which is an honest picture of what happened.

## 3. The y-axis is truncated, deliberately and with a guard

The zero-baseline rule is ironclad for bars and does not transfer to lines. Weight is the case
truncation exists for: an 80 kg person's meaningful range is a few kilograms, and a chart from
zero renders every change as a flat line.

The counter-evidence is real too — the higher a truncated axis starts, the more dramatic
readers judge the same trend to be. The guard adopted here is the practical one from the
visualisation literature: **choose the range so no data falls in the bottom third of the
plot.** Implemented as padding of half the data's own span below and 15% above, with a
one-kilogram minimum window so a flat week does not scale its own rounding noise to full
height.

## 4. Logging is the loop that has to be cheap

Every well-reviewed tracker optimises one path: record today's weight. The patterns that
recur are a single prominent input defaulted to today, one decimal place, and replacing rather
than appending when a date already has a figure.

Rejected: a weight picker wheel (precise input is faster to type on a phone keyboard with a
decimal pad), and any streak or congratulation mechanic (the app informs and never enforces —
`TG-3` — and a tracker that praises a number has an opinion about the body on the scale).

## 5. Decisions taken

| Decision | Why |
| --- | --- |
| Weight lives inside Targets, at `/weight` | The bottom bar already holds five tabs, which is the maximum both the iOS HIG and Material Design set. A sixth shrinks every target. Targets is where the figures a plan is measured against already live. |
| Entries, trend graph, rate of change | The minimum that is useful. The rate (kg/week) is the number people act on, and it is nearly free once the trend exists. |
| Kilograms only | The app is metric throughout — grams, millilitres, kcal. A unit toggle means a stored preference and a conversion at every display site, for no gain to a single metric user. |
| One entry per date, replacing | A day has one number worth keeping. Keyed by date like `calendar_days`, so logging twice corrects rather than accumulates. |
| Trend computed in Rust | It is a domain rule, not a drawing concern, so it is tested without a browser and cannot drift between views. The frontend receives the daily series and draws it. |
| Hand-rolled SVG | The app has no chart library and should not gain one: a dependency for a single 180px chart, in an offline mobile app, is a poor trade. The geometry lives in `lib/domain/weight.ts` as pure functions, and is unit-tested. |
| Ink, not a macro hue | Weight is not a macro, and the accent means "interactive" everywhere else in the app. The trend takes the primary ink, the measurements the muted one — which also makes the chart theme-correct for free. |

## 6. Rejected and deferred

**Rejected outright.** BMI: it needs a stored height and is a contested metric to put in front
of someone daily. Streaks, badges and congratulation. A goal-weight countdown, which turns a
measurement into a verdict.

**Deferred** (see `docs/todo.md`):

- **A goal weight and a goal line on the chart.** The natural next increment, and it fits the
  Targets concept. Left out of "simple" deliberately; it is one field, one command and one
  line on the plot when it is wanted.
- **Weight against energy intake.** The app knows what was planned and what was eaten; the
  interesting chart is the trend against the calorie balance that produced it. This is a real
  feature, not a tweak, and it needs the calendar's totals per day.
- **Export.** A weight history is the kind of data people want out of an app.

## Sources

- [Signal and Noise — The Hacker's Diet](https://www.fourmilab.ch/hackdiet/e4/signalnoise.html)
- [Weight Trend — MacroFactor](https://help.macrofactorapp.com/en/articles/21-weight-trend)
- [Tab Bars — Apple Human Interface Guidelines](https://miniring.gitbook.io/hig/bars/tab-bars)
- [Bottom navigation — Material Design](https://m2.material.io/components/bottom-navigation/ios)
- [Truncating the Y-Axis: Threat or Menace? (Correll et al.)](https://arxiv.org/pdf/1907.02035)
- [It's OK not to start your y-axis at zero — Quartz](https://qz.com/418083/its-ok-not-to-start-your-y-axis-at-zero)
- [Case study: a weight tracking app concept](https://medium.com/design-bootcamp/case-study-design-a-concept-of-weight-tracking-app-in-a-day-620f65778268)
