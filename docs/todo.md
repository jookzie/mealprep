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
