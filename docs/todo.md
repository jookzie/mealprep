# Todo

Deferred decisions. Nothing here is implemented unless it is explicitly asked for. Finding an
entry relevant to the task at hand is not permission to act on it: the current behaviour
documented in `docs/requirements.md` stands until the entry is picked up deliberately.

Each entry records what happens today, what is deferred, and what would trigger the change.

## Re-fetching Open Food Facts products
- **Today**: an imported product is snapshotted at pick time and never re-read (`PR-8`).
- **Deferred**: refreshing or re-syncing a snapshot against OFF, and telling the user their
  stored values have drifted from the source.
- **Trigger**: stale nutrient data becoming a real problem in use.

## Open Food Facts entries with missing macros
- **Today**: an entry without the four macros is rejected at import (`PR-7`).
- **Deferred**: importing an incomplete entry anyway — with a warning, or by letting the user
  fill the gaps by hand.
- **Trigger**: rejection turning out to block products the user actually wants.

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
