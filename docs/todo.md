# Todo

Deferred decisions. Nothing here is implemented unless it is explicitly asked for. Finding an
entry relevant to the task at hand is not permission to act on it: the current behaviour
documented in `docs/requirements.md` stands until the entry is picked up deliberately.

Each entry records what happens today, what is deferred, and what would trigger the change.

## Frontend test runner
- **Today**: `bun test src` covers the pure modules under `src/lib/domain` — macro
  arithmetic, week dates, the calendar join, plausibility, sorting — with no config and no
  extra dependency.
- **Deferred**: Vitest, which resolves `$lib` and `$app` and can render components.
- **Trigger**: the first unit test that needs to import `$app/*` or mount a component.

## End-to-end tests
- **Today**: none. The predecessor ran Playwright against a live backend; there is no backend
  to run against, and driving the app means driving a Tauri window or an Android emulator.
- **Deferred**: WebDriver through `tauri-driver` on desktop, or an emulator-based run.
- **Trigger**: a regression that a `lib/domain` test could not have caught, or a release
  process that needs a smoke test before shipping an APK.

## Frontend formatter
- **Today**: none. The predecessor used Biome; nothing formats or lints the frontend here, and
  the standard is to match the file being edited.
- **Deferred**: adopting a formatter for `.svelte` and `.ts`.
- **Trigger**: formatting churn that costs review time, or a second person editing the
  frontend.

## Switching the store at runtime
- **Today**: `AppState` in `src-tauri/src/setup.rs` is a type alias, so the store is chosen at
  compile time and dispatched statically.
- **Deferred**: an enum store or boxed traits, so a build can carry both a local and a remote
  store and pick between them.
- **Trigger**: a second store existing at all.

## A cloud database and sync
- **Today**: one SQLite file on the device. Nothing synchronises, and the security section of
  the SRS rests on the data never leaving.
- **Deferred**: a crate implementing the `core::store` traits against Postgres, and a sync
  design for reconciling two stores.
- **Trigger**: wanting the same data on a second device.

## iOS
- **Today**: the barcode scanner is compiled for Android and iOS, and `src-tauri/gen` holds
  an Android project only. Nothing has been built or run for iOS.
- **Deferred**: `tauri ios init` and whatever the build needs.
- **Trigger**: access to a Mac and a reason to ship there.

## The unused radar and heatmap modules
- **Today**: `src/lib/domain/radar.ts` and `src/lib/domain/heatmap.ts` are ported from the
  predecessor, fully tested, and imported by nothing. The features they served — the day plan
  radar and the shaded macro columns — were dropped from the UI in the port.
- **Deferred**: either drawing them in the new mobile layout, or deleting both modules and
  their tests.
- **Trigger**: a screen that wants either visualisation, or a cleanup pass that would rather
  not carry dead code.

## A goal weight
- **Today**: weight is recorded and trended, with no goal and no opinion about the figure.
  The rate of change says which way it is going, which is what there is to act on.
- **Deferred**: storing a target weight and drawing it as a line on the chart. It fits the
  Targets concept, and it is one field, one command and one line on the plot.
- **Trigger**: wanting to see the distance to a goal rather than the direction of travel.
  Deliberately excluded from "simple" — see `docs/research/2026-09-20-weight-tracker-ui-ux.md`.

## Weight against energy intake
- **Today**: the calendar knows what was planned per day and the weight trend knows what
  happened, and nothing joins them.
- **Deferred**: a chart of the trend against the calorie balance that produced it, which is
  the question a food planner with a weight tracker in it can uniquely answer.
- **Trigger**: wanting to know whether the plan is working, rather than what the plan is.
  Tracked as TBD-14 in the SRS.

## Per-day cost on the calendar
- **Today**: cost is derived for meals and day plans. The calendar shows macros per day and
  per week, and no cost figure.
- **Deferred**: whether a day's or a week's cost is worth showing, and whether an incomplete
  cost can be summed across days without becoming meaningless.
- **Trigger**: wanting to read a week's spend off the calendar. Tracked as TBD-13 in the SRS.

## Getting the data off the device
- **Today**: the database is a private file in the app's Android storage with nothing that
  exports it. `mise run android:db:pull` copies it out, but only from a debug install, so a
  release-only device has no path to its own data. An uninstall — which a change of signing
  key forces — takes the database with it, as one did on 2026-09-20.
- **Deferred**: an in-app export, which would be the first feature to admit a second copy of
  the dataset and sits against `docs/requirements.md` "One user, one device, one dataset.
  Nothing synchronises."
- **Trigger**: wanting the data to survive the device, or a second device at all.

## Closed by the port
Deferred decisions the predecessor carried that this version answers.

| Was deferred | Answer here |
| --- | --- |
| A migration mechanism, because `schema.sql` was applied with `CREATE TABLE IF NOT EXISTS` and a new column needed a hand-run `ALTER TABLE` | `crates/sqlite/migrations`, embedded with `sqlx::migrate!` and applied when the store opens |
| How the built frontend ships — `go:embed` in the backend, or a static server beside it | Tauri serves the built SPA into its own webview; there is no server and no deployment question |
