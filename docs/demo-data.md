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
week and the weight trend reads "now", so once that date has passed the calendar starts
empty and the chart ends in the past: rebuild it before using it, and commit the rebuild
only when the data itself changed.

## How it is built
- Every write goes through `Mealprep`, so the file is what the app would have produced
  and every rule is applied to it. There is no SQL in the seeder.
- States that only arise over time are reached the way the user reaches them: an entity
  is created, used, and deleted at the end of the seed, so each kind of dangling
  reference has a miss to render.
- It is deterministic apart from ids and timestamps: two builds around the same day give
  the same figures, the same calendar and the same chart.
- It refuses to write over an existing file, because seeding twice would duplicate every
  entity. The task deletes the old file first.
- It seeds into a scratch file and copies out with `VACUUM INTO`, so the result is one
  file with no WAL sidecars.

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
| Over target (the hatch) | Training day A, and days it is on | Over on energy, protein and carbohydrates |
| Calendar, four weeks | Calendar | Two weeks back to three ahead planned; the current view and the one before both have days |
| Partly planned week | Calendar, last week of the view | Monday to Wednesday only |
| Unplanned days | Calendar | Every other Saturday, and the Wednesday of next week (assigned, then cleared) |
| Date whose plan was deleted | Calendar, tomorrow | "Old cutting plan", deleted; reads as unplanned |
| Weight trend and rate | Targets → Weight | Ten weeks ending today, losing about 0.4 kg a week |
| Skipped weigh-ins | Weight chart | Every few days, and ten days away with no scale |

The catalogue itself is not seeded: search, barcode lookup and the import review read Open
Food Facts live, so they need a network and not this file.

## Keeping it current
A feature that adds an entity, a field or a state worth seeing is not finished until the
seeder produces it and a row above says where. A schema change needs no change to the
seeder, which writes through the services, and an older file is migrated when the app
opens it; rebuild it anyway once the new feature is seeded, so the checked-in file shows it.
