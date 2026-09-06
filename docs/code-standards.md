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
- Split a container as soon as it holds more than one implementation, resource or domain.
- Every package carries a package comment stating its role.

### Files
- Small and single-purpose.
- A file is named after the main function or type it hosts, and after the concept its code represents when it hosts neither.

### Constructors
- A package that has to be constructed before it can be used holds a `new.go`, and that file holds `New`.
- `New` returns the struct, or the struct and an error, and validates its input before returning it.
- Up to two inputs are plain parameters. From three on they are grouped into a `Config` struct.

### Wiring
- `cmd/app/main.go` carries no logic beyond starting and stopping the application.
- Wiring and route registration happen in `application.New`. There is no separate registration step.
- Routes hang off a versioned group.
- Liveness and readiness come from Fiber's `middleware/healthcheck`, never a hand-rolled route.

### Types
- Use a pointer only where one is necessary: state that must not be copied (a `sync.Mutex`, a connection pool), mutation through the receiver, or a type that is nilable by design. Overrides [Receiver type](https://google.github.io/styleguide/go/decisions#receiver-type), which leaves the choice open for large structs.
- A package that provides behaviour returns concrete types. It never declares an interface for its own implementation, and no package exists to hold interfaces for others ([Interfaces](https://google.github.io/styleguide/go/decisions#interfaces)).

### Configuration
- One `Config` struct, parsed with `caarlos0/env` in `application`. No `os.Getenv` elsewhere.
- Every setting is required. No field carries a `required` tag.
- A setting is made optional by giving it an `envDefault`, which supplies its fallback. That is the only use for `envDefault`; never add one to a setting that must come from the environment.
- `.env.example` lists every setting. `.env` is local and untracked.

### Logging
- One global `zerolog`. No per-package loggers.
- Log through `log.Info()`, `log.Error()` and friends. Never `fmt.Println`.

### Errors
- Errors are returned, not logged, until the handler.
- Sentinels are declared at the top of the file that returns them, one per error path, prefixed `Err` (`ErrMealNotFound`).
- A sentinel is joined with its cause via `errors.Join` at exactly one point in the code.
- `internal/errorx` holds the structured error types shared across layers.
- Prefer sentinels and `errorx` types to wrapping with `fmt.Errorf("...: %w", err)`.

### Tooling
- The tree stays formatted and lint-clean. `mise` holds the tasks, and the pre-commit hook runs them.

## Frontend
- Svelte + TypeScript, strict mode.
- UI components from shadcn-svelte; no ad-hoc component library.
- API calls live in one client module; components never call `fetch` directly.
- Types mirror the backend JSON contracts.

## General
- Comments clarify what the code cannot state itself: intent, constraints, surprises. A comment that restates the code is discouraged. Overrides [Doc comments](https://google.github.io/styleguide/go/decisions#doc-comments), which asks for one on every exported name; package comments stay required.
- Docs under `docs/` stay concise and current with the code.
