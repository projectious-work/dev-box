# Contributing to aibox

## Prerequisites

- Rust stable toolchain (`rustup`)
- `cargo` on PATH
- The dev container (`.devcontainer/`) is the recommended environment — it includes
  the cross-compiler for x86_64 and all release tooling.

## Build

```bash
cd cli && cargo build           # debug build
cd cli && cargo build --release # release build
```

## Test

```bash
# All tests (unit + integration + E2E tier 1 — no container needed)
cd cli && cargo test

# Lint — zero warnings required
cd cli && cargo clippy --all-targets -- -D warnings

# Format check
cd cli && cargo fmt -- --check
```

### E2E Tier 2 (full container lifecycle tests)

Requires the `aibox-e2e-testrunner` companion service running alongside the devcontainer.
Rebuild the devcontainer once if needed (or when `Dockerfile.e2e` changes).

```bash
# 1. Build the CLI binary
cd /workspace/cli && cargo build

# 2. Run E2E tier 2 tests (deploys binary to companion via SCP on first run)
cd /workspace/cli && cargo test --features e2e

# Run a specific E2E test
cd /workspace/cli && cargo test --features e2e -- lifecycle
```

The deploy step is guarded by `std::sync::Once` — runs once per `cargo test` invocation.
Re-running after a code change: `cargo build` again, then re-run `cargo test --features e2e`.

## Before committing

```bash
cd cli && cargo test && cargo clippy --all-targets -- -D warnings
```

Both must be clean. `cargo audit` must also be clean before tagging a release.

## Commit message format

Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`.
Always reference GitHub issue numbers: `fixes #N`, `refs #N`.
Include `Cargo.lock` in version bump commits.

## Release

See `context/notes/NOTE-20260411_0000-LoyalSpruce-aibox-release-process.md` for the full release process.
Quick summary: `./scripts/maintain.sh release X.Y.Z` (in container) then
`./scripts/maintain.sh release-host X.Y.Z` (on macOS host).

## Repository layout

| Path | Owns |
|------|------|
| `cli/` | The Rust CLI — the only shipped artifact besides addon YAMLs |
| `addons/` | YAML addon definitions (python, rust, node, latex, …) |
| `images/` | Container image build recipes published to GHCR |
| `docs-site/` | Docusaurus documentation site |
| `scripts/` | Release and maintenance tooling |
| `context/` | This project's context (workitems, decisions, notes, …) |

**Key Rust modules:**

| Module | Responsibility |
|--------|---------------|
| `cli/src/main.rs` | Entry point, tracing setup, dispatch |
| `cli/src/cli.rs` | clap derive-based arg parsing |
| `cli/src/config.rs` | `aibox.toml` deserialization (serde + toml) |
| `cli/src/generate.rs` | Dockerfile / compose / devcontainer.json generation |
| `cli/src/container.rs` | `init` / `start` / `stop` / `sync` / `status` |
| `cli/src/content_source.rs` | processkit release-asset fetcher with fallback strategies |
| `cli/src/content_install.rs` | Install map — where each processkit file lands |
| `cli/src/content_init.rs` | `install_content_source` orchestration; templates mirror |
| `cli/src/content_diff.rs` | Three-way diff; migration document generation |
| `cli/src/mcp_registration.rs` | Per-harness MCP server registration |
| `cli/src/processkit_vocab.rs` | **Central constants module** — all processkit vocabulary |
| `cli/src/addon_loader.rs` | YAML addon loading and template context building |
| `cli/src/seed.rs` | `.aibox-home/` runtime config seed |
| `cli/src/doctor.rs` | Diagnostic checks |
| `cli/src/context.rs` | Project skeleton scaffolding, gitignore, provider thin pointers |

**Rule:** Never hardcode processkit path strings, filenames, or vocabulary in production
Rust source — add constants to `processkit_vocab.rs` instead.

## Critical distinction

**We are in a dev-container building dev-containers.**

- **`.devcontainer/`** — THIS project's dev environment (Rust + Python/uv + Docusaurus).
- **`images/`** — Published images for OTHER projects (pushed to GHCR).

Never confuse these two. Changes to `.devcontainer/` affect our development.
Changes to `images/` affect downstream projects.
