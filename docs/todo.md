# Todo

Deferred decisions. Nothing here is implemented unless it is explicitly asked for. Finding an
entry relevant to the task at hand is not permission to act on it: the current behaviour
documented in `docs/requirements.md` stands until the entry is picked up deliberately.

Each entry records what happens today, what is deferred, and what would trigger the change.

## Frontend test runner
- **Today**: `bun test` covers the pure modules under `frontend/src/lib/domain` — macro
  arithmetic, week dates, the calendar join — with no config and no extra dependency.
  Playwright covers the critical paths end to end.
- **Deferred**: Vitest, which resolves `$lib` and `$app` and can render components.
- **Trigger**: the first unit test that needs to import `$app/*` or mount a component.

## Serving the built frontend
- **Today**: `mise run frontend:build` emits static files, and development goes through a
  Vite proxy to the backend on `PORT`. Nothing serves the bundle in `docker-compose.yml`.
- **Deferred**: how the artifact ships — the backend embedding it with `go:embed`, or a
  static file server beside the backend that also proxies `/v1`.
- **Trigger**: wanting to run the application outside a development machine.

## Biome's Svelte support
- **Today**: `biome` formats and lints `.svelte` files with
  `html.experimentalFullSupportEnabled`, which is what makes template handling work.
- **Deferred**: dropping the flag once the support leaves experimental, or adding
  `prettier-plugin-svelte` for `.svelte` alone if the markup formatter proves unreliable.
- **Trigger**: a Biome release that stabilises it, or formatting churn that costs review time.

## Schema changes against an existing database
- **Today**: `backend/sqlc/sqlite/schema.sql` is applied at startup with
  `CREATE TABLE IF NOT EXISTS`, so a fresh database gets the current schema and an existing
  one keeps whatever it already had. Adding `products.brand` needed a manual
  `ALTER TABLE products ADD COLUMN brand TEXT` against the development database; SQLite has
  no `ADD COLUMN IF NOT EXISTS`, so the statement cannot simply live in `schema.sql`.
- **Deferred**: a migration mechanism — numbered files with a `schema_version` table, or a
  library — so a column can be added without a hand-run statement.
- **Trigger**: the second schema change that has to reach a database someone cares about,
  or the application being deployed anywhere its data cannot be recreated.
