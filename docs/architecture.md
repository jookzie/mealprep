# Architecture

```
browser (Svelte+TS+shadcn)  ──HTTP/JSON──▶  Go backend (Fiber)  ──▶  SQLite
                                                   │
                                                   └──▶  Open Food Facts API
```

## Backend layout
```
cmd/app/main.go        entrypoint: create application, Start, Shutdown on Ctrl+C
internal/
  application/         wiring, config (env), logger, Fiber lifecycle
  handler/             HTTP handlers, registered to Fiber
  service/             business logic (nutrient computation, planning)
  repository/          persistence interfaces + SQLite implementation
  client/              external APIs (Open Food Facts)
```

## Dependency direction
`handler → service → repository | client`. Lower layers never import higher ones. `application` imports all and wires them.

## Key decisions
- **Repository as interface**: SQLite is one implementation; swapping databases means adding a new implementation, not touching services.
- **Nutrients computed in service**: meals and day plans store composition only; nutrient totals are derived on read.
- **Placeholders allowed**: packages without current needs stay as empty placeholders.
- **Config from env**: parsed once in `application` with `caarlos0/env`.
- **Global logger**: `zerolog` configured in `application.New`, used everywhere.

## Domain model
- `Food` — from Open Food Facts (id, name, nutrients per 100g).
- `Meal` — label + list of (food, quantity).
- `DayPlan` — label + list of meals.
- `CalendarDay` — date → day plan.
- `Targets` — calories, fat, protein, carbs.
