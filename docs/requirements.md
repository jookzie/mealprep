# Software Requirements Specification — Mealprep

Structured after [IEEE Std 830-1998](http://www.cse.msu.edu/~cse870/IEEEXplore-SRS-template.pdf)
and the [markdown SRS template](https://github.com/antonmry/markdown-templates/blob/master/requirements.md).

This document supersedes the SRS of the Go/SvelteKit `mealprep` application. Requirement
identifiers are carried over unchanged wherever the requirement itself survived the port, so
`PR-8` means here what it meant there; identifiers retired by the port are listed in
Appendix D rather than reused.

## 1. Introduction

### 1.1 Purpose
Mealprep helps one person plan a week of meals against daily macro targets — calories, fat,
protein and carbohydrates — and see what the plan costs.

### 1.2 Scope
Mealprep is a single application that runs entirely on the user's device: a webview frontend,
the business rules and the database in one process, with no server and no network service.
It covers configuring products, composing meals from them, grouping meals and products into
day plans, assigning day plans to calendar days, recording body weight and measurements,
reading recovery and sleep from Health Connect, and reporting nutrients, cost and energy
balance against targets. It does not cover shopping lists, recipes with
instructions, multiple users or sharing.

Cost tracking was excluded by the predecessor and is now in scope (§4.6). Categories for
meals and day plans are new (§4.5), as are weight tracking (§4.10), body measurements
(§4.12), the Health Connect import (§4.13) and the insights drawn from them (§4.14).

### 1.3 Document conventions
Requirements are identified as `XX-n` (`PR-1`, `ML-2`) and are binding. Priority is stated
per feature, not per requirement. *To be determined* in italics marks an open item; every
one is listed with its blocking condition in Appendix C.

### 1.4 Intended audience
The developers building and maintaining this repository. Read Section 2 for context,
Section 4 for what to build, Sections 3 and 5 for the constraints around it.

### 1.5 Definitions and abbreviations
- **Product** — a food item with its nutrients per 100 g, or per 100 ml when it is a liquid. Imported from Open Food Facts or created by the user.
- **Meal** — a labeled set of products, each with a serving size in grams or millilitres.
- **DayPlan** — a labeled, ordered list of items, where an item is a whole meal or a product eaten on its own.
- **Category** — an optional grouping a meal or a day plan belongs to. A category belongs to one scope and cannot move.
- **CalendarDay** — a date with a day plan assigned to it.
- **Targets** — the user's daily figures for energy (kcal), fat, protein and carbohydrates.
- **Macro** — one of those four figures.
- **Cost** — what a composition costs, derived from its products' prices, carrying whether every product in it had one.
- **Weigh-in** — one body weight recorded against a date, in kilograms. A date holds at most one.
- **Trend** — the exponentially smoothed average of the weigh-ins, which is the figure to read rather than the last measurement.
- **OFF** — [Open Food Facts](https://world.openfoodfacts.org/), the external product database.
- **Catalogue** — OFF as the application sees it, behind the `Catalogue` trait.
- **SRS** — this document.

### 1.6 References
- [Open Food Facts](https://world.openfoodfacts.org/) — external product database.
- `docs/architecture.md` — system structure and key decisions.
- `docs/code-standards.md` — binding implementation constraints.
- `docs/todo.md` — deferred decisions, not to be implemented unless asked.

## 2. Overall description

### 2.1 Product perspective
The successor to the Go/SvelteKit `mealprep` application, which ran a browser frontend
against a self-hosted HTTP backend. This version collapses that into one process on the
device: the webview invokes Tauri commands directly, and the HTTP API and its OpenAPI
contract are gone. The domain rules, the entities and the product catalogue behaviour are
carried over. See `docs/architecture.md` for the component diagram.

### 2.2 Product functions
- Configure products, by importing from OFF — by name search or by scanning a barcode — or creating them by hand.
- Compose meals from products with serving sizes.
- Group meals and loose products into day plans, in one order.
- Organise meals and day plans into categories.
- Assign day plans to calendar days.
- Set daily macro targets and see planned nutrients and cost against them.
- Record body weight and read its smoothed trend and rate of change.
- Record body measurements and read the waist-to-height ratio and four-week changes.
- Import sleep, recovery and activity from Health Connect, and read them against the user's own baseline, the plan and the weight trend.

### 2.3 User classes and characteristics
One class, the single user, who both configures products and plans meals. No roles, no
administrator, no authentication. The user is assumed to understand macros well enough to
set their own targets.

### 2.4 Operating environment
- Application: Rust (edition 2024, `rust-version` 1.94) in a Tauri v2 shell. One process, no server.
- Storage: a SQLite database in the platform's app data directory, reached through `sqlx`.
- Frontend: SvelteKit + TypeScript (Svelte 5 runes), built with Bun into a static single-page app that Tauri serves into its webview.
- Android is the primary target; the desktop build runs the same code. The page only works inside the Tauri webview and says so when opened in a plain browser.

### 2.5 Design and implementation constraints
- `docs/code-standards.md` governs all implementation and overrides nothing in this document.
- The command surface in `src-tauri/src/command/` is the only path from the frontend to the rules.
- Persistence sits behind the `core::store` traits so the database stays swappable.
- `mealprep-core` performs no I/O and must not learn about SQLite, HTTP or Tauri.
- No authentication and no user accounts while the product is single-user and on-device.

### 2.6 User documentation
*To be determined* (TBD-3).

### 2.7 Assumptions and dependencies
- The OFF API is reachable and free to use under its terms; the application degrades to hand-created products when it is not.
- An OFF entry missing the four macros seeds the import form empty rather than zero, and the user completes it (`IM-4`).
- One user, one device, one dataset. Nothing synchronises.

## 3. External interface requirements

### 3.1 User interfaces
A webview UI built from plain CSS with no component library, laid out mobile-first around a
persistent bottom tab bar. The predecessor's sidebar, command palette and keyboard shortcuts
are gone: the primary device is a phone, where neither a sidebar nor `⌘K` has a place.

Nutrients are shown as the four macros, with the full nutrient set behind an expansion
(`PR-9`), ordered as Regulation (EU) 1169/2011 Annex XV presents a nutrition declaration.

| Screen | Path | Serves |
| --- | --- | --- |
| Product list | `/products` | `PR-10`, `PR-11`, `CO-3` — filter by name or brand, sort by name or any macro |
| New product | `/products/new` | `PR-3`, `PR-7`, `CO-1`, `PL-1` |
| Catalogue search | `/products/search` | `PR-1`, `PR-2`, `PR-8`, `SC-3` |
| Import review | `/products/import/[code]` | `IM-1`–`IM-4`, `SC-2` — where a scan and a search result both land |
| Product detail | `/products/[productId]` | `PR-9`, `CO-3`, §6 — macros, nutrient expansion, source credit and link |
| Edit product | `/products/[productId]/edit` | `PR-10`, `PL-1` |
| Meal list | `/meals` | `ML-5`, `CT-3` |
| Meal categories | `/meals/categories` | `CT-1`, `CT-2`, `CT-5` |
| New meal | `/meals/new` | `ML-1`, `ML-2`, `ML-6`, `CT-3` |
| Meal detail | `/meals/[mealId]` | `ML-3`, `ML-4`, `CO-2` |
| Edit meal | `/meals/[mealId]/edit` | `ML-5`, `ML-6` |
| Day plan list | `/day-plans` | `DP-5`, `TG-2`, `CT-3` |
| Day plan categories | `/day-plans/categories` | `CT-1`, `CT-2`, `CT-5` |
| New day plan | `/day-plans/new` | `DP-1`, `DP-2`, `DP-6` |
| Day plan detail | `/day-plans/[dayPlanId]` | `DP-4`, `TG-2`, `TG-3`, `CO-2` |
| Edit day plan | `/day-plans/[dayPlanId]/edit` | `DP-5`, `DP-6` |
| Calendar | `/calendar` | `CL-1`–`CL-5`, `TG-2`, `TG-3` |
| Body | `/body` | The hub behind the fifth tab: one line per card for recovery, sleep, weight, measurements, energy balance and targets, and the one Health Connect row (`HC-1`, `HC-4`) |
| Targets | `/targets` | `TG-1` |
| Weight | `/weight` | `WT-1`–`WT-7` — quick log, the graph, and the weigh-in history |
| Recovery | `/body/recovery` | `IN-1`–`IN-4` |
| Sleep | `/body/sleep` | `IN-5` |
| Measurements | `/body/measurements` | `BM-1`–`BM-5` |
| Energy balance | `/body/energy` | `IN-6`, `IN-7` |

Creating and editing are screens rather than dialogs, because every write replaces the whole
entity and so has to start from a fresh read. Dialogs carry what is genuinely one decision:
assigning a plan to a date, applying a plan across weekdays, managing categories, and
confirming a delete.

**The composition editors** — meals and day plans — share one layout: identity, the ordered
rows, and the derived total shown as not editable (`ML-3`, `DP-4`). Rows are added by picking
the item, which creates the row already populated, and are reordered with move-up and
move-down buttons. Reordering is never drag-only: WCAG 2.2 SC 2.5.7 requires a single-pointer
alternative to any dragging motion, so buttons are the baseline rather than the fallback.

**The calendar** shows four weeks at a time and pages by the whole block (`CL-5`). An
unplanned day reads as unplanned and stays out of every denominator, rather than reading as a
shortfall — the target is a daily figure, so a four-week total cannot be read against it
directly.

**Planned against target** is drawn as a meter per macro, never a ring: a ring encodes
0–100% and has no honest way to render 130%, which `TG-3` requires.

**Colour.** Protein, fat and carbs each carry a fixed hue in every view — Okabe-Ito orange,
reddish purple and bluish green — held apart far enough to survive the common colour-vision
deficiencies. Energy stays neutral, because it is the sum of the other three rather than a
fourth sibling. No macro is red, and no macro uses the interactive accent: red in this
application always means something is broken, never that a target was exceeded (`TG-3`).
Colour is never the only channel — every figure is also labelled, and the series order never
changes.

### 3.2 Hardware interfaces
The device camera, on phones only, for barcode scanning (`SC-1`). A wearable reaches the
application only through Health Connect, never directly.

### 3.3 Software interfaces
- **Open Food Facts REST API** — read-only, over rustls with webpki roots. Search by text and fetch by barcode, behind the `Catalogue` trait.
- **SQLite** — a local file, reached only through `mealprep-sqlite`.
- **Barcode scanner** — `tauri-plugin-barcode-scanner`, on Android and iOS only.
- **Health Connect** — Android's on-device health store, read-only, through the `mealprep-health-connect` plugin. Every other platform reports it unsupported.

### 3.4 Communications interfaces
Between the frontend and the rules there is no network protocol: the webview invokes Tauri
commands over the IPC bridge, and arguments and results are JSON-serialised domain types.
The only outbound traffic is HTTPS to the OFF API; Health Connect is a local Android
service, not a network one. The application listens on no port.

## 4. System features
Features are ordered by the dependency between their domains: products come first, meals
build on products, day plans build on both.

### 4.1 Products
**Description and priority.** The unit the user configures first. High.

**Stimulus/response.** The user searches OFF by name or scans a barcode, reviews what the
catalogue supplied, and a snapshot of that product joins their own set. Or the user fills in a
product by hand and it joins the same set.

**Functional requirements.**
- `PR-1` The system shall let the user search the OFF database by text and show the matches.
- `PR-2` The system shall let the user add a searched product to their own products.
- `PR-3` The system shall let the user create a product by entering its name and nutrients per 100 g, or per 100 ml when it is a liquid.
- `PR-4` A product created by the user shall carry the same fields as an imported one.
- `PR-5` The system shall treat imported and hand-created products identically everywhere downstream; nothing shall depend on a product's origin.
- `PR-6` The system shall store every nutrient value an imported product carries, not only the four macros.
- `PR-7` The system shall require the four macros when the user creates a product by hand, and shall accept any further nutrient as optional.
- `PR-8` The system shall snapshot an imported product's data at pick time and shall not re-read it from OFF afterwards.
- `PR-9` The system shall display the four macros, and shall offer the remaining nutrient information behind an expansion.
- `PR-10` The system shall support create, read, update and delete for products, where delete is soft.
- `PR-11` The system shall record a product's brand where the catalogue carries one, and shall treat it as part of the product's identity rather than as presentation. It is optional for a product created by hand.

### 4.2 Meals
**Description and priority.** A labeled set of products with serving sizes. High.

**Functional requirements.**
- `ML-1` The system shall let the user create a meal with a label.
- `ML-2` The system shall let the user add products to a meal, each with a serving size in the product's unit.
- `ML-3` The system shall store a meal's composition only, and derive its nutrients on read.
- `ML-4` The system shall show a meal's computed nutrients alongside it.
- `ML-5` The system shall support create, read, update and delete for meals, where delete is soft.
- `ML-6` The system shall let the user order a meal's servings, and shall preserve that order.

### 4.3 Day plans
**Description and priority.** A labeled, ordered list of what is eaten in a day. High.

**Functional requirements.**
- `DP-1` The system shall let the user create a day plan with a label.
- `DP-2` The system shall let the user add whole meals and individual products to a day plan, in one shared order, because a day is eaten in one sequence.
- `DP-3` A day plan shall carry no properties beyond its label, its optional category and its items.
- `DP-4` The system shall show a day plan's nutrients as the sum of its items'.
- `DP-5` The system shall support create, read, update and delete for day plans, where delete is soft.
- `DP-6` The system shall let the user order a day plan's items, and shall preserve that order. A day plan has no time-of-day slots, so the order is the only sequence it carries.
- `DP-7` A product whose row is deleted shall remain in a day plan as a removed entry keeping its id, while a deleted meal shall drop out of it.

### 4.4 Calendar
**Description and priority.** Assignment of day plans to dates. Medium.

**Functional requirements.**
- `CL-1` The system shall let the user assign a day plan to a calendar day.
- `CL-2` The system shall let the user view the assigned plans of the weeks in view.
- `CL-3` A calendar day shall hold at most one day plan. Assigning a plan to a day that already has one shall replace it.
- `CL-4` The system shall let the user assign one day plan to several days at once, naming the days it will touch before writing anything.
- `CL-5` The system shall show four consecutive weeks at a time, and shall page forward and back by four weeks.

### 4.5 Categories
**Description and priority.** Optional grouping for meals and day plans. Medium. New in this
version.

**Functional requirements.**
- `CT-1` The system shall let the user create, rename and delete categories.
- `CT-2` A category shall belong to one scope — meals or day plans — fixed at creation and never changed afterwards.
- `CT-3` A meal or a day plan shall belong to at most one category, and may belong to none.
- `CT-4` A label shall be unique within its category, ignoring case; entities without a category form one scope of their own for this purpose.
- `CT-5` Deleting a category shall not delete what it holds; those entities become uncategorised.

### 4.6 Cost
**Description and priority.** What a plan costs. Medium. Out of scope in the predecessor.

**Functional requirements.**
- `CO-1` The system shall let the user record what 100 units of a product cost, as an optional figure.
- `CO-2` The system shall derive the cost of meals and day plans from their products' prices, the way macros are derived, and shall never store it.
- `CO-3` A derived cost shall carry whether every product in the composition had a price. Where one did not, the figure shall be presented as a floor rather than a total, so an unpriced product is never silently counted as free.

### 4.7 Catalogue import
**Description and priority.** Reviewing what a crowd-sourced entry supplies before keeping it.
Medium. New in this version.

**Functional requirements.**
- `IM-1` The system shall show an imported entry for review before it is stored, and shall store only what the user confirmed.
- `IM-2` The review shall name who supplied the entry, distinguishing a manufacturer-supplied entry from a crowd-sourced one.
- `IM-3` The review shall surface the catalogue's own figure-related quality warnings about the entry, and when it was last modified.
- `IM-4` A macro the entry does not carry shall seed the form empty rather than zero.
- `IM-5` The system shall record the catalogue code an imported product came from, as its provenance.

### 4.8 Barcode scanning
**Description and priority.** Getting to an entry without typing. Medium. New in this version.

**Functional requirements.**
- `SC-1` The system shall let the user scan a product barcode with the device camera on platforms that have one.
- `SC-2` A successful scan shall open the import review for that code.
- `SC-3` On a platform without a camera, a barcode typed into catalogue search shall reach the same review.
- `SC-4` The scan shall be cancellable, including by back navigation, and shall supply its own frame and cancel control because the plugin draws none.

### 4.9 Targets and feedback
**Description and priority.** The comparison the product exists for. Medium.

**Functional requirements.**
- `TG-1` The system shall let the user set daily targets for calories, fat, protein and carbohydrates.
- `TG-2` The system shall show the four planned macros next to the targets.
- `TG-3` The system shall display deviations from the targets and shall never block or alter a plan because of them.
- `TG-4` Energy shall be expressed in kcal, mass in grams and volume in millilitres, throughout the product.

### 4.10 Weight
**Description and priority.** What the targets exist to move. Medium. New in this version.

**Stimulus/response.** The user records the weight the scale showed. The application smooths
it into a trend, draws it against the previous weigh-ins, and says which way it is going.

**Functional requirements.**
- `WT-1` The system shall let the user record a body weight in kilograms against a date, defaulting to today.
- `WT-2` A date shall hold at most one weigh-in. Recording again for a date shall replace it, and the user shall be told that it will before they save.
- `WT-3` The system shall support reading and deleting weigh-ins, where delete is soft.
- `WT-4` The system shall derive a trend from the weigh-ins by exponential smoothing, and shall treat the days between two weigh-ins as though the weight moved evenly across them. A trend shall never be stored.
- `WT-5` The system shall present the trend as the headline figure rather than the most recent measurement, and shall draw the measurements alongside it rather than instead of it.
- `WT-6` The system shall derive a rate of change in kilograms per week from the trend, and shall withhold it until the history is long enough for the figure to mean anything.
- `WT-7` The graph shall scale to the data rather than to zero, and shall leave enough room below the lowest value that ordinary fluctuation cannot read as collapse.
- `WT-8` The system shall refuse a figure outside the plausible human range, and shall otherwise have no opinion about the weight recorded — no goal, no judgement, no congratulation.

### 4.12 Body measurements
**Description and priority.** Tape measurements and body fat, of which waist carries the most
evidence. Medium. New in this version.

**Functional requirements.**
- `BM-1` The system shall let the user record a waist, hips, chest, neck, arm, thigh, height (cm) or body fat (%) against a date, defaulting to today.
- `BM-2` A date shall hold at most one value per kind; recording again replaces it. Delete is soft.
- `BM-3` The system shall refuse a figure outside that kind's plausible range, as `WT-8` does.
- `BM-4` The system shall show the waist-to-height ratio from the latest waist and height, read in the NICE NG246 bands, and nothing about BMI.
- `BM-5` The system shall show how the waist and the weight trend moved over the same four weeks, and name the pattern only when it is clear.

### 4.13 Health Connect import
**Description and priority.** What a wearable records, on the device. Medium. New in this
version.

**Functional requirements.**
- `HC-1` The system shall ask for Health Connect access only when the user chooses to connect, and shall read sleep, resting heart rate, HRV, respiratory rate, workouts and energy burned. It shall never write.
- `HC-2` The system shall summarise what it reads into one row per morning, assigning a night and any reading taken from 18:00 onwards to the morning after, and using only the longest sleep session of a night.
- `HC-3` Energy burned and exercise time shall be taken as Health Connect aggregates them per day, so that overlapping apps are not counted twice.
- `HC-4` The system shall re-read Health Connect when the Body screen opens and the last import is at least an hour old, and on request. Each import replaces the days it covers and nothing else.
- `HC-5` Every figure shall be optional: a source that does not write one leaves the insights that need it absent, never wrong.

### 4.14 Insights
**Description and priority.** The correlations recovery and nutrition apps have converged on,
computed on the device. Medium. New in this version. The reasoning and sources are in
`docs/research/2026-09-27-health-insights-ui-ux.md`.

**Functional requirements.**
- `IN-1` The system shall show HRV and resting heart rate as a 7-day average against the user's own normal range — the last 60 days' mean ± half a standard deviation — and withhold the range below 14 readings.
- `IN-2` The system shall say in words whether the average is below, within or above the range, and never colour the verdict.
- `IN-3` The system shall compare the marker on mornings with and without each of: 7 hours' sleep, an earlier bedtime than usual, a workout the day before, a plan over the energy target and a plan at the protein target the day before. It shall do so over 90 days, only with five mornings on each side, using HRV where there is enough and resting heart rate otherwise.
- `IN-4` A difference under half a standard deviation shall read as no clear difference, and every comparison shall carry its counts and be called an association.
- `IN-5` The system shall show sleep duration over the last week, and bedtime and wake-time variability over four weeks once there are 14 nights.
- `IN-6` The system shall estimate expenditure over the last 21 days as planned intake minus the energy the weight trend stored, at 7,700 kcal per kilogram, only with 14 planned days and a trend covering the window.
- `IN-7` The wearable's own energy figure shall be shown beside the estimate as a comparison, and the estimate shall say it assumes the plan was eaten.

### 4.11 Plausibility checks
**Description and priority.** Catching a typo without refusing the save. Low. New in this
version.

**Functional requirements.**
- `PL-1` The system shall check a product's figures against each other as they are entered — energy against the macros, kJ against kcal, saturates against fat, sugars against carbohydrates, and the total against 100 g.
- `PL-2` A failed check shall warn and shall never block a save, consistent with `TG-3`.

## 5. Other non-functional requirements

### 5.1 Performance requirements
Out of scope as a target, with one constraint from the platform: the database is on a phone
and every read crosses the IPC bridge, so a screen loads what it displays and nothing more.

### 5.2 Safety requirements
None. The application gives no medical or dietary advice and enforces nothing, per `TG-3`
and `PL-2`. The recovery and sleep screens describe patterns in the user's own data and say
so (`IN-4`); they diagnose nothing.

### 5.3 Security requirements
No authentication and no transport security for the application itself, because it listens on
nothing and its data never leaves the device. The one outbound connection, to the OFF API, is
HTTPS with webpki roots. Health data is read from Health Connect with the user's permission,
copied into the private database, and sent nowhere. Giving this application a network service, or synchronising its
database anywhere, invalidates this section and requires it to be rewritten first.

### 5.4 Software quality attributes
- **Maintainability** — `docs/code-standards.md` is binding; the tree stays formatted and lint-clean.
- **Portability** — persistence is reachable only through the `core::store` traits, so the database can be swapped by adding a crate that implements them.
- **Correctness** — the rules live in `mealprep-core`, which has no I/O and is tested without one.
- Reliability and availability targets: out of scope, as in 5.1.

### 5.5 Logical database requirements
The database stores products with their full nutrient set and optional price, categories,
meals with their servings, day plans with their ordered items, calendar assignments, weigh-ins,
body measurements, imported health days with the time of the last import, and targets. Nutrient and cost totals are never stored; they are derived on read. Every table
carries `created_at`, `updated_at` and `deleted_at`; a delete sets `deleted_at` and removes no
row, and reads exclude soft-deleted rows. The schema is `crates/sqlite/migrations/`, applied
on open — which closes the predecessor's TBD-12 and its hand-run `ALTER TABLE`s.

### 5.6 Business rules
- The application informs and never enforces (`TG-3`, `PL-2`).
- A product's origin never changes its behaviour (`PR-5`).
- Nothing is deleted outright; entities are soft-deleted (5.5).
- A derived figure never hides what it is missing (`CO-3`).
- The trend is the figure worth reading; a single weigh-in is mostly noise (`WT-5`).

## 6. Other requirements
Open Food Facts data is published under the Open Database License (ODbL), with its individual
contents under the Database Contents License. The application shall credit Open Food Facts
wherever imported product data is shown, and shall link to the source entry on a product's
detail view. ODbL share-alike obligations attach to redistributing the database; a single
on-device instance redistributes nothing, so nothing further is required today. Publishing
this application's data, or shipping it with a bundled catalogue, revisits this section.

## Appendix A: Glossary
See 1.5.

## Appendix B: Analysis models
`docs/architecture.md` holds the component diagram, the crate layout and the domain model.

## Appendix C: To be determined list
IEEE 830 §4.3.3.1 asks that every TBD record why it is open and what closes it.

| ID | Open item | Why it is open | What closes it |
| --- | --- | --- | --- |
| TBD-3 | User documentation | Premature while the app is used by the person building it | Someone else installs it |
| TBD-13 | Whether a day plan's cost should be shown per day or per plan | Only the plan-level figure exists, and no view has needed the other | A view needs a per-day cost |

TBD-14 (weight against energy intake) is closed by `IN-6`.

The predecessor's TBD-12 (database schema) is closed: the schema is
`crates/sqlite/migrations/0001_initial.sql`, applied by `sqlx::migrate!` on open.

## Appendix D: Requirements retired by the port
Identifiers that were binding in the predecessor and are not reused here.

| ID | Was | Why it is gone |
| --- | --- | --- |
| `PR-7` (second clause) | An OFF entry missing the four macros shall be rejected at import | Replaced by `IM-4`: the entry is shown for review with those macros empty, and the user completes them. Rejecting a usable entry over a missing figure cost more than it saved. |
