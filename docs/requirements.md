# Software Requirements Specification — Mealprep

Structured after [IEEE Std 830-1998](http://www.cse.msu.edu/~cse870/IEEEXplore-SRS-template.pdf)
and the [markdown SRS template](https://github.com/antonmry/markdown-templates/blob/master/requirements.md).

## 1. Introduction

### 1.1 Purpose
Mealprep helps one person plan a week of meals against daily macro targets — calories, fat,
protein and carbohydrates.

### 1.2 Scope
Mealprep is a single self-hosted web application: a browser frontend, an HTTP/JSON backend
and a local database. It covers configuring products, composing meals from them, grouping
meals into day plans, assigning day plans to calendar days, and reporting nutrients against
targets. It does not cover shopping lists, recipes with instructions, cost tracking,
multiple users or sharing.

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
- **DayPlan** — a labeled group of meals, with no other properties.
- **CalendarDay** — a date with a day plan assigned to it.
- **Targets** — the user's daily figures for energy (kcal), fat, protein and carbohydrates.
- **Macro** — one of those four figures.
- **OFF** — [Open Food Facts](https://world.openfoodfacts.org/), the external product database.
- **SRS** — this document.

### 1.6 References
- [Open Food Facts](https://world.openfoodfacts.org/) — external product database.
- `docs/architecture.md` — system structure and key decisions.
- `docs/code-standards.md` — binding implementation constraints.
- `api/openapi.yaml` — the API contract, source of truth for both sides.
- `docs/todo.md` — deferred decisions, not to be implemented unless asked.

## 2. Overall description

### 2.1 Product perspective
A new, self-contained product with no predecessor. The browser frontend talks HTTP/JSON to
the Go backend, which owns a SQLite database and reads from the OFF API. See
`docs/architecture.md` for the component diagram.

### 2.2 Product functions
- Configure products, by importing from OFF or creating them by hand.
- Compose meals from products with serving sizes in grams.
- Group meals into day plans.
- Assign day plans to calendar days.
- Set daily macro targets and see planned nutrients against them.

### 2.3 User classes and characteristics
One class, the single user, who both configures products and plans meals. No roles, no
administrator, no authentication. The user is assumed to understand macros well enough to
set their own targets.

### 2.4 Operating environment
- Backend: Go 1.26 with Fiber v3, running in a Linux container (`docker-compose.yml`).
- Storage: a SQLite database file on a mounted volume.
- Frontend: SvelteKit + TypeScript + shadcn-svelte, built with Bun into a static single-page app. Any browser that runs the compiled bundle is supported; no older baseline is targeted.
- Deployment: localhost only. The application is not reachable from another machine.

### 2.5 Design and implementation constraints
- `docs/code-standards.md` governs all implementation and overrides nothing in this document.
- The API is spec-first: `api/openapi.yaml` generates the Go server and the frontend client.
- Persistence sits behind a repository layer so the database stays swappable.
- No authentication and no user accounts while the product is single-user.

### 2.6 User documentation
*To be determined* (TBD-3).

### 2.7 Assumptions and dependencies
- The OFF API is reachable and free to use under its terms; the application degrades to
  hand-created products when it is not.
- An OFF entry without the four macros is rejected at import (`PR-7`).
- One user, one dataset, no concurrent editing from multiple devices.

## 3. External interface requirements

### 3.1 User interfaces
A browser UI built from shadcn-svelte components, served as a static single-page app.
Nutrients are shown as the four macros, with the full nutrient set behind an expansion
(`PR-9`).

Navigation is a persistent sidebar listing Calendar, Day plans, Meals, Products and
Targets, in that order. `/` redirects to the calendar. There is no separate dashboard:
the calendar already carries the planned-against-target figures `TG-2` and `TG-3` ask for,
and a second screen repeating them would be one more thing to keep true.

| Screen | Path | Serves |
| --- | --- | --- |
| Product list | `/products` | `PR-10` — the user's products, with delete |
| New product | `/products/new` | `PR-3`, `PR-7` — the four macros required, others optional |
| Catalog search | `/products/search?q=` | `PR-1`, `PR-2`, `PR-8` — search and snapshot an entry |
| Product detail | `/products/{id}` | `PR-9`, §6 — macros, nutrient expansion, source credit and link |
| Edit product | `/products/{id}/edit` | `PR-10` |
| Meal list | `/meals` | `ML-5` |
| New meal | `/meals/new` | `ML-1`, `ML-2` — products with serving sizes |
| Meal detail | `/meals/{id}` | `ML-3`, `ML-4` — servings and the derived nutrients |
| Edit meal | `/meals/{id}/edit` | `ML-5` |
| Day plan list | `/day-plans` | `DP-5` |
| New day plan | `/day-plans/new` | `DP-1`, `DP-2` |
| Day plan detail | `/day-plans/{id}` | `DP-4`, `TG-2`, `TG-3` — the sum, against the targets |
| Edit day plan | `/day-plans/{id}/edit` | `DP-5` |
| Calendar | `/calendar?week=` | `CL-1`, `CL-2`, `CL-3`, `TG-2`, `TG-3` — a week, its assignments and totals |
| Targets | `/targets` | `TG-1` |

Creating and editing are screens rather than dialogs, because every write replaces the
whole entity and so has to start from a fresh read. Dialogs carry the single-field
actions: assigning a plan to a date, and confirming a delete.

The calendar compares a week's total against the daily target multiplied by the days
actually planned, so an unplanned day reads as unplanned rather than as a shortfall.

### 3.2 Hardware interfaces
None. The application runs on commodity hardware and talks to no devices.

### 3.3 Software interfaces
- **Open Food Facts REST API** — read-only. Product search and fetch by identifier.
- **SQLite** — local file, accessed only through `internal/repository/sqlite`.

### 3.4 Communications interfaces
HTTP/JSON between frontend and backend, versioned under `/v1`, described by
`api/openapi.yaml`. Plain HTTP over loopback; no transport security while the deployment is
localhost only.

## 4. System features
Features are ordered by the dependency between their domains: products come first, meals
build on products, day plans build on meals.

### 4.1 Products
**Description and priority.** The unit the user configures first. High.

**Stimulus/response.** The user searches OFF by name, picks a result, and a snapshot of that
product joins their own set. Or the user fills in a product by hand and it joins the same set.

**Functional requirements.**
- `PR-1` The system shall let the user search the OFF database by text and show the matches.
- `PR-2` The system shall let the user add a searched product to their own products.
- `PR-3` The system shall let the user create a product by entering its name and nutrients per 100 g, or per 100 ml when it is a liquid.
- `PR-4` A product created by the user shall carry the same fields as an imported one.
- `PR-5` The system shall treat imported and hand-created products identically everywhere downstream; nothing shall depend on a product's origin.
- `PR-6` The system shall store every nutrient value an imported product carries, not only the four macros.
- `PR-7` The system shall require the four macros when the user creates a product by hand, and shall accept any further nutrient as optional. An OFF entry missing the four macros shall be rejected.
- `PR-8` The system shall snapshot an imported product's data at pick time and shall not re-read it from OFF afterwards.
- `PR-9` The system shall display the four macros, and shall offer the remaining nutrient information behind an expansion.
- `PR-10` The system shall support create, read, update and delete for products, where delete is soft.

### 4.2 Meals
**Description and priority.** A labeled set of products with serving sizes. High.

**Stimulus/response.** The user names a meal, selects products, and gives each a serving
size. The meal is shown with its computed nutrients.

**Functional requirements.**
- `ML-1` The system shall let the user create a meal with a label.
- `ML-2` The system shall let the user add products to a meal, each with a serving size in grams or millilitres.
- `ML-3` The system shall store a meal's composition only, and derive its nutrients on read.
- `ML-4` The system shall show a meal's computed nutrients alongside it.
- `ML-5` The system shall support create, read, update and delete for meals, where delete is soft.

### 4.3 Day plans
**Description and priority.** A labeled group of meals. High.

**Stimulus/response.** The user names a day plan and adds meals to it. The day plan is shown
with the sum of its meals' nutrients.

**Functional requirements.**
- `DP-1` The system shall let the user create a day plan with a label.
- `DP-2` The system shall let the user group meals into a day plan.
- `DP-3` A day plan shall carry no properties beyond its label and its meals.
- `DP-4` The system shall show a day plan's nutrients as the sum of its meals'.
- `DP-5` The system shall support create, read, update and delete for day plans, where delete is soft.

### 4.4 Calendar
**Description and priority.** Assignment of day plans to dates. Medium.

**Functional requirements.**
- `CL-1` The system shall let the user assign a day plan to a calendar day.
- `CL-2` The system shall let the user view a week of calendar days with their assigned plans.
- `CL-3` A calendar day shall hold at most one day plan. Assigning a plan to a day that already has one shall replace it.

### 4.5 Targets and feedback
**Description and priority.** The comparison the product exists for. Medium.

**Functional requirements.**
- `TG-1` The system shall let the user set daily targets for calories, fat, protein and carbohydrates.
- `TG-2` The system shall show the four planned macros next to the targets.
- `TG-3` The system shall display deviations from the targets and shall never block or alter a plan because of them.
- `TG-4` Energy shall be expressed in kcal, mass in grams and volume in millilitres, throughout the product.

## 5. Other non-functional requirements

### 5.1 Performance requirements
Out of scope. One user on localhost sets no meaningful load.

### 5.2 Safety requirements
None. The application gives no medical or dietary advice and enforces nothing, per `TG-3`.

### 5.3 Security requirements
No authentication, and no transport security, because the deployment is localhost only and
single-user. Exposing the application beyond the local machine invalidates this section and
requires it to be rewritten first.

### 5.4 Software quality attributes
- **Maintainability** — `docs/code-standards.md` is binding; the tree stays formatted and lint-clean.
- **Portability** — persistence is reachable only through the repository layer, so the database can be swapped by adding an implementation.
- **Correctness** — handlers implement a generated interface, so a route that drifts from `api/openapi.yaml` fails to compile.
- Reliability and availability targets: out of scope, as in 5.1.

### 5.5 Logical database requirements
The database stores products with their full nutrient set, meals with their servings, day
plans, calendar assignments and targets. Nutrient totals are never stored; they are derived
on read. Every entity carries `created_at`, `updated_at` and `deleted_at`; a delete sets
`deleted_at` and removes no row, and reads exclude soft-deleted rows unless asked otherwise.
Schema: *To be determined* (TBD-12).

### 5.6 Business rules
- The application informs and never enforces (`TG-3`).
- A product's origin never changes its behaviour (`PR-5`).
- Nothing is deleted outright; entities are soft-deleted (5.5).

## 6. Other requirements
Open Food Facts data is published under the Open Database License (ODbL), with its
individual contents under the Database Contents License. The application shall credit
Open Food Facts wherever imported product data is shown, and shall link to the source entry
on a product's detail view. ODbL share-alike obligations attach to redistributing the
database; a localhost, single-user deployment redistributes nothing, so nothing further is
required today. Publishing this instance, or exporting its data to others, revisits this
section.

## Appendix A: Glossary
See 1.5.

## Appendix B: Analysis models
`docs/architecture.md` holds the component diagram, the backend layout and the domain model.

## Appendix C: To be determined list
IEEE 830 §4.3.3.1 asks that every TBD record why it is open and what closes it.

| ID | Open item | Why it is open | What closes it |
| --- | --- | --- | --- |
| TBD-3 | User documentation | Premature before the UI exists | The UI stabilises |
| TBD-12 | Database schema | Persistence is not implemented | `repository/sqlite` is designed |
