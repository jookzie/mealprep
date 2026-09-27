# Demo data

`demo/mealprep-demo.db` is a small, believable dataset that puts every feature of the app
on screen: for trying the app, for screenshots, and for checking a UI change against every
state it has to render. It is built by `crates/sqlite/examples/seed_demo.rs`, and this
document is the map from each feature to the data that shows it.

## Using it

```sh
mise run demo:seed                          # rebuild it, around today
mise run demo:seed -- 2026-09-27            # rebuild it around a given day
mise run android:db:push -- demo/mealprep-demo.db   # load it into the debug install
```

On desktop, stop the app and copy it over `mealprep.db` in the app data directory
(`~/.local/share/com.mertan.mealprep/` on Linux). Either way it replaces the data that was
there; `android:db:push` saves that first, a desktop copy does not.

The checked-in file was built around **2026-09-27**. The calendar opens on the current
week, the weight trend reads "now" and every insight is read as of today, so once that date
has passed the calendar starts empty, the charts end in the past and the recovery baseline
falls out of its window: rebuild it before using it, and commit the rebuild only when the
data itself changed.

## How it is built
- Every write goes through `Mealprep`, so the file is what the app would have produced
  and every rule is applied to it. There is no SQL in the seeder.
- The health days are no exception: the seeder builds the raw sleep sessions, readings and
  activity a sync arrives with and hands them to `import_health`, so the stored rows are
  what the rules summarised rather than what the seeder thinks they should be.
- States that only arise over time are reached the way the user reaches them: an entity
  is created, used, and deleted at the end of the seed, so each kind of dangling
  reference has a miss to render.
- It is deterministic apart from ids and timestamps: two builds around the same day give
  the same figures, the same calendar and the same chart.
- It refuses to write over an existing file, because seeding twice would duplicate every
  entity. The task deletes the old file first.
- It seeds into a scratch file and copies out with `VACUUM INTO`, so the result is one
  file with no WAL sidecars.

## The shape of the history
Everything but the calendar reaches thirteen weeks back, and the calendar thirteen weeks
back and three ahead. That is not decoration: the energy balance reads the last three weeks
of plans, sleep regularity four weeks of nights, a recovery baseline eight weeks of
readings, and the behaviour comparisons thirteen weeks of both. A shorter history would
leave four screens saying there is not enough yet, which is one state each rather than the
state they are usually in.

One person's three months, and the parts of it agree: the days the calendar plans a
training day are the days the watch recorded a workout, the ten days away from home are
missing from the scale and the strap alike, and a waist coming down 1.1 cm in four weeks
goes with a weight trend 1.6 kg lower over the same four.

## What shows what

| Feature | Where to look | The data |
| --- | --- | --- |
| Products by hand, per 100 g | Products | Rolled oats, banana, chicken breast, … |
| Products per 100 ml | Products | Semi-skimmed milk, olive oil, orange juice |
| Every nutrient the checks read | Rolled oats → Edit | kJ, fibre, saturates, sugars, salt, all consistent |
| Imported products and their code | Greek yoghurt 0% (Demo Dairy), whey, chocolate, orange juice | Codes `2000000000017`–`…048` |
| Brand as identity | Products | Two "Greek yoghurt 0%", Demo Dairy (imported) and Hillside Farm (by hand) |
| Plausibility: part above its whole | Dark chocolate 85% → Edit | 25 g sugars in 19 g carbohydrates |
| Plausibility: kJ against kcal | Orange juice → Edit | 450 kJ against 45 kcal |
| Unpriced product, cost as a floor | Peanut butter, and every meal and plan with it | No cost entered |
| Complete cost | Rest day | Every product priced, nothing removed |
| Meal categories, with a rename | Meals → Categories | Breakfast, Lunch, Dinner (created as "Diner"), Snack, Weekend |
| Day-plan categories | Day plans → Categories | Training day, Rest day, Weekend |
| One name in both scopes | Both category lists | Weekend |
| Uncategorised | Leftovers, Travel day | No category |
| Meal in a removed category | Sweet potato soup | "Seasonal", deleted |
| Serving of a removed product | Protein bar and apple | Protein bar, deleted |
| Day plan: meals and loose products in one order | Training day A | Four meals, a banana and chocolate between and after them |
| Day plan: loose products only | Travel day | Six products, no meals |
| Deleted meal drops out of a plan | Lazy Sunday | "Old smoothie" was its second item |
| Deleted product stays as removed | Lazy Sunday | Protein bar, 60 g |
| Targets and the meters | Targets, and any plan | 2000 kcal, 65 g fat, 140 g protein, 220 g carbohydrates |
| Over target (the hatch) | Training day A, and days it is on | 2112 kcal: over on energy, protein and carbohydrates |
| Over on one meter only | Rest day | 1846 kcal and 143 g protein: at the protein target, under the energy one |
| Calendar, four weeks | Calendar | Thirteen weeks back to three ahead; every view from late June on has days |
| Partly planned week | Calendar, last week of the view | Monday to Wednesday only |
| Unplanned days | Calendar | Every other Saturday, and the Wednesday of next week (assigned, then cleared) |
| Date whose plan was deleted | Calendar, tomorrow | "Old cutting plan", deleted; reads as unplanned |
| Weight trend and rate | Targets → Weight | Thirteen weeks ending today, losing about 0.4 kg a week |
| Skipped weigh-ins | Weight chart | Every few days, and the ten days away with no scale |
| The Body hub | Body | One line on each card: recovery, sleep, weight, measurements, energy balance, targets |
| Measurements of several kinds | Measurements | Waist weekly, body fat fortnightly, hips, chest and thigh every four weeks |
| A kind measured once | Measurements → Height, Neck | 178 cm in July, a 39.6 cm neck in August: too short a series to compare |
| A kind never measured | Measurements, the picker | Arm |
| Waist-to-height, in the NICE bands | Measurements | 86.7 cm over 178 cm: 0.49, read as healthy |
| Waist against the weight trend | Measurements, last 4 weeks | −1.1 cm against −1.6 kg of trend: "both coming down" |
| A tape that does not fall in a line | Measurements → Waist | Four weeks that read higher than the week before, inside a 4.4 cm fall |
| Imported health days | Body, the sync line | 81 mornings from 29 June, synced at 07:10 on the reference day |
| Sleep, with stages | Sleep | Light, deep, REM and time awake, from the strap |
| A night the source gave bounds for only | Sleep, the first three weeks | Recorded by a phone: no stages, and all of it counts as sleep |
| Nights missing from the chart | Sleep | The ten days away, and two nights the strap was flat |
| Sleep duration and regularity | Sleep | 6 h 45 m a night over the last week; bedtime 23:27 ±30 min, waking 06:34 ±53 min |
| HRV and resting heart rate | Recovery | 58 mornings of readings, the 7-day average drawn over its band |
| A week outside the normal range | Recovery | A hard last week: HRV 56 ms below its 59–64 band, resting heart rate 54 bpm above its 51–53 |
| Three days ill | Recovery, three weeks back | The spike in both series, and in the respiratory rate behind them |
| A baseline that is not there yet | Recovery, the left of either chart | No band until fourteen readings are in |
| An association that is clear | Recovery → "Slept 7 hours or more" | HRV 7 % higher; after a workout the day before, 9 % lower |
| An association that is not | Recovery → "Went to bed earlier" | −0.4 %, well inside half a standard deviation |
| Two habits that are not the same days | Recovery → the two plan habits | Over the energy target is Monday and Friday; at the protein target is those and the rest days |
| Energy balance | Energy balance | 1738 kcal planned against 2166 burned, from a 1.2 kg fall in the trend |
| The wearable's own figure | Energy balance | 2405 kcal a day, 239 above what the trend implies |
| A window not completely covered | Energy balance | 19 of 21 days planned, 20 of 21 reported by the wearable |
| Every health figure optional | Recovery, Sleep, Energy balance | Three weeks with nights but no markers, two days with markers but no activity |

The catalogue itself is not seeded: search, barcode lookup and the import review read Open
Food Facts live, so they need a network and not this file.

## Health Connect and this file
The imported days are in the database, but the Body screen decides what to show from the
device, not from them:

- On desktop Health Connect reports itself unsupported, so the Body hub hides the recovery
  and sleep cards. Both screens still render from the seeded days at `/body/recovery` and
  `/body/sleep`.
- On an Android debug install with Health Connect connected, opening Body re-syncs when the
  last import is an hour old — which this file's always is — and **each import replaces the
  days it covers**, so the seeded three months are replaced by whatever that device holds.
  To keep them, look at the screens without granting access, or push the file again after.

Nothing in the seeder can avoid either: availability and permission are the platform's
answers, and `HC-4` is the behaviour the app is meant to have.

## Keeping it current
A feature that adds an entity, a field or a state worth seeing is not finished until the
seeder produces it and a row above says where. A schema change needs no change to the
seeder, which writes through the services, and an older file is migrated when the app
opens it; rebuild it anyway once the new feature is seeded, so the checked-in file shows it.

An insight is a feature with a second requirement: it has to be given enough history to
have an answer, and the figures above have to be checked after a seed rather than assumed.
A threshold that is not reached shows the same empty state as a feature that was never
seeded at all.
