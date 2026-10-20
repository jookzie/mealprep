# Architecture

## Overview
```
SvelteKit SPA (webview) ──invoke──▶ src-tauri commands ──▶ mealprep-core::Mealprep
                                                           │        │        │
                                              SqliteStore ─┘        │        └─ Anthropic
                                               (sqlx)               │            (reqwest)
                                                  │        OpenFoodFacts            │
                                                  │           (reqwest)             │
                                             mealprep.db          │        Anthropic Messages API
                                                         Open Food Facts API   (user's own key)
```
Everything runs in one process on the device. The frontend builds to static files that
Tauri serves; the Rust side owns all state. The two outward arrows are HTTPS clients and
nothing else: the application listens on no port. The right-hand one carries plan context
off the device and exists only once the user has stored an API key — see §5.3 of the SRS.

The predecessor split this across a browser, a Go HTTP backend and an OpenAPI contract that
generated both sides. That contract is gone: there is no wire format to agree on when both
sides are the same process, so the domain types serialise straight across the IPC bridge and
`src/lib/api/types.ts` mirrors them by hand.

## Repository layout
```
crates/core            domain types, business rules, storage traits (no I/O)
crates/sqlite          SqliteStore and the schema migrations
crates/openfoodfacts   the Catalogue trait over the public API
crates/anthropic       the Assistant trait over the Claude Messages API
src-tauri              Tauri commands, error serialisation, setup
src                    the SvelteKit static SPA the webview loads
docs/                  this document, the SRS, the standards, the deferred list
mise.toml              the tasks: format, lint, test, dev, android, autocommit
```

## Crates

| Crate | Role | Depends on |
| --- | --- | --- |
| `mealprep-core` | Domain types, validation, derivation of macros and cost, storage traits | serde, time, uuid |
| `mealprep-sqlite` | `SqliteStore`: every storage trait over one SQLite file | core, sqlx |
| `mealprep-openfoodfacts` | `OpenFoodFacts`: the `Catalogue` trait over the public API | core, reqwest |
| `mealprep-anthropic` | `Anthropic`: the `Assistant` trait over the Claude Messages API, with one tool per kind of operation | core, reqwest |
| `mealprep-rs` (`src-tauri`) | Commands, error serialisation, opening the store in the app data dir | all four, tauri |

`core` has no I/O and no knowledge of SQLite, HTTP, Claude or Tauri. An adapter depends on
`core` and never on another adapter.

## Layering
`command → service → store`. Lower layers never depend on higher ones, and the direction is
enforced by the crate graph rather than by convention: `mealprep-core` does not depend on
`mealprep-sqlite` or on Tauri, so a rule cannot reach a database or a window even by mistake.
`core::store` declares what the rules need; `crates/sqlite`, `crates/openfoodfacts` and
`crates/anthropic` provide it; `src-tauri` chooses the implementations and exposes the
result.

This replaces the predecessor's `handler → service → repository → client`, and holds the same
way for the same reason.

## Command surface
`src-tauri/src/command/` splits by domain — `product.rs`, `meal.rs`, `day_plan.rs`,
`calendar.rs`, `category.rs`, `targets.rs`, `weight.rs`, `assistant.rs` — and each function
is a thin `#[tauri::command]` that takes `State<'_, AppState>`, calls one service method and
returns its value. There is no
DTO layer: the domain types are the wire format, so a command that compiles carries the shape
the frontend declares.

Commands are the whole public surface. Anything the frontend needs is a command; anything
else stays in Rust.

## Frontend layout
```
src/
  app.css              the tokens: macro hues, spacing, the light and dark palettes
  routes/              one directory per screen: +page.ts reads, +page.svelte renders
  lib/
    api/               the only way the app reaches Rust
      index.ts         one function per command
      types.ts         hand-written mirror of the Rust domain types
      errors.ts        errorKind, messageOf, read
      mutate.ts        runMutation
    domain/            pure logic: macro arithmetic, week dates, the calendar join,
                       plausibility, sorting, formatting, the assistant's preview
                       summaries and reversal hints
    components/        forms, pickers, meters, the weight chart, the scan overlay,
                       the page header
```
Routes own all I/O. Components take data and callbacks, and never call `invoke`.

The shell is a bottom tab bar in `+layout.svelte`, sized around
`env(safe-area-inset-bottom)` and hidden while a scan is running. There is no sidebar and no
command palette: the primary device is a phone.

## Frontend data flow
```
+page.ts load ──▶ $lib/api ──invoke──▶ commands
      ▲                                    │
      └────── invalidateAll() ◀── runMutation
```
Reads are `load` functions; `read` turns a failure into the error page. Writes go through
`runMutation`, which toasts, re-runs every load and redirects with `replaceState`, so going
back never returns to a submitted form. It never throws, and answers `undefined` when the
call failed. There is no second caching layer.

`$lib/api` refuses to work outside the webview — `isTauri()` is checked once, in `call` — and
says so, because the data lives on the device and a browser tab has none of it.

Editors load the entity first, because every write is a full replacement.

## Key decisions
- **sqlx for persistence**: it is the async SQL crate on blessed.rs that speaks both SQLite and
  PostgreSQL, so a future cloud database is another store in the same idiom. Queries are
  plain SQL with runtime binding (no `DATABASE_URL` needed at build time); rows map to
  domain types by hand in `crates/sqlite`.
- **Migrations are embedded**: `crates/sqlite/migrations` is compiled in with
  `sqlx::migrate!` and applied on open, which replaces the original's hand-run `ALTER TABLE`s
  and closes the predecessor's deferred schema-change question.
- **One trait per aggregate, one facade**: `core::store` declares `ProductStore`, `MealStore`,
  `CategoryStore`, `DayPlanStore`, `CalendarStore`, `TargetsStore`, `WeightStore`,
  `AssistantStore`, `Catalogue` and `Assistant`.
  `service::Mealprep<S, C, A>` holds a store, a catalogue and an assistant, and each `impl`
  block asks only for the traits it uses. Traits are `#[trait_variant::make(Send)]` so futures are `Send` for
  Tauri's async commands. This is the predecessor's consumer-defined interfaces, expressed as
  traits rather than as Go interfaces declared at the call site.
- **Static dispatch**: the application picks its store with one type alias,
  `src-tauri/src/setup.rs::AppState`. A cloud store implements the same traits; switching
  at runtime would mean an enum store or boxing, which is deferred until it exists.
- **Domain types are the wire format**: serde derives in `core::domain` use camelCase and
  omit absent optionals, so `src/lib/api/types.ts` mirrors them one to one. Composed reads use
  `Derived<T>` (entity fields plus `macros` and `cost`) and `ResolvedDayPlan`.
- **Errors**: `core::Error` is a `thiserror` enum — `NotFound`, `Invalid`, `Conflict`,
  `CatalogueUnavailable`, `AssistantUnavailable`, `Internal` — each carrying the `Entity` it
  is about. Adapters
  convert their library errors into it at the boundary; `src-tauri` serialises it as
  `{ kind, message }` with `kind` in kebab-case, so the frontend reacts on the kind and
  never parses the message.
- **Rules carried over from the original**: soft deletes everywhere; macros and cost derived
  on read, never stored; a product without a price makes a cost incomplete rather than
  zero-and-silent; a label is unique within its category ignoring case, with uncategorised
  entities as one scope; a category's scope is fixed at creation; deleted products stay in
  a day plan as removed entries while deleted meals drop out; imports are snapshots.
- **A day plan holds meals and loose products in one order**: `DayPlanItem` is an enum over
  the two, tagged `kind` on the wire, because a day is eaten in one sequence and a yoghurt
  eaten on its own is not worth promoting to a meal.
- **Imports are reviewed, not trusted**: Open Food Facts is crowd-sourced, so
  `get_catalogue_entry` fetches an entry for `/products/import/[code]`, which prefills the
  product form, and `import_product` stores the draft the user confirmed, with the code as
  provenance. The entry carries `source` (manufacturer when `data_sources_tags` names the
  producers' platform or a GS1 data pool), the catalogue's own figure-related quality
  tags as `warnings`, `modifiedAt`, and `missingMacros`, which seed the form empty rather
  than zero.
- **Plausibility checks live in the frontend** (`src/lib/domain/plausibility.ts`): they
  re-run on every keystroke, which a command round trip would not suit, and they only
  inform, so the Rust rules have nothing to enforce.
- **Barcode scanning** uses `tauri-plugin-barcode-scanner` (CameraX and ML Kit through
  Google Play services), compiled for Android and iOS only and granted by
  `src-tauri/capabilities/mobile.json`. The scan runs windowed under a transparent page:
  the plugin draws no controls, so `ScanOverlay` supplies the frame and Cancel, and
  cancels on back navigation. Desktop has no scanner; a barcode typed into search goes
  straight to review.
- **The weight trend is smoothed in Rust, not drawn from the raw weigh-ins**: day-to-day
  weight is mostly water and gut contents, so `service::weight` runs the Hacker's Diet's
  exponentially smoothed average — a tenth of each new measurement, lagging like a twenty-day
  window without needing one. Skipped days are linearly interpolated first, so a fortnight
  away from the scale does not smooth as though it were one day. The trend is derived on read
  and never stored, the way macros and cost are, and it lives in the rules rather than in the
  chart so that no view can disagree about it.
- **The weight chart is hand-rolled SVG**: one 180px plot does not justify a charting
  dependency in an offline mobile app. The geometry is pure functions in
  `src/lib/domain/weight.ts` — domain, scale, path, ticks — so it is unit-tested without a
  browser, and the component only draws. The y-axis is truncated, because a weight chart from
  zero flattens every change worth seeing, with padding below the data so ordinary
  fluctuation cannot read as a collapse. Weight takes the ink colours rather than a macro
  hue: it is not a macro, and the accent means "interactive" everywhere else.
- **Bulk calendar assignment is a sequential loop**: SQLite has one writer.
- **Open Food Facts over rustls with webpki roots**: no OpenSSL or platform verifier to
  cross-compile or initialise on Android.
- **Macro colour is identity, not status**: `app.css` fixes one hue per macro and the app
  never introduces a second colour to mean "over target" — over-target is the same hue under
  a hatch. Energy is the sum of the other three, so it stays neutral rather than becoming a
  fourth series. The palette contains no red, because red means broken.
- **A meter, not a progress bar**: planned-against-target is a scalar within a known range
  that can legitimately exceed its maximum, which is a meter; a progress bar is monotonic
  task completion.
- **Ordering is persisted and editable with buttons, not drag**: WCAG 2.2 SC 2.5.7 makes a
  non-drag path the required baseline.
- **Dangling references are a state, not an error**: entities are soft-deleted and then never
  returned, so every join can miss. A serving whose product is gone renders as removed while
  keeping the id, so the user can still fix it.
- **Claude rather than a model on the device**: the assistant has to resolve a sentence onto
  several typed edits over real ids, dates and amounts, and get all of them right or be
  refused. A model small enough to run on a phone is not reliable at that, and the failure
  mode — a plausible edit naming the wrong product — is the expensive one. So the request
  goes to `claude-sonnet-5` over HTTPS. The cost of that choice is that plan context leaves
  the device, which is why the feature is off until the user supplies their own API key and
  why `docs/requirements.md` §5.3 had to be rewritten. The application has no backend: the
  key is the user's, stored on the device, and the only place it goes is the `x-api-key`
  header of that one request. A local-model adapter would be another crate implementing
  `store::Assistant` and nothing above it would change — see `docs/todo.md`.
- **The model proposes and never writes**: it answers with a `Vec<Operation>` from a closed,
  typed set plus a sentence of rationale. `service::assistant` checks every operation against
  the real data — unknown product, impossible amount, out-of-range date are all
  `Error::Invalid` and never a silent fix — builds a preview, and writes only the operations
  the user confirmed, re-checked from scratch. Both halves matter: a typed operation set means
  the model cannot express an edit the application has no rule for, and the confirmation means
  a wrong answer costs a glance rather than a database. One tool per kind of operation, with
  the schema table in `crates/anthropic` written as a match on `OperationKind`, so a new
  variant stops the workspace compiling until it has a tool rather than going silently
  unproposed.
- **Copy-on-write per date**: a `calendar_days` row points at a *shared* `day_plans` row, so
  editing "the next four days" in place would rewrite every past date on that plan as well —
  the history would move, and nothing would say it had. So an operation bounded to dates
  detaches each date onto its own copy of its plan (`day_plans.source_plan_id`), then edits
  the copy; the original and every other date on it are untouched. A copy belongs to one date:
  it is excluded from `list_day_plans` and from label uniqueness, so no list and no picker can
  reach it, and a second edit to the same date finds the copy and edits it in place rather than
  copying the copy. The calendar therefore needs `list_calendar_day_plans` to resolve a
  detached date, because the plan list no longer names it. On top of the copies, the service
  refuses any date before today unless the request names a past day in so many words
  (`service::validate::past_dates_requested`), and every applied request is recorded with what
  each date pointed at before.

## Domain model
- `Product` — id, name, unit, the four macros, every further nutrient per 100 units, optional cost per 100 units, optional brand, optional `sourceCode`. Brand is identity, not presentation: two imports often share a name and differ only there.
- `Meal` — label, optional category, ordered list of (product, serving amount in the product's unit).
- `DayPlan` — label, optional category, ordered list of `DayPlanItem`, each either a meal or a product with an amount.
- `Category` — name and a scope (meals or day plans) fixed at creation.
- `CalendarDay` — date → day plan, at most one per date.
- `Targets` — energy (kcal), fat, protein, carbs. Exactly one row.
- `Operation` — one typed edit the assistant may propose, each bounded to a `DateRange`: adjust a loose product's amount, add an item, remove an item, swap a meal, assign a plan, clear a plan. `OperationKind` is the same set without the arguments, so an adapter derives its tools from it.
- `Proposal` — a request, the model's rationale, and its operations. Unchecked. `Preview` is a checked proposal: one `PreviewDay` per date that would change, carrying the change lines, the day before and after, the deviation from the targets, and whether applying detaches that date.
- `AssistantRecord` — one applied request: the text, the rationale, the operations, and an `AppliedChange` per date with what it pointed at before. That last field is the undo: re-assign it, or clear the date when there was none.
- `AssistantKeyStatus` — whether a key is stored and its last four characters. The key itself never crosses the IPC bridge outward.
- `WeightEntry` — date → kilograms, at most one per date. `WeightSeries` is the derived read: one point per day from the first weigh-in to the last, each carrying the trend and the measurement where there was one, plus the trend's current value and its rate of change per week.

A `DayPlan` also carries `sourcePlanId`, set when it is one date's own copy of another plan.
Domain types carry `createdAt` and `updatedAt`; `deleted_at` is a column the store filters on
and never surfaces. Energy is kcal, mass is grams, volume is millilitres. Macros and cost on a
meal or day plan are derived on read and never stored.

## Data access
`crates/sqlite/migrations/` is the schema — `0001_initial.sql` for `products`, `categories`,
`meals`, `meal_servings`, `day_plans`, `day_plan_items`, `calendar_days` and `targets`,
`0002_weight_entries.sql` for `weight_entries`, and `0003_assistant.sql` for
`day_plans.source_plan_id`, `app_settings` and `assistant_requests` — embedded with
`sqlx::migrate!` and applied when the store opens. Queries are hand-written SQL in
`crates/sqlite`; rows map to domain types there, and nothing above that crate sees a row type.

The database is `mealprep.db` in the platform's app data directory. On Android that is the
app's private storage, which is the app root rather than `files/`.

## Tooling
`mise.toml` holds the tasks. `cargo fmt` and `cargo clippy` cover Rust, `svelte-check` is the
frontend's contract check, `cargo test` covers the domain rules and the SQLite integration,
and `bun test src` covers the pure frontend modules. `mise run autocommit` drafts a commit
message from the staged diff and opens it for review.

`mise run android:build:release` builds the signed arm64 APK. Gradle takes the keystore from
`src-tauri/gen/android/keystore.properties`, which is untracked and holds `storeFile`,
`storePassword`, `keyAlias` and `keyPassword`; the task refuses to run without it, because a
release build with no signing config succeeds and yields an APK that cannot be installed.
The debug tasks sign with the default debug key in `~/.android` instead.

Debug builds carry the `.debug` application id suffix
(`tauri.conf.json > bundle > android > debugApplicationIdSuffix`), so a debug and a release
install coexist with separate data directories. Without it the two share one id, they cannot
be installed at once, and every switch between them costs an uninstall — which deletes the
database, since the app's storage is private and nothing exports it.

`mise run android:db:pull` copies the database out of the debug install into `backups/`,
sidecars included, and `mise run android:db:push -- <file.db>` replaces it, snapshotting what
was there first. Push validates the file and its `_sqlx_migrations` row before touching the
device, folds any WAL into the main file, and stops the app first, because overwriting a
database a running process holds open is how a WAL and its main file stop agreeing.

Both reach the debug install only — `run-as` refuses a release build, and the suffix gives
the two separate databases. The release install's data has no route off the device.

`mise run demo:seed` rebuilds `demo/mealprep-demo.db` through
`crates/sqlite/examples/seed_demo.rs`, writing through the services so the rules apply to it;
`docs/demo-data.md` says which data shows which feature.

## Future: another assistant
Add a crate implementing `core::store::Assistant` — a model on the device, or a different
hosted one — and change the `AppState` alias. `core` does not know Anthropic exists, so
nothing in the rules, the commands or the frontend changes, and the operations, the preview
and the copy-on-write behaviour are the same whoever proposes them.

## Future: a cloud database
Add a crate (e.g. `crates/postgres`) implementing the `core::store` traits with sqlx's
Postgres driver, then change `AppState`. Nothing in `core` or the frontend changes.
Sync between a local and a remote store is not designed yet.
