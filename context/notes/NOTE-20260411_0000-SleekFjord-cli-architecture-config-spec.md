---
id: NOTE-20260411_0000-SleekFjord-cli-architecture-config-spec
title: "CLI Architecture — Config Spec, Image Flavors, E2E Test Architecture"
type: reference
status: permanent
created: 2026-04-11T00:00:00Z
tags: [cli, architecture, config, images, e2e, testing]
skill: software-architecture
---

# CLI Architecture Reference

Technical reference for the aibox CLI. See `CONTRIBUTING.md` for build/test workflow.

## Config Spec — aibox.toml

`aibox.toml` is the single source of truth. All generated files derive from it.

Key sections:
- `[aibox]` — version, base image flavor
- `[container]` — name, hostname, user, ports, extra_packages, extra_volumes, environment, keepalive, post_create_command
- `[context]` — schema_version, packages
- `[processkit]` — source, version (pin), branch
- `[skills]` — include/exclude lists for processkit skill filtering
- `[ai]` — providers (claude, gemini, codex, copilot, continue, aider, mistral)
- `[addons.X]` — per-addon config with `[addons.X.tools]` for tool version management
- `[customization]` — theme, prompt, fonts
- `[audio]` — enabled, pulse_server
- `[agents]` — agent-specific configuration

## Docker Image Architecture

8 images built from `images/`:

| Image | Base | Added |
|-------|------|-------|
| `base-debian` | debian:trixie-slim | zellij, vim, git, lazygit, gh, claude CLI, audio, unzip, python3, uv |
| `python` | base-debian | poetry, pdm, additional Python tooling |
| `latex` | base-debian | TeX Live (multi-stage CTAN install, ~2GB builder → slim runtime copy) |
| `typst` | base-debian | Typst (static musl binary) |
| `rust` | base-debian | rustup, cargo, clippy, rustfmt |
| `python-latex` | python | + latex |
| `python-typst` | python | + typst |
| `rust-latex` | rust | + latex |

TeX Live uses multi-stage build: builder installs from CTAN (~2GB), runtime copies the tree.

**python3 + uv in base:** All base images include python3 (Debian Trixie, 3.13.x) and uv
unconditionally — required for processkit MCP servers (PEP 723 scripts). See DEC-20260411_0000-TallDawn-python3-uv-unconditionally-in.

## Container User Support

`container.user` controls:
- Mount paths inside container (e.g. `/root/.vim` vs `/home/user/.vim`)
- `remoteUser` in `devcontainer.json`
- `GIT_CONFIG_GLOBAL` env var path

## E2E Test Architecture

Two-tier E2E testing in `cli/tests/e2e/`:

**Tier 1** (always run, no container needed): Config coverage tests (`config_coverage.rs`)
and appearance tests (`appearance.rs`) that verify `aibox init` + `aibox sync` produce
correct generated files.

**Tier 2** (`--features e2e`): Full lifecycle tests on the `aibox-e2e-testrunner`
companion container via SSH. Runs init→sync→start→stop→remove, reset/backup, migration,
addon management, and application smoke tests. The companion runs podman (rootless)
for container operations and sshd for remote test execution. No shared volumes —
the harness deploys artifacts via SCP (realistic simulation of a user's machine).

Key E2E test files:

| File | Tests |
|------|-------|
| `cli/tests/e2e/runner.rs` | SSH+SCP test harness (`E2eRunner`) |
| `cli/tests/e2e/mock_runtime.rs` | Mock docker/podman for command validation |
| `cli/tests/e2e/lifecycle.rs` | Container lifecycle tests |
| `cli/tests/e2e/reset.rs` | Reset/backup tests |
| `cli/tests/e2e/addon.rs` | Addon management tests |
| `cli/tests/e2e/smoke.rs` | Application smoke tests (podman validation) |
| `cli/tests/e2e/appearance.rs` | Theme/prompt rendering tests (Tier 1) |
| `cli/tests/e2e/config_coverage.rs` | `aibox.toml` settings coverage (Tier 1) |

The deploy step (`Once` guard) runs exactly once per `cargo test` invocation.
Subsequent test re-runs in the same session skip the SCP deploy.

## processkit Install Pipeline

```
content_source.rs      fetch release-asset tarball → verify SHA256 → extract to cache
content_install.rs     install_action_for() → Install | InstallTemplated | Skip
content_init.rs        install_content_source() → cache walk → live files
                       copy_templates_from_cache_with_vars() → templates mirror (rendered)
content_diff.rs        three-way diff: templates mirror ↔ cache ↔ live files
                       → generate migration documents for conflicts
mcp_registration.rs    regenerate_mcp_configs() → per-harness MCP config files
```

`processkit_vocab.rs` is the single source of truth for all processkit path constants,
filenames, category names, and frontmatter vocabulary. All production code references
constants from there — no hardcoded strings.
