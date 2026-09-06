# Architecture

## Overview
```
browser (Svelte+TS+shadcn)  ──HTTP/JSON──▶  Go backend (Fiber)  ──▶  SQLite
                                                   │
                                                   └──▶  Open Food Facts API
```

## Backend layout
```
cmd/app/main.go        entrypoint: create the application, Start, Shutdown on signal
internal/
  application/         wiring, config, logger, Fiber lifecycle
  handler/             HTTP handlers, one sub-package per resource
  service/             business logic, one sub-package per domain
  repository/          persistence, one sub-package per store
    sqlite/
  client/              external APIs, one sub-package per API
    openfoodfacts/
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
- **Placeholders allowed**: a package with no current need stays empty rather than speculative.

## Domain model
- `Food` — from Open Food Facts (id, name, nutrients per 100g).
- `Meal` — label + list of (food, quantity).
- `DayPlan` — label + list of meals.
- `CalendarDay` — date → day plan.
- `Targets` — calories, fat, protein, carbs.
