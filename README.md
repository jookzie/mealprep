# Mealprep (Rust + Tauri)

Plan a week of meals against daily macro targets — calories, fat, protein and
carbohydrates — and see what it costs. A port of the Go/SvelteKit `mealprep` app to a
single Tauri v2 application whose data lives on the device.

Android is the primary target; the desktop build runs the same code.

## Layout

```
crates/core            domain types, business rules, storage traits (no I/O)
crates/sqlite          SQLite store (sqlx) and the schema migrations
crates/openfoodfacts   Open Food Facts catalogue client
src-tauri              Tauri commands wiring the three together
src                    SvelteKit static SPA (Svelte 5) the webview loads
docs/                  the specification, the architecture and the standards
```

## Docs

| Document | Holds |
| --- | --- |
| [`docs/requirements.md`](docs/requirements.md) | what the application does, as binding requirements |
| [`docs/architecture.md`](docs/architecture.md) | how the pieces fit and why |
| [`docs/code-standards.md`](docs/code-standards.md) | how the code is written |
| [`docs/todo.md`](docs/todo.md) | deferred decisions, not to be implemented unless asked |
| [`docs/demo-data.md`](docs/demo-data.md) | the demo database, and which data shows which feature |

## Prerequisites

- Rust ≥ 1.94 and [Bun](https://bun.sh).
- Desktop (Linux): the [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/#linux),
  notably `webkit2gtk-4.1`.
- Android: JDK 17+, Android SDK with platform tools, NDK, and `ANDROID_HOME` / `NDK_HOME` set as
  described in the [Tauri Android prerequisites](https://v2.tauri.app/start/prerequisites/#android).
  The Android Rust targets need `rustup`:
  `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`.

## Running

```sh
bun install

bun tauri dev                 # desktop window
bun tauri android init        # once: generates src-tauri/gen/android
bun tauri android dev         # emulator or USB device
bun tauri android build --apk # release APK
```

The database is `mealprep.db` in the platform's app data directory
(on Android, the app's private storage).

## Checks

```sh
cargo test --workspace        # domain rules, SQLite integration, catalogue mapping
cargo clippy --workspace --all-targets
bun test src                  # pure frontend modules (week dates, calendar join, …)
bun run check                 # svelte-check
```

`mise.toml` wraps these: `mise run test` runs both suites, `mise run lint` and
`mise run format` cover Rust, `mise run check` typechecks the frontend, and
`mise run android:dev` builds and installs on a running emulator. `mise tasks` lists them
all. `mise run demo:seed` rebuilds `demo/mealprep-demo.db`, a dataset that shows every
feature (see [`docs/demo-data.md`](docs/demo-data.md)). `mise run autocommit` drafts a
commit message from the staged diff and opens it in the editor for review.

## Scope of the port

Kept: products (manual, and Open Food Facts import by name search or barcode), meals, day plans with meals and loose
products in one order, meal and day-plan categories, the four-week calendar with bulk
assignment, targets, derived macros and cost with the incomplete-cost caveat, soft deletes
and dangling references rendered as removed.

Simplified: layout is a mobile-first bottom-tab shell with plain CSS instead of
shadcn-svelte; no command palette, radar chart or column heatmap; list sorting is local
state rather than a URL parameter.

Added: barcode scanning on phones (on desktop, type the barcode into search), a review
step before every import that says who supplied the entry and what Open Food Facts itself
doubts, and plausibility checks on every product form (energy against the macros, kJ
against kcal, saturates and sugars against their totals, grams against 100 g). The checks
warn and never block a save. Also a Body tab: a weight tracker whose weigh-ins are smoothed
into a trend line with the rate of change per week; body measurements with the
waist-to-height ratio; and, on Android, recovery, sleep and "what goes with better mornings"
from Health Connect (a WHOOP or any other wearable that writes to it), plus the energy
expenditure the plan and the weight trend imply.

The page only works inside the Tauri webview; opened in a plain browser it says so.
