# Code Standards

## Go
- `cmd/app/main.go` only: build `application`, call `Start()`, call shutdown on SIGINT. No other logic.
- All code lives under `internal/` in: `application`, `client`, `repository`, `service`, `handler`.
- `application` wires dependencies, resolves handlers, registers them to Fiber, runs Fiber in `Start()`.
- Config: one struct parsed with `caarlos0/env` inside `application/`. No `os.Getenv` elsewhere.
- Logging: global `zerolog` configured in `application.New`. Use `log.Info()`, `log.Error()`, etc.; never `fmt.Println`.
- Repositories expose interfaces; SQLite types implement them. Services depend on interfaces only.
- Keep unused packages as placeholders (a `doc.go` or empty file). Don't implement speculatively.
- Errors are wrapped with context (`fmt.Errorf("...: %w", err)`) and returned, not logged, until the handler.
- `gofmt` + `go vet` clean.

## Frontend
- Svelte + TypeScript, strict mode.
- UI components from shadcn-svelte; no ad-hoc component library.
- API calls in one client module; components never call `fetch` directly.
- Types mirror backend JSON contracts.

## General
- Small, single-purpose files. Names describe intent.
- Docs under `docs/` stay concise and current with the code.
