# Development Instructions

## Project Structure

```
cli/                    Rust CLI source (the aibox binary)
  src/
    main.rs             Entry point, tracing setup, dispatch
    cli.rs              clap derive-based arg parsing
    config.rs           aibox.toml deserialization (serde + toml)
    generate.rs         Dockerfile / compose / devcontainer.json generation
    runtime.rs          podman / docker abstraction
    container.rs        build / start / stop / attach / status / init
    context.rs          context scaffolding + gitignore + doctor helpers
    doctor.rs           diagnostic checks + migration artifacts
    update.rs           registry version checking (GHCR + GitHub releases)
    seed.rs             .aibox-home/ directory seeding
    audio.rs            host-side PulseAudio diagnostics and setup
    output.rs           ANSI-colored terminal output
    templates/          embedded Jinja2 templates (Dockerfile.j2, docker-compose.yml.j2)
images/                 Published images for downstream projects (8 flavors)
templates/              Work process flavor templates (4 flavors, 21 files)
schemas/                Context schema documents (versioned)
docs/                   MkDocs documentation source
scripts/                Build/release/maintenance scripts
```

## Building

```bash
cd cli && cargo build           # Debug build
cd cli && cargo build --release # Release build
```

## Testing

```bash
cd cli && cargo test                          # All tests (unit + integration + E2E tier 1)
cd cli && cargo test --features e2e           # Include E2E tier 2 (requires aibox-e2e-testrunner)
cd cli && cargo clippy -- -D warnings         # Lint check
cd cli && cargo fmt -- --check                # Format check
```

### E2E Test Architecture

Two-tier E2E testing (`cli/tests/e2e/`):

- **Tier 1** (always run): Config coverage tests and appearance tests that verify
  `aibox init` + `aibox sync` produce correct generated files. No container needed.
- **Tier 2** (`--features e2e`): Full lifecycle tests that run on the `aibox-e2e-testrunner`
  companion container via SSH. Tests init→sync→start→stop→remove, reset/backup,
  migration, addon management, and application smoke tests.

The `aibox-e2e-testrunner` service is defined in `.devcontainer/docker-compose.yml` and starts
alongside the dev-container. It runs podman (rootless) for container operations and
sshd for remote test execution. No shared volumes — the test harness deploys
artifacts via SCP, making the companion a realistic simulation of a user's machine.

### Running E2E Tier 2 Tests (step by step)

Prerequisites: dev-container must be rebuilt so the `aibox-e2e-testrunner` companion service
is running alongside it. This only needs to happen once (or when `Dockerfile.e2e`
changes).

```bash
# 1. Build the CLI binary (inside the dev-container)
cd /workspace/cli && cargo build

# 2. Run E2E Tier 2 tests
#    On first invocation, the test harness automatically:
#    - SCPs cli/target/debug/aibox → aibox-e2e-testrunner:/usr/local/bin/aibox
#    - SCPs addons/ → aibox-e2e-testrunner:/opt/aibox/addons/
#    - Verifies the deployed binary runs on the companion
#    Then executes all Tier 2 tests over SSH.
cd /workspace/cli && cargo test --features e2e

# To run a specific E2E test:
cd /workspace/cli && cargo test --features e2e -- lifecycle
```

The deploy step is guarded by `std::sync::Once` — it runs exactly once per
`cargo test` invocation, so re-running tests is fast. If you change the CLI
code, just `cargo build` again and re-run; the next `cargo test --features e2e`
will SCP the updated binary.

### Key Files

- `.devcontainer/Dockerfile.e2e` — Companion container image (debian + podman + sshd)
- `.devcontainer/ssh-e2e/` — Pre-seeded ed25519 test SSH keys
- `cli/tests/e2e/runner.rs` — SSH+SCP test harness (`E2eRunner`)
- `cli/tests/e2e/mock_runtime.rs` — Mock docker/podman for command validation
- `cli/tests/e2e/infra/mock-docker.sh` / `mock-podman.sh` — Mock runtime scripts
- `cli/tests/e2e/lifecycle.rs` — Container lifecycle tests
- `cli/tests/e2e/reset.rs` — Reset/backup tests
- `cli/tests/e2e/addon.rs` — Addon management tests
- `cli/tests/e2e/smoke.rs` — Application smoke tests (podman validation)
- `cli/tests/e2e/appearance.rs` — Theme/prompt rendering tests (Tier 1)
- `cli/tests/e2e/config_coverage.rs` — aibox.toml settings coverage (Tier 1)

## Config Spec — aibox.toml

`aibox.toml` is the single source of truth. All generated files derive from it.

Key sections:
- `[aibox]` — version, image flavor, process flavor
- `[container]` — name, hostname, user, ports, extra_packages, extra_volumes, environment
- `[context]` — schema_version
- `[ai]` — providers (claude, etc.)
- `[audio]` — enabled, pulse_server

## Docker Image Architecture

8 images built from `images/`:
- **base** — debian:trixie-slim + zellij + vim + git + lazygit + gh + claude CLI + audio + unzip
- **python** — FROM base + python 3.13 + uv + mkdocs-material
- **latex** — FROM base + TeX Live (multi-stage CTAN install)
- **typst** — FROM base + Typst (static musl binary)
- **rust** — FROM base + rustup + cargo + clippy + rustfmt
- **python-latex**, **python-typst**, **rust-latex** — combinations

TeX Live uses multi-stage build: builder installs from CTAN (~2GB), runtime copies the tree.

## Container User Support

The `container.user` config controls:
- Mount paths inside container (e.g., `/root/.vim` vs `/home/user/.vim`)
- `remoteUser` in devcontainer.json
- `GIT_CONFIG_GLOBAL` env var path
