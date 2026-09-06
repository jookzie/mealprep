# Code Standards

## Baseline
Go code follows the [Google Go Style Guide](https://google.github.io/styleguide/go/):
[Style Guide](https://google.github.io/styleguide/go/guide) (canonical),
[Style Decisions](https://google.github.io/styleguide/go/decisions) (normative),
[Best Practices](https://google.github.io/styleguide/go/best-practices) (advisory).

Whatever it settles is settled here. The sections below are this project's additions
and, where marked, its overrides.

## Go

### Packages
- All code lives under `internal/`.
- `handler`, `service`, `repository` and `client` are containers. Each holds either its implementation directly or sub-packages split by category.
- Code in a container package is what its sub-packages share — common types and helpers. A single implementation's detail belongs in a sub-package, not in the container.
- `handler`, `service` and `repository` split by domain: `handler/meal`, `service/food`.
- `repository` nests the database implementation first, then the domain: `repository/sqlite/meal`.
- `client` splits by the API it talks to: `client/openfoodfacts`.
- Every package carries a package comment stating its role.

### Files
- Small and single-purpose.
- A file is named after the main function or type it hosts, and after the concept its code represents when it hosts neither.

### Constructors
- A package that has to be constructed before it can be used holds a `new.go`, and that file holds `New`.
- `New` returns the struct, or the struct and an error, and validates its input before returning it.
- Up to two inputs are plain parameters. From three on they are grouped into a `Config` struct.

### Generated code
- `api/openapi.yaml` is the contract. Handlers implement the generated `ServerInterface`; routes are never declared by hand.
- Generated files end in `.gen.go` and are edited only by regenerating them: `mise run openapi:gen` for the API, `mise run sqlc:gen` for the data access layer.
- SQL is the source of the data access layer. Queries live in `backend/sqlc/sqlite/queries`, the schema beside them, and nothing outside `repository/sqlite` imports the generated `db` package.
- The generated `ServerInterface` is the one interface a provider package may hand out, because the spec, not the package, defines it.

### Handlers
- A handler takes a `<Operation>Request` struct, unless its whole input is a single primitive.
- A handler answers with a `<Operation>Response` struct. Never a bare domain value, a raw array or a map.
- These DTOs exist to keep the wire shape explicit and independent of the domain types, so the schemas in `api/openapi.yaml` carry the suffixes and the generator produces them.
- The handler function builds its response inline, field by field, from the domain value. No mapper helpers, no constructor methods, no shared conversion layer: the mapping is visible where the response is returned.
- Domain types never reach the wire.

### Wiring
- `cmd/app/main.go` carries no logic beyond starting and stopping the application.
- Wiring and route registration happen in `application.New`. There is no separate registration step.
- `application` holds one private struct per layer — `clients`, `repositories`, `services`, `handlers` — each in the file it is named after, each with a `from` method taking the layer below.
- `New` declares all four in one `var` block and calls `from` in order, then registers the handlers.
- A layer depends on the one below through an interface the consumer declares, never on a concrete type.
- Routes hang off a versioned group.
- Liveness and readiness come from Fiber's `middleware/healthcheck`, never a hand-rolled route.

### Types
- Use a pointer only where one is necessary: state that must not be copied (a `sync.Mutex`, a connection pool), mutation through the receiver, or a type that is nilable by design. Overrides [Receiver type](https://google.github.io/styleguide/go/decisions#receiver-type), which leaves the choice open for large structs.
- A package that provides behaviour returns concrete types. It never declares an interface for its own implementation, and no package exists to hold interfaces for others ([Interfaces](https://google.github.io/styleguide/go/decisions#interfaces)).

### Entities
- Every persisted entity carries `created_at`, `updated_at` and `deleted_at`.
- Deletes are soft: `deleted_at` is set and the row stays. Nothing removes rows.
- Reads exclude soft-deleted rows unless a caller explicitly asks for them.

### Configuration
- One `Config` struct, parsed with `caarlos0/env` in `application`. No `os.Getenv` elsewhere.
- Every setting is required. No field carries a `required` tag.
- A setting is made optional by giving it an `envDefault`, which supplies its fallback. That is the only use for `envDefault`; never add one to a setting that must come from the environment.
- `.env.example` lists every setting. `.env` is local and untracked.

### Logging
- One global `zerolog`. No per-package loggers.
- Log through `log.Info()`, `log.Error()` and friends. Never `fmt.Println`.

### Errors
- Errors are created in clients, repositories and services. Handlers create none; they translate.
- A handler turns a service error into a status with a `switch` over the error, ending in a catch-all that answers 500.
- Errors are returned, not logged, until the handler.
- Sentinels are declared at the top of the file that returns them, one per error path, prefixed `Err` (`ErrMealNotFound`).
- A sentinel is joined with its cause via `errors.Join` at exactly one point in the code.
- `internal/errorx` holds the structured error types shared across layers.
- Prefer sentinels and `errorx` types to wrapping with `fmt.Errorf("...: %w", err)`.

### Tooling
- The tree stays formatted and lint-clean. `mise` holds the tasks, and the pre-commit hook runs them.

## Frontend

### Baseline
Svelte 5 with runes, following
[Svelte best practices](https://svelte.dev/docs/svelte/best-practices). Whatever it
settles is settled here. TypeScript in strict mode.

### Structure
- `src/routes/` holds one directory per screen; `src/lib/` holds what they share.
- `lib/api` is the only path to the backend. `lib/domain` is pure logic — no Svelte, no I/O. `lib/components` splits by domain, the way `handler` and `service` do.
- `lib/components/ui` is shadcn-svelte's output, and `lib/utils.ts` comes with it. Neither is edited by hand, and the linter skips both.

### Files
- Small and single-purpose, named after the component or concept they host.
- Kebab-case, matching shadcn-svelte's own output.

### Routes
- A read is a `load` in `+page.ts`, never a `fetch` inside a component.
- What selects the read — a week, a search query — lives in the URL. `url` is already a tracked dependency, so navigating re-runs the load and the result is shareable.
- Create and edit are routes, not dialogs: every write is a full-replacement `PUT`, so an editor has to load the entity first. Dialogs are for single-field writes.
- A failing read throws and lands in `+error.svelte`.

### Components
- No component calls the API. Forms take a draft and an `onSubmit`; tables take rows and callbacks. Routes own all I/O.
- UI comes from shadcn-svelte; no ad-hoc component library.
- A component that carries a binding requirement — the Open Food Facts credit, the target bars — is the single place that requirement is implemented, so it cannot be half-applied.

### State
- `$derived` over `$effect`. Effects are an escape hatch.
- Classes or plain `$state` fields; never stores.
- Keyed `{#each}`, and never the index as a key where identity matters.
- A form seeds its `$state` from props once, through `untrack`, and is re-seeded by keying it on the entity's `updatedAt`. Syncing props into state with an effect would discard what the user is typing every time a mutation invalidates.
- No legacy syntax: no `export let`, `$:`, `<slot>`, `on:click` or `use:`.

### Generated code
- `src/lib/api/gen` is generated from `api/openapi.yaml` and committed, like the `*.gen.go` files. Regenerate with `mise run openapi:gen`; never edit it.
- Nothing outside `src/lib/api` imports from `gen/`.
- The spec is linted before it generates anything: `mise run openapi:lint`.

### Errors
- The API answers every failure with `ErrorResponse`. `messageOf` is the one place that is unwrapped.
- Reads throw; writes go through `runMutation`, which never throws and reports through a toast.
- An absent singleton is not a failure: `GET /targets` answering 404 becomes `null` in `loadTargets`, in one place.
- Every client-side join can miss, because entities are soft-deleted and then never returned. A join renders the missing case and keeps the id.

### Tooling
- `biome` formats and lints, `bun test` covers `lib/domain`, `svelte-check` is the contract check, and Playwright covers the critical paths end to end.

## General
- Comments clarify what the code cannot state itself: intent, constraints, surprises. A comment that restates the code is discouraged. Overrides [Doc comments](https://google.github.io/styleguide/go/decisions#doc-comments), which asks for one on every exported name; package comments stay required.
- Docs under `docs/` stay concise and current with the code.
