# Code Standards

## Go
- `cmd/app/main.go` only: build `application`, call `Start()`, call shutdown on SIGINT. No other logic.
- All code lives under `internal/` in: `application`, `client`, `repository`, `service`, `handler`.
- `application` wires dependencies, resolves handlers, registers them to Fiber, runs Fiber in `Start()`.
- Config: one struct parsed with `caarlos0/env` inside `application/`. No `os.Getenv` elsewhere.
- Logging: global `zerolog` configured in `application.New`. Use `log.Info()`, `log.Error()`, etc.; never `fmt.Println`.
- Repositories expose interfaces; SQLite types implement them. Services depend on interfaces only.
- Keep unused packages as placeholders (a `doc.go` or empty file). Don't implement speculatively.
- Errors are returned, not logged, until the handler.

### Error handling
- Sentinel errors are defined at the top of the file that uses them, one per error path.
- Error variables are always prefixed with `Err` (e.g. `ErrMealNotFound`).
- A sentinel is joined with another error (`errors.Join`) at exactly one point in the code.
- Wrapping with context via `fmt.Errorf("...: %w", err)` is discouraged. Use sentinels and `internal/errorx` types instead.
- `internal/errorx` holds structured error types (e.g. `ErrNotFound`) that carry resource, id and the internal error.
- `gofmt` + `go vet` clean.

## Frontend
- Svelte + TypeScript, strict mode.
- UI components from shadcn-svelte; no ad-hoc component library.
- API calls in one client module; components never call `fetch` directly.
- Types mirror backend JSON contracts.

## General
- Small, single-purpose files. Names describe intent.
- Docs under `docs/` stay concise and current with the code.
