# Architecture

## Overview
```
static SPA (SvelteKit+TS+shadcn)  ──HTTP/JSON──▶  Go backend (Fiber)  ──▶  SQLite
                                                          │
                                                          └──▶  Open Food Facts API
```
The frontend builds to static files and runs entirely in the browser; the backend is a
separate process that owns all state. Both sides are generated from the same contract.

## API contract
`api/openapi.yaml` is the source of truth for the HTTP API, and both sides are generated
from it:

| Side | Generator | Output |
| --- | --- | --- |
| Go server | [oapi-codegen](https://github.com/oapi-codegen/oapi-codegen) (`fiber-v3-server` plus models) | `backend/internal/api/api.gen.go` |
| TypeScript client | [@hey-api/openapi-ts](https://heyapi.dev) (one function per `operationId`) | `frontend/src/lib/api/gen/` |

`mise run openapi:gen` regenerates both. Neither output is ever edited by hand: a route
that drifts from the spec stops compiling on the Go side and stops typechecking on the
frontend. The spec itself is linted with [vacuum](https://quobix.com/vacuum/) —
`mise run openapi:lint`, ruleset in `.vacuum.yaml` — so a broken contract is caught before
it produces two broken sides.

## Backend layout
```
api/openapi.yaml       the API contract, source of both generated sides
backend/sqlc/sqlite/   the SQL the data access layer is generated from
cmd/app/main.go        entrypoint: create the application, Start, Shutdown on signal
internal/
  api/                 generated models and ServerInterface (do not edit)
  application/         config, logger, Fiber lifecycle, and the wiring structs
  domain/              the entities every layer shares
  handler/<domain>/    ServerInterface implementations, DTO mapping, error to status
  service/<domain>/    business logic, the errors it returns, derived nutrients
  repository/<source>/<domain>/   persistence and catalog access
  client/<source>/     connections to the outside: sqlite, openfoodfacts
  errorx/              structured error types shared across layers
```
What belongs in a container package versus its sub-packages is covered in
[code-standards](code-standards.md#packages).

## Wiring
`application.New` builds four private structs in order, each from the one before it:

```
clients ──▶ repositories ──▶ services ──▶ handlers ──▶ registered on /v1
```

Each lives in its own file (`clients.go`, `repositories.go`, `services.go`,
`handlers.go`) and carries a `from` method taking what it is built on. `handlers`
embeds the per-domain handlers, so one value satisfies the whole generated
`ServerInterface`. Every layer depends on the one below through an interface the
consumer declares, so a service names the repository methods it uses and nothing more.

## Layering
`handler → service → repository → client`. Lower layers never import higher ones.
`application` imports all of them and wires them together. `domain` and `errorx` sit
under everything and import nothing of their own.

## Frontend layout
```
frontend/
  openapi-ts.config.ts   what generates src/lib/api/gen from api/openapi.yaml
  biome.json             one formatter and linter for the whole frontend
  playwright.config.ts   the smoke tests, run against a live backend
  e2e/                   those tests
  src/
    app.css              Tailwind entry and the shadcn-svelte theme tokens
    routes/              one directory per screen: +page.ts reads, +page.svelte renders
    lib/
      api/               the only way the app reaches the backend
        gen/             generated client (do not edit)
      domain/            pure logic: macro arithmetic, week dates, the calendar join
      components/
        ui/              shadcn-svelte primitives (do not edit)
        app/             shell and cross-domain pieces: sidebar, command palette,
                         number field, move buttons, sortable header, skeletons
        <domain>/        macros, product, meal, day-plan, calendar
```
Routes own all I/O. Components take data and callbacks, never call the API themselves.

## Frontend data flow
```
+page.ts load ──▶ $lib/api (generated SDK) ──▶ backend
      ▲                                           │
      └────── invalidateAll() ◀── runMutation ────┘
```
Reads are `load` functions, so navigation and the URL drive them — the calendar's week
and the catalog search's query are both query parameters rather than component state.
Writes go through `runMutation`, which toasts, re-runs every load, and redirects; it is
the one place the failure path is written. There is no second caching layer.

Editors load the entity first, because every write is a full-replacement `PUT`. A form is
seeded from its props exactly once and re-seeded by keying it on the entity's
`updatedAt`, so an invalidation mid-edit cannot overwrite what is being typed.

## Key decisions
- **Consumer-defined interfaces**: a store package returns its concrete type, and each caller declares the narrow interface it needs. Swapping the database means writing another store, not editing a shared contract.
- **Sub-packages per category**: every layer grows sideways, and no package accumulates unrelated files.
- **Nutrients computed in the service layer**: meals and day plans store composition only; totals are derived on read.
- **Config from the environment**: parsed once in `application`; only optional settings carry a default.
- **Global logger**: one `zerolog`, configured in `application.New`.
- **Spec-first API**: handlers implement a generated interface, so a route that drifts from `api/openapi.yaml` stops compiling.
- **SQL-first persistence**: queries are written by hand and `sqlc` generates the Go for them, so the database access is as reviewable as the rest.
- **Snapshot, not reference**: an imported product is copied at pick time and never re-read from the catalog ([todo](todo.md)).
- **Static SPA**: the frontend builds to static files (`adapter-static`, `ssr = false`). Nothing runs on a server at request time, so the backend stays a separate instance and how the bundle is served is a deployment question, not a code one.
- **Generated client, one module**: `$lib/api` is the only path to the backend. Components never call `fetch`, and nothing outside that directory imports `gen/`.
- **Macro colour is identity, not status**: `app.css` fixes one hue per macro and the app never introduces a second colour to mean "over target" — over-target is the same hue under a hatch. That keeps `--destructive` meaning "broken" and nothing else, and is why the palette contains no red. Energy is the sum of the other three, so it stays neutral rather than becoming a fourth series.
- **A meter, not a progress bar**: planned-against-target is a scalar within a known range that can legitimately exceed its maximum, which is a `meter`; a `progressbar` is monotonic task completion. `macros/macro-meter.svelte` wraps bits-ui's `Meter` and sets `aria-valuetext` explicitly, because a percentage reading of 231 against a max of 200 announces as nonsense.
- **Ordering is persisted, and now editable**: `meal_servings` and `day_plan_meals` have always carried `position`, and the API's array order is that column. Reordering is exposed with move-up/move-down buttons rather than drag, because WCAG 2.2 SC 2.5.7 makes a non-drag path the required baseline.
- **Bulk assignment is a client-side loop**: there is no endpoint that writes a range, so applying a plan across weekdays issues one `PUT /calendar/{date}` per day, in sequence — the backend holds a single SQLite writer, so concurrency would only contend for its lock.
- **Totals joined in the browser**: the API derives macros for meals and day plans, but no endpoint totals a date range. `domain/calendar.ts` is the single place a `dayPlanId` becomes macros, so the week grid and its summary cannot disagree. It is also where an unplanned day is kept out of the week's denominator.
- **Dangling references are a state, not an error**: entities are soft-deleted and then never returned, so every client-side join can miss. A serving whose product is gone, and a date whose plan is gone, each render as removed while keeping the id, so the user can still fix them.
- **One linter per language**: `golangci-lint` for Go, `biome` for the frontend, `vacuum` for the spec. `mise` holds the tasks and the pre-commit hook runs them.

## Domain model
- `Product` — id, name, optional brand, unit, the four macros and every further nutrient per 100 units. Snapshotted from Open Food Facts (with `sourceCode`) or created by the user. Brand is identity, not presentation: two imports often share a name and differ only there.
- `Meal` — label + list of (product, serving amount in the product's unit).
- `DayPlan` — label + ordered list of meals, each carrying its own servings so a plan breaks down to products without a second read of every meal.
- `CalendarDay` — date → day plan, at most one per date.
- `Targets` — energy (kcal), fat, protein, carbs. Exactly one row.

Every entity carries `created_at`, `updated_at` and `deleted_at`; deletes are soft.
Energy is kcal, mass is grams, volume is millilitres. Macros on a meal or day plan
are derived on read and never stored.

## Data access
`backend/sqlc/sqlite/schema.sql` is the schema, applied at startup by the SQLite
client, and `backend/sqlc/sqlite/queries/` holds the SQL. `mise run sqlc:gen`
generates `internal/repository/sqlite/db` from both. Repositories map generated
rows to domain entities; nothing above them sees a `db` type.
