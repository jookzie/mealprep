# Code Standards

## Baseline
Rust code follows the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and
the [Rust Style Guide](https://doc.rust-lang.org/style-guide/), as `rustfmt` and `clippy`
enforce them. Whatever they settle is settled here. The sections below are this project's
additions and, where marked, its overrides.

The predecessor's standards were written for Go. Where a rule survived the port it is kept in
the same words; where the language answers it already, it is gone.

## Rust

### Crates
- `mealprep-core` holds the domain, the rules and the storage traits, and performs no I/O. It must not depend on `mealprep-sqlite`, `mealprep-openfoodfacts` or `tauri`.
- An adapter crate provides one outside thing: `mealprep-sqlite` the database, `mealprep-openfoodfacts` the catalogue. It depends on `core` and never on another adapter.
- `src-tauri` chooses the implementations, exposes the commands, and is the only crate that knows both a concrete store and a window exist.
- A new outside dependency is a new crate implementing a `core::store` trait, not a branch inside an existing one.

### Modules
- `core` splits three ways: `domain` (the types), `service` (the rules over them), `store` (what the rules need from the outside).
- Each of those splits by aggregate — `product`, `meal`, `day_plan`, `calendar`, `category`, `targets` — one file each, the same names in all three.
- A `mod.rs` declares its private modules and re-exports their public names through one `pub use` block. Callers import from the module, never from the file inside it.
- Every module carries a `//!` comment stating its role where the role is not obvious from the name.

### Files
- Small and single-purpose.
- A file is named after the main type it hosts, and after the concept its code represents when it hosts no single type.

### Types
- Domain types are plain data with public fields. They validate on the way in, through `service::validate`, not in a constructor per type.
- A draft type (`ProductDraft`, `MealDraft`) is what the user writes; the entity is what the store returns. A command takes a draft and answers an entity.
- Prefer an enum to a boolean pair or a nullable field where the states are named: `DayPlanItem` is an enum over meal and product, `CategoryScope` over the two scopes.
- A derived figure travels with its caveat rather than beside it. `Cost` carries `complete`, so no view can render the amount without knowing whether it is a floor.

### Services
- Rules live in `impl` blocks on `Mealprep<S, C>`, one file per aggregate, and each block bounds only the traits it uses — `S: ProductStore`, not `S: Store`.
- A service method validates, then calls the store. Validation lives in `service::validate` so the same rule cannot be written twice with two answers.
- Derivation lives in `service::derive`. Macros and cost are computed on read and never stored.

### Storage traits
- One trait per aggregate in `core::store`, declaring what the rules need and nothing more.
- Traits are `#[trait_variant::make(Send)]`, because Tauri's async commands need `Send` futures.
- A trait method's doc comment states the rules the implementation must keep — that deletes are soft, that nothing deleted is returned, what the ordering is. An implementation that breaks one is wrong even when it compiles.
- `core` never names a concrete store. `src-tauri/src/setup.rs::AppState` is the one place the choice is made.

### Commands
- One `#[tauri::command]` per operation, in `src-tauri/src/command/<aggregate>.rs`.
- A command takes `State<'_, AppState>` and the operation's arguments, calls one service method, and returns its value. No logic, no mapping, no DTOs: the domain types are the wire format.
- Everything the frontend can do is a command. Nothing else is exposed.

### Errors
- `core::Error` is a `thiserror` enum and the only error the rules return. Each variant names the `Entity` it is about, so a message reads the same wherever it surfaces.
- Adapters convert their library's errors into `core::Error` at the boundary. A `sqlx::Error` or a `reqwest::Error` never leaves its crate.
- `src-tauri` serialises the error as `{ kind, message }`. `kind` is the variant in kebab-case, and it exists so the frontend can react without parsing the message.
- Errors are returned, not logged, until the boundary that answers the user.

### Entities
- Every persisted entity carries `created_at`, `updated_at` and `deleted_at`.
- Deletes are soft: `deleted_at` is set and the row stays. Nothing removes rows.
- Reads exclude soft-deleted rows. `deleted_at` is a storage concern and is not a field on a domain type.

### Data access
- The schema is a migration in `crates/sqlite/migrations`, embedded with `sqlx::migrate!` and applied when the store opens. A schema change is a new migration file, never an edit to an applied one.
- SQL is written by hand and lives next to the code that runs it. Rows map to domain types in `crates/sqlite`, and no row type escapes the crate.

### Tests
- The rules are tested in `core`, without a database, because `core` has no I/O to stand up.
- Unit tests sit in a `#[cfg(test)] mod tests` beside the code they cover.
- A crate's integration tests sit in its `tests/` directory: `crates/sqlite/tests/service.rs` drives the real store through the services.

### Tooling
- The tree stays formatted and lint-clean: `mise run format` and `mise run lint`.
- `clippy` runs with `-D warnings`. A lint that is wrong here is silenced at the item with a reason, never project-wide.

## Frontend

### Baseline
Svelte 5 with runes, following
[Svelte best practices](https://svelte.dev/docs/svelte/best-practices). Whatever it
settles is settled here. TypeScript in strict mode.

### Structure
- `src/routes/` holds one directory per screen; `src/lib/` holds what they share.
- `lib/api` is the only path to Rust. `lib/domain` is pure logic — no Svelte, no `invoke`. `lib/components` holds what more than one screen uses.
- `lib/api/types.ts` mirrors the Rust domain types by hand. It is not generated, so it is changed in the same commit as the Rust type it mirrors.

### Files
- Small and single-purpose, named after the component or concept they host.
- Components are PascalCase (`ProductForm.svelte`), modules kebab-case (`day-plan.ts`).

### Routes
- A read is a `load` in `+page.ts`, never an `invoke` inside a component.
- A failing read throws through `read` and lands in `+error.svelte`.
- Create and edit are routes, not dialogs: every write is a full replacement, so an editor has to load the entity first. Dialogs carry what is genuinely one decision.

### Components
- No component calls `invoke`. Forms take a draft and an `onSubmit`; lists take rows and callbacks. Routes own all I/O.
- There is no component library. Styling is plain CSS against the tokens in `app.css`; a component that needs a token adds it there rather than hard-coding a value.
- A component that carries a binding requirement — the Open Food Facts credit, the target meters — is the single place that requirement is implemented, so it cannot be half-applied.

### State
- `$derived` over `$effect`. Effects are an escape hatch.
- Classes or plain `$state` fields; never stores.
- Keyed `{#each}`, and never the index as a key where identity matters.
- A form seeds its `$state` from props once and is re-seeded by keying it on the entity's `updatedAt`. Syncing props into state with an effect would discard what the user is typing every time a mutation invalidates.
- No legacy syntax: no `export let`, `$:`, `<slot>`, `on:click` or `use:`.

### Errors
- Reads throw; writes go through `runMutation`, which never throws and reports through a toast.
- `messageOf` is the one place a command failure is unwrapped, and `errorKind` the one place its kind is read.
- Every join can miss, because entities are soft-deleted and then never returned. A join renders the missing case and keeps the id.

### Layout
- Mobile first. The shell is a bottom tab bar; there is no sidebar and no keyboard-only affordance.
- Anything anchored to the bottom of the viewport accounts for `env(safe-area-inset-bottom)`.
- A row that holds a title and actions lets the actions wrap to their own line rather than squeezing the title, because the phone viewport is the narrow case, not the exception.

### Tooling
- `svelte-check` is the contract check, and `bun test src` covers `lib/domain`.
- There is no formatter or linter for the frontend. Match the file you are editing.

## General
- Comments clarify what the code cannot state itself: intent, constraints, surprises. A comment that restates the code is discouraged. Module-level comments stay required where the role is not obvious.
- Docs under `docs/` stay concise and current with the code.
- A decision that was deliberately not taken belongs in `docs/todo.md`, with what happens today and what would trigger the change.
