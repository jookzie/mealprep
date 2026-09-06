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
cmd/app/main.go        entrypoint: create the application, Start, Shutdown on signal
internal/
  api/                 generated models and ServerInterface (do not edit)
  application/         wiring, config, logger, Fiber lifecycle
  handler/<domain>/    ServerInterface implementations, DTO mapping, error to status
  service/<domain>/    business logic, the errors it returns
  repository/<db>/<domain>/   persistence, e.g. repository/sqlite/meal
  client/<api>/        external APIs, e.g. client/openfoodfacts
  errorx/              structured error types shared across layers
```
What belongs in a container package versus its sub-packages is covered in
[code-standards](code-standards.md#packages).

## Layering
`handler → service → repository | client`. Lower layers never import higher ones.
`application` imports all of them and wires them together.

## Key decisions
- **Consumer-defined interfaces**: a store package returns its concrete type, and each caller declares the narrow interface it needs. Swapping the database means writing another store, not editing a shared contract.
- **Sub-packages per category**: every layer grows sideways, and no package accumulates unrelated files.
- **Nutrients computed in the service layer**: meals and day plans store composition only; totals are derived on read.
- **Config from the environment**: parsed once in `application`, with no defaults in code.
- **Global logger**: one `zerolog`, configured in `application.New`.
- **Spec-first API**: handlers implement a generated interface, so a route that drifts from `api/openapi.yaml` stops compiling.
- **Placeholders allowed**: a package with no current need stays empty rather than speculative.

## Domain model
- `Product` — id, name, full nutrient set per 100 g or 100 ml. Snapshotted from Open Food Facts or created by the user.
- `Meal` — label + list of (product, serving size in grams or millilitres).
- `DayPlan` — label + list of meals.
- `CalendarDay` — date → day plan.
- `Targets` — energy (kcal), fat, protein, carbs.

Every entity carries `created_at`, `updated_at` and `deleted_at`; deletes are soft.
Energy is kcal, mass is grams, volume is millilitres.
