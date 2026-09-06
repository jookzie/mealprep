# Architecture

## Overview
```
browser (Svelte+TS+shadcn)  ──HTTP/JSON──▶  Go backend (Fiber)  ──▶  SQLite
                                                   │
                                                   └──▶  Open Food Facts API
```

## API contract
`api/openapi.yaml` is the source of truth for the HTTP API. Go server code is generated
from it with [oapi-codegen](https://github.com/oapi-codegen/oapi-codegen) (`fiber-v3-server`
plus models) into `internal/api`, and the frontend client is generated from the same file.
Regenerate with `mise run openapi:gen`; never edit `*.gen.go`.

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

## Key decisions
- **Consumer-defined interfaces**: a store package returns its concrete type, and each caller declares the narrow interface it needs. Swapping the database means writing another store, not editing a shared contract.
- **Sub-packages per category**: every layer grows sideways, and no package accumulates unrelated files.
- **Nutrients computed in the service layer**: meals and day plans store composition only; totals are derived on read.
- **Config from the environment**: parsed once in `application`; only optional settings carry a default.
- **Global logger**: one `zerolog`, configured in `application.New`.
- **Spec-first API**: handlers implement a generated interface, so a route that drifts from `api/openapi.yaml` stops compiling.
- **SQL-first persistence**: queries are written by hand and `sqlc` generates the Go for them, so the database access is as reviewable as the rest.
- **Snapshot, not reference**: an imported product is copied at pick time and never re-read from the catalog ([todo](todo.md)).

## Domain model
- `Product` — id, name, unit, the four macros and every further nutrient per 100 units. Snapshotted from Open Food Facts (with `sourceCode`) or created by the user.
- `Meal` — label + list of (product, serving amount in the product's unit).
- `DayPlan` — label + list of meals.
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
