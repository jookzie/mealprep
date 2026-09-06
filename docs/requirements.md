# Requirements

## Purpose
Help the user plan a week of meals against target macros (calories, fat, protein, carbs). The app **informs**, it never enforces targets.

## Functional
- **Targets**: user sets daily targets for calories, fat, protein, carbs.
- **Foods**: search and pick foods from the [Open Food Facts](https://world.openfoodfacts.org/) database.
- **Meals**: a labeled set of foods with quantities. Shown with computed nutrients.
- **Day plans**: a labeled group of meals, no other properties. Shown with computed nutrients (sum of meals).
- **Calendar**: assign a day plan to a calendar day.
- **Feedback**: nutrients are shown next to targets; deviations are displayed, not blocked.

## Non-functional
- Browser frontend: Svelte + TypeScript + shadcn.
- Backend: Go, HTTP API via Fiber.
- Storage: SQLite now; repository layer keeps the database swappable.
- Single user for now; no auth.
