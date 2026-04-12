---
name: go-conventions
description: |
  Go idioms — error wrapping, small interfaces, goroutine lifecycle, table-driven tests. Use when writing or reviewing Go code, designing package layouts, or working through concurrency, error handling, and testing decisions.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-go-conventions
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Go Conventions

## Intro

Idiomatic Go is small interfaces, explicit error wrapping, clear
goroutine ownership, and domain-oriented packages. Follow `gofmt`,
respect `context.Context`, and never start a goroutine without
knowing how it stops.

## Overview

### Error handling

Return errors as the last return value; never panic in library code.
Wrap with context using `fmt.Errorf("loading config: %w", err)` so
callers can unwrap with `errors.Is` (sentinel errors) or `errors.As`
(typed errors). Define sentinel errors at package level:
`var ErrNotFound = errors.New("not found")`. Use custom error types
only when callers need structured fields. Never silently ignore an
error — assigning to `_` requires a comment explaining why.

### Interfaces

Accept interfaces, return concrete types. Keep interfaces small —
1–3 methods is ideal ("the bigger the interface, the weaker the
abstraction"). Define interfaces where they are consumed, not where
they are implemented. Reach for stdlib interfaces (`io.Reader`,
`io.Writer`, `fmt.Stringer`) before inventing new ones. Name
single-method interfaces with the `-er` suffix.

### Goroutine patterns

Pass `context.Context` as the first parameter to any function that
blocks or does I/O. Use `errgroup.Group` for fan-out/fan-in — it
handles error propagation and cancellation. Every goroutine must
have a clear owner responsible for its lifecycle. Always select on
`ctx.Done()` alongside channel operations.

### Package layout

Organize by domain, not by layer: `user/`, `order/`, not `models/`,
`handlers/`. Package names are short, lowercase, and singular
(`http`, `user`, `config`). Avoid catch-all `util`, `common`, and
`helpers` packages. Keep `main.go` minimal — parse flags, wire
dependencies, call `run(ctx)`. Put implementation details under
`internal/` to block external imports.

### Testing

Use table-driven tests with named subtests for multi-case coverage.
`testify/assert` for readable checks, `testify/require` for fatal
ones. Use `httptest.NewServer` for client tests, `NewRecorder` for
handler tests. Call `t.Helper()` in helpers so failures point at the
caller. Run `go test -race ./...` in CI. Use `t.Parallel()` for
independent tests.

### Code style

`gofmt` is non-negotiable; `goimports` handles import grouping
(stdlib, blank line, third-party, blank line, internal). Exported
names get doc comments that start with the name. Use named return
values sparingly. Prefer early returns and guard clauses over deep
nesting.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Goroutines with no visible stop condition.** Starting a goroutine without a `context.Context` cancellation path or a `done` channel means it may run forever after the caller is gone. Every goroutine must have an owner and a clear termination path.
- **`panic` in library code for non-programmer-error conditions.** Library code must return errors, not panic, for any condition a caller might encounter in normal use. Reserve `panic` for programmer errors: nil pointers being dereferenced, impossible state. In application code `panic` is still rare; prefer `log.Fatal` + `os.Exit` at the `main` level.
- **Large interfaces defined alongside their only implementation.** Interfaces should be defined where they are consumed, not where they are produced. A 10-method interface defined next to its sole implementation is not an abstraction — it is added complexity with no benefit.
- **`util`, `common`, and `helpers` packages.** These names say nothing about what the code does and attract unrelated functions. Name packages by their domain and responsibility; if something doesn't fit a named package, question whether it belongs in the codebase.
- **Silently ignoring errors with `_ =`.** Discarding an error without even a comment is a bug waiting to happen. When an error genuinely cannot matter, document why: `_ = f.Close() // best-effort cleanup after write; error already reported`.
- **Implementing the same interface in the same file as the interface definition.** Go's interface satisfaction is implicit; coupling the interface definition to the implementation defeats the decoupling purpose. Keep them in separate packages.
- **`context.Background()` deep inside a call stack.** Creating a fresh `context.Background()` inside a called function discards cancellation and deadline signals from the caller. Accept `ctx context.Context` as the first parameter and propagate it.

## Full reference

### Functional options

Extensible constructors that stay backward compatible:

```go
type Server struct {
    addr    string
    timeout time.Duration
    logger  *slog.Logger
}

type Option func(*Server)

func WithTimeout(d time.Duration) Option {
    return func(s *Server) { s.timeout = d }
}

func NewServer(addr string, opts ...Option) *Server {
    s := &Server{addr: addr, timeout: 30 * time.Second, logger: slog.Default()}
    for _, opt := range opts {
        opt(s)
    }
    return s
}
```

Use when a constructor has 3+ optional parameters or when backward
compatibility matters on a public API.

### Worker pool with errgroup

```go
func ProcessItems(ctx context.Context, items []Item, workers int) error {
    g, ctx := errgroup.WithContext(ctx)
    jobs := make(chan Item, workers)

    g.Go(func() error {
        defer close(jobs)
        for _, item := range items {
            select {
            case jobs <- item:
            case <-ctx.Done():
                return ctx.Err()
            }
        }
        return nil
    })

    for range workers {
        g.Go(func() error {
            for item := range jobs {
                if err := process(ctx, item); err != nil {
                    return fmt.Errorf("processing %s: %w", item.ID, err)
                }
            }
            return nil
        })
    }
    return g.Wait()
}
```

`errgroup.WithContext` cancels all goroutines on first error. Size
the buffer to the worker count. Always select on `ctx.Done()` in the
producer.

### Graceful shutdown

```go
func run(ctx context.Context) error {
    srv := &http.Server{Addr: ":8080", Handler: mux}
    errCh := make(chan error, 1)
    go func() { errCh <- srv.ListenAndServe() }()

    select {
    case err := <-errCh:
        return err
    case <-ctx.Done():
    }

    shutdownCtx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()
    return srv.Shutdown(shutdownCtx)
}

func main() {
    ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
    defer stop()
    if err := run(ctx); err != nil && !errors.Is(err, http.ErrServerClosed) {
        slog.Error("server failed", "error", err)
        os.Exit(1)
    }
}
```

`signal.NotifyContext` for OS signals, `srv.Shutdown` to drain
in-flight requests, timeout on shutdown so it never hangs forever.

### Middleware chain

```go
type Middleware func(http.Handler) http.Handler

func Chain(h http.Handler, mw ...Middleware) http.Handler {
    for i := len(mw) - 1; i >= 0; i-- {
        h = mw[i](h)
    }
    return h
}
```

First listed is outermost. See `references/go-patterns.md` for
complete logging and recovery middleware examples.

### Table-driven tests

```go
func TestParseSize(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    int64
        wantErr bool
    }{
        {name: "bytes", input: "100B", want: 100},
        {name: "kilobytes", input: "2KB", want: 2048},
        {name: "empty string", input: "", wantErr: true},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            t.Parallel()
            got, err := ParseSize(tt.input)
            if tt.wantErr {
                require.Error(t, err)
                return
            }
            require.NoError(t, err)
            assert.Equal(t, tt.want, got)
        })
    }
}
```

Always use `t.Run` with named subtests. Put expected-error cases
last for readability.

### Go proverbs worth tattooing

- Don't communicate by sharing memory; share memory by communicating.
- A little copying is better than a little dependency.
- Clear is better than clever.
- Make the zero value useful.
- Errors are values — handle them, don't just check them.
- The empty interface says nothing.

### Anti-patterns

- Goroutines with no visible stop condition.
- `panic` in library code for non-programmer-error conditions.
- Large interfaces defined alongside their sole implementation.
- `util`/`common`/`helpers` packages that accrete unrelated code.
- Ignoring errors without a `_ =` plus justification comment.
- Naked returns in long functions where named returns obscure flow.

### Further reading

- `references/go-patterns.md` — full functional options, middleware,
  worker pool, and graceful shutdown examples.
