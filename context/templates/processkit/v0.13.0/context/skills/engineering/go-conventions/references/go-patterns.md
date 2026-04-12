# Go Patterns

Common Go patterns with code examples for reference.

## Functional Options

The functional options pattern provides clean, extensible configuration for constructors
without breaking API compatibility when adding new options.

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

func WithLogger(l *slog.Logger) Option {
    return func(s *Server) { s.logger = l }
}

func NewServer(addr string, opts ...Option) *Server {
    s := &Server{
        addr:    addr,
        timeout: 30 * time.Second,         // sensible default
        logger:  slog.Default(),
    }
    for _, opt := range opts {
        opt(s)
    }
    return s
}

// Usage:
// srv := NewServer(":8080", WithTimeout(10*time.Second), WithLogger(myLogger))
```

When to use: constructors with 3+ optional parameters, public APIs where backward
compatibility matters.

## Middleware Chain

HTTP middleware follows a consistent signature for composable request processing.

```go
type Middleware func(http.Handler) http.Handler

func Logging(logger *slog.Logger) Middleware {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            start := time.Now()
            next.ServeHTTP(w, r)
            logger.Info("request", "method", r.Method, "path", r.URL.Path,
                "duration", time.Since(start))
        })
    }
}

func Recovery() Middleware {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            defer func() {
                if err := recover(); err != nil {
                    http.Error(w, "internal error", http.StatusInternalServerError)
                }
            }()
            next.ServeHTTP(w, r)
        })
    }
}

// Chain applies middleware in order: first listed = outermost
func Chain(h http.Handler, mw ...Middleware) http.Handler {
    for i := len(mw) - 1; i >= 0; i-- {
        h = mw[i](h)
    }
    return h
}

// Usage:
// handler := Chain(myHandler, Logging(logger), Recovery())
```

## Worker Pool with Errgroup

Fan-out/fan-in pattern using `errgroup` for controlled concurrency.

```go
func ProcessItems(ctx context.Context, items []Item, workers int) error {
    g, ctx := errgroup.WithContext(ctx)
    jobs := make(chan Item, workers)

    // Producer: feed items into the channel
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

    // Workers: process items concurrently
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

    return g.Wait() // returns first error, cancels remaining work
}
```

Key points: `errgroup.WithContext` cancels all goroutines on first error. Size the
channel buffer to the number of workers. Always select on `ctx.Done()` in the producer.

## Graceful Shutdown

Clean shutdown pattern for HTTP servers that finishes in-flight requests.

```go
func run(ctx context.Context) error {
    srv := &http.Server{Addr: ":8080", Handler: mux}

    // Start server in background
    errCh := make(chan error, 1)
    go func() { errCh <- srv.ListenAndServe() }()

    // Wait for interrupt or server error
    select {
    case err := <-errCh:
        return err // server failed to start
    case <-ctx.Done():
    }

    // Graceful shutdown with timeout
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

Pattern: `signal.NotifyContext` for OS signals, `srv.Shutdown` for draining connections,
timeout on shutdown to avoid hanging forever.

## Table-Driven Tests

Standard Go testing pattern for exhaustive input/output coverage.

```go
func TestParseSize(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    int64
        wantErr bool
    }{
        {name: "bytes",          input: "100B",  want: 100},
        {name: "kilobytes",      input: "2KB",   want: 2048},
        {name: "megabytes",      input: "1MB",   want: 1048576},
        {name: "empty string",   input: "",       wantErr: true},
        {name: "invalid suffix", input: "10XB",   wantErr: true},
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

Always use `t.Run` with named subtests. Use `t.Parallel()` when tests are independent.
Put the expected error case last for readability.
