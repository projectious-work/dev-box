# CLAUDE.md — dev-box Hand-off Document

This document captures the key decisions, architecture, and implementation
details of the `dev-box` project for continuity in Claude Code sessions.

---

## Project Vision

**dev-box** is evolving from a single dev-container into a tool analogous to
**uv for AI work environments**. Like uv unifies Python packaging, dev-box
unifies the reproducible setup of containerized development environments with
built-in AI context structure and work process management.

The tool consists of:
1. **Published container images** — versioned base + derived flavors
2. **A Rust CLI (`dev-box`)** — manages the full lifecycle (init, build, run, doctor, update)
3. **`dev-box.toml`** — single source of truth for project configuration
4. **Context schemas** — versioned structure definitions for AI work processes
5. **MkDocs documentation** — for humans and AI agents alike

---

## Critical Architectural Distinction

**We are in a dev-container building dev-containers.**

This repo has TWO separate sets of container definitions:

- **`.devcontainer/`** — THIS project's own dev environment. It includes
  the Rust toolchain, Python/uv/MkDocs, and everything needed to develop
  the dev-box CLI and build the published images. This is what VS Code
  and `scripts/dev.sh` use to run the workspace we're coding in right now.

- **`images/`** — The published images that OTHER projects consume.
  These are the base image and 7 derived flavors that get pushed to GHCR.
  They do NOT include the Rust toolchain or MkDocs (unless a flavor needs it).

Never confuse these two. Changes to `.devcontainer/` affect our own
development experience. Changes to `images/` affect downstream projects.

---

## Repository Layout (Target)

```
<project-root>/
├── .devcontainer/              ← THIS project's dev environment
│   ├── Dockerfile              ← Rust + Python/uv + MkDocs + all dev tools
│   ├── docker-compose.yml
│   ├── devcontainer.json
│   └── config/                 ← config templates (vimrc, gitconfig, zellij/)
│
├── images/                     ← Published images for downstream projects
│   ├── base/
│   │   └── Dockerfile          ← debian trixie-slim + zellij + vim + git + gh + claude + audio
│   ├── python/
│   │   └── Dockerfile          ← FROM base + python 3.13 + uv + mkdocs-material
│   ├── latex/
│   │   └── Dockerfile          ← FROM base + TeX Live (multi-stage)
│   ├── typst/
│   │   └── Dockerfile          ← FROM base + Typst (static binary)
│   ├── rust/
│   │   └── Dockerfile          ← FROM base + rustup + cargo + clippy + rustfmt
│   ├── python-latex/
│   │   └── Dockerfile          ← FROM python + TeX Live
│   ├── python-typst/
│   │   └── Dockerfile          ← FROM python + Typst
│   └── rust-latex/
│       └── Dockerfile          ← FROM rust + TeX Live
│
├── cli/                        ← Rust CLI source (the dev-box binary)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── cli.rs              ← clap derive-based arg parsing
│       ├── config.rs           ← dev-box.toml deserialization (serde + toml)
│       ├── generate.rs         ← Dockerfile / compose / devcontainer.json generation
│       ├── runtime.rs          ← podman / docker abstraction
│       ├── container.rs        ← build / start / stop / attach / status
│       ├── context.rs          ← scaffolding + doctor
│       ├── migrate.rs          ← schema diff + migration artifact generation
│       ├── update.rs           ← registry version checking
│       ├── seed.rs             ← .root/ directory seeding
│       └── templates/          ← embedded templates (minijinja)
│           ├── Dockerfile.j2
│           ├── docker-compose.yml.j2
│           ├── devcontainer.json.j2
│           ├── CLAUDE.md.j2
│           └── context/        ← per-work-process-flavor templates
│
├── schemas/                    ← Context schema documents (versioned)
│   └── v1.0.0/
│       └── context-schema.md   ← AI-consumable structure definition + migration prompt
│
├── templates/                  ← Work process flavor templates
│   ├── minimal/               ← CLAUDE.md only
│   ├── managed/               ← backlog, decisions, standups, owner, team
│   ├── research/              ← progress, research notes, analysis
│   └── product/               ← PRD, backlog, projects, standups, work-instructions
│
├── docs/                       ← MkDocs documentation source
│   ├── index.md
│   ├── getting-started/
│   │   ├── installation.md
│   │   ├── new-project.md
│   │   └── existing-project.md
│   ├── container/
│   │   ├── base-image.md
│   │   ├── texlive-image.md
│   │   ├── features.md
│   │   └── audio.md
│   ├── context/
│   │   ├── overview.md
│   │   ├── work-processes.md
│   │   └── migration.md
│   ├── cli/
│   │   ├── commands.md
│   │   └── configuration.md
│   └── changelog.md
│
├── scripts/
│   └── dev.sh                  ← LEGACY — kept during transition, replaced by CLI
│
├── .root/                      ← host-side persisted config (NOT tracked in git)
├── mkdocs.yml
├── CLAUDE.md                   ← this file
├── README.md
└── .gitignore
```

---

## dev-box.toml Specification

`dev-box.toml` is the single source of truth for any project using dev-box.
All generated files (Dockerfile, docker-compose.yml, devcontainer.json) derive
from it. The generated files are artifacts, not sources.

```toml
[dev-box]
version = "1.0.0"                     # dev-box release this project targets
image = "python-latex"                 # one of: base, python, latex, typst, rust, python-latex, python-typst, rust-latex
process = "product"                    # one of: minimal, managed, research, product

[container]
name = "my-project"                    # container_name in compose
hostname = "my-project"               # container hostname
ports = []                             # forwarded ports
extra_packages = []                    # additional apt packages for thin overlay Dockerfile
extra_volumes = []                     # additional bind mounts [{source, target, read_only}]
environment = {}                       # additional env vars beyond base set

[context]
owner = "~/.config/dev-box/OWNER.md"  # path to shared OWNER.md (symlinked into context/)
schema_version = "1.0.0"              # context schema version

[audio]
enabled = true                         # include PulseAudio bridging
pulse_server = "tcp:host.docker.internal:4714"
```

### What `dev-box generate` produces from this

1. `.devcontainer/Dockerfile` — `FROM ghcr.io/.../dev-box:<image>-v<version>` + extra_packages
2. `.devcontainer/docker-compose.yml` — all base mounts + audio + extra_volumes + environment
3. `.devcontainer/devcontainer.json` — VS Code settings appropriate for the image flavor
4. `.dev-box-version` — lockfile recording exact versions

Generated files carry a header comment: `# Generated by dev-box — do not edit. Modify dev-box.toml instead.`

---

## Published Images

Eight images built from `images/` and published to GHCR:

| Image | Tag Pattern | Base | Adds |
|-------|------------|------|------|
| base | `base-vX.Y.Z` | debian:trixie-slim | zellij, vim, git, lazygit, gh, claude CLI, audio (sox, pulseaudio-utils), curl, jq, less, bash-completion |
| python | `python-vX.Y.Z` | base | python 3.13, uv, mkdocs-material |
| latex | `latex-vX.Y.Z` | base | TeX Live (multi-stage CTAN install), poppler-utils, inkscape, fontconfig, latexindent perl deps |
| typst | `typst-vX.Y.Z` | base | Typst (static musl binary from GitHub releases) |
| rust | `rust-vX.Y.Z` | base | rustup, cargo, clippy, rustfmt |
| python-latex | `python-latex-vX.Y.Z` | python | TeX Live (same multi-stage pattern) |
| python-typst | `python-typst-vX.Y.Z` | python | Typst (same static binary pattern) |
| rust-latex | `rust-latex-vX.Y.Z` | rust | TeX Live (same multi-stage pattern) |

### TeX Live Build Strategy

TeX Live requires a multi-stage build (~2GB install from CTAN, builder stage
discarded). For images that combine TeX Live with another flavor (python-latex,
rust-latex), the Dockerfile uses a TeX Live builder stage and copies the tree
into the flavor's runtime stage:

```dockerfile
FROM debian:trixie-slim AS texlive-builder
# ... install TeX Live from CTAN ...

FROM ghcr.io/.../dev-box:python-vX.Y.Z AS runtime
COPY --from=texlive-builder /usr/local/texlive /usr/local/texlive
# ... symlinks, poppler-utils, inkscape, etc.
```

### Audio Support (Base Image)

All images include PulseAudio bridging for Claude Code voice:
- **Packages:** `sox`, `pulseaudio-utils` (paplay, parecord, client libs)
- **Compose env:** `PULSE_SERVER`, `AUDIODRIVER=pulseaudio`
- **Compose mount:** `.root/.asoundrc` → `/root/.asoundrc` (read-only)
- **Config seeding:** default `.asoundrc` created if missing

`host.docker.internal` works on Docker Desktop (macOS/Windows) and Podman
with pasta networking. On bare Linux Docker, may need `--add-host` — the
CLI can detect and adjust.

---

## Rust CLI — `dev-box`

### Design Principles (inspired by uv)

- **Single static binary.** No runtime dependencies. Distributed via GitHub
  releases — `curl -fsSL .../install.sh | sh` (same pattern as uv, zellij).
- **`dev-box.toml` is the primary input.** Every command reads it. Generated
  files are downstream artifacts.
- **Fast.** Sub-millisecond cold start. No interpreter overhead.
- **Cross-platform.** CI builds for linux-amd64, linux-arm64, macos-amd64,
  macos-arm64.

### Commands

| Command | Purpose |
|---------|---------|
| `dev-box init` | Interactive setup → creates `dev-box.toml`, generates files, scaffolds context |
| `dev-box generate` | Re-generates devcontainer files from `dev-box.toml` |
| `dev-box build [--no-cache]` | Builds the container image |
| `dev-box start` | Ensure container running + attach via zellij |
| `dev-box stop` | Stop container |
| `dev-box attach` | Attach to running container |
| `dev-box status` | Report container state |
| `dev-box doctor` | Validate context, produce migration artifacts |
| `dev-box update [--check]` | Check for / apply version updates |

### Crate Dependencies

| Need | Crate |
|------|-------|
| CLI parsing | `clap` (derive mode) |
| TOML parsing | `toml` + `serde` |
| YAML generation | `serde_yaml` (for compose output) |
| JSON generation | `serde_json` (for devcontainer.json output) |
| Template rendering | `minijinja` (for scaffolding) |
| Logging | `tracing` + `tracing-subscriber` (JSON output built-in) |
| Process execution | `std::process::Command` |
| Semver | `semver` |
| HTTP (registry) | `ureq` (minimal, blocking) |
| File paths | `dirs` (for ~/.config/dev-box/) |

### Config Deserialization

```rust
#[derive(Debug, Deserialize)]
struct DevBoxToml {
    dev_box: DevBoxSection,
    container: ContainerSection,
    context: ContextSection,
    audio: AudioSection,
}

#[derive(Debug, Deserialize)]
struct DevBoxSection {
    version: String,
    image: String,     // enum: base, python, latex, typst, rust, python-latex, python-typst, rust-latex
    process: String,   // enum: minimal, managed, research, product
}

// ... etc
```

---

## Work Process Flavors

Independent of container image flavors. Selected via `dev-box init --process <flavor>`.

### minimal
For simple projects. Only `CLAUDE.md` at root. No `context/` directory.

### managed
For projects with structured task tracking:
```
CLAUDE.md
context/
├── OWNER.md              ← symlinked from ~/.config/dev-box/
├── DECISIONS.md
├── BACKLOG.md
├── STANDUPS.md
└── work-instructions/
    └── GENERAL.md        ← shared rules from dev-box
```

### research
For learning/documentation projects:
```
CLAUDE.md
context/
├── OWNER.md
├── PROGRESS.md
├── research/             ← research notes
└── analysis/             ← structural analysis
```

### product
For full product development (like kaits):
```
CLAUDE.md
context/
├── OWNER.md
├── DECISIONS.md
├── BACKLOG.md
├── PROJECTS.md
├── STANDUPS.md
├── PRD.md
├── work-instructions/
│   ├── GENERAL.md        ← shared from dev-box
│   ├── DEVELOPMENT.md    ← scaffolded per image flavor
│   └── TEAM.md           ← scaffolded from template
├── project-notes/
└── ideas/
```

### Shared vs Project-Specific Files

| File | Ownership | Update mechanism |
|------|-----------|-----------------|
| `OWNER.md` | Shared (host `~/.config/dev-box/`) | Symlink, never project-edited |
| `work-instructions/GENERAL.md` | Shared from dev-box | `doctor` detects drift, `update` refreshes |
| `work-instructions/DEVELOPMENT.md` | Scaffolded per flavor, then project-owned | `doctor` reports structural drift only |
| All others | Fully project-owned | `doctor` validates structure, never modifies content |

---

## Context Schema & Doctor System

### Schema Documents

Each dev-box release includes a schema document at `schemas/vX.Y.Z/context-schema.md`.
This is both human-readable documentation and an AI-consumable prompt.

The schema document contains:
- A `schema_version` header
- An AI prompt preamble (instructions for a migration agent)
- Required directory layout per work process flavor
- Per-file: purpose, required sections, format, how to work with it
- Examples of well-formed entries

### How `dev-box doctor` Works

1. Reads `dev-box.toml` → knows target schema version and work process flavor
2. Reads `.dev-box-version` → knows current schema version
3. If versions differ, generates migration artifacts in `.dev-box/migration/`:
   - `schema-current.md` — schema the project was initialized with
   - `schema-target.md` — schema of the new version
   - `diff.md` — structural changes between versions
   - `migration-prompt.md` — AI-ready prompt with project-specific context
4. Prints summary to terminal: what's missing, what's structurally drifted
5. **Never modifies project files.** The human decides what to feed to which AI agent.

---

## This Project's Dev Container

`.devcontainer/Dockerfile` for THIS repo includes everything needed to
develop dev-box itself:

| Tool | Purpose |
|------|---------|
| Everything from base image | zellij, vim, git, lazygit, gh, claude CLI, audio, etc. |
| Rust toolchain (rustup) | Building the CLI |
| cargo components | clippy, rustfmt |
| Python 3.13 + uv | MkDocs documentation |
| mkdocs-material | Documentation site |
| podman or docker | Building and testing published images |

The Dockerfile for this project is self-contained and NOT derived from the
published base image (to avoid circular dependencies).

---

## Existing Decisions (Preserved)

### Dockerfile Patterns

- **Multi-stage build** for Zellij: builder downloads binary, runtime is clean
- **`uname -m`** for arch detection (not `TARGETARCH` — Podman doesn't inject it)
- **`DEBIAN_FRONTEND=noninteractive`** before all apt-get calls
- **Locale ENV after locale-gen** to avoid perl warnings
- **dpkg excludes before apt-get install** to suppress man page warnings
- **`CMD ["sleep", "infinity"]`** — container idles, tools exec into it

### Configuration Persistence

- `.root/` on host, bind-mounted into container, gitignored
- **Vim:** `.root/.vim/vimrc` (Vim 7.4+ auto-detects `~/.vim/vimrc`)
- **Git:** `.root/.config/git/config` (XDG path, `GIT_CONFIG_GLOBAL` env var)
- **Zellij:** `.root/.config/zellij/` (config + themes + layouts + plugin cache)
- **Claude:** `.root/.claude/` (config + memory)
- **Audio:** `.root/.asoundrc`
- Config seeding: missing files copied from templates on first run, never overwritten

### Zellij Configuration

- Theme: Gruvbox dark
- All bindings use `Alt` modifier (avoids vim/TUI conflicts)
- Key reference: Alt+s (strider), Alt+m (session), Alt+hjkl (navigate),
  Alt+n/d/r (new/split-down/split-right), Alt+x (close), Alt+f (fullscreen),
  Alt+t/w (new/close tab), Alt+[/] (prev/next tab), Alt+1-5 (jump tab),
  Alt+u (scroll), Alt+/ (search), Ctrl+q (quit)
- Layout: 3 tabs (dev: strider+vim+terminals, git: lazygit, shell: bash)

### Vim Configuration

- Space leader, relative+absolute line numbers, 4-space indent (2 for YAML/JSON/KDL/HTML/CSS/JS)
- undofile at `/root/.vim/undo`, no swap files
- colorcolumn=88, grepprg=rg, netrw tree mode, colorscheme desert

### Container Runtime

- Auto-detects Podman or Docker
- Uses `inspect` directly (not `compose ps`) for state checks
- Container/service names read from compose file, never hardcoded

---

## Deployment & Release Process

### No GitHub Actions

GitHub Actions are avoided due to cost. All builds and deploys are local.

### Release Workflow

When asked to release version X.Y.Z, follow ALL steps in order.

#### Phase 0 — Dependency version check (Claude does this FIRST)

Before every release, check ALL upstream dependencies for updates.
This cannot be automated reliably — each source has a different API.
Claude must check each one and report findings to the user before
proceeding with the release.

**Base image:**

| Dependency | Current | How to check |
|-----------|---------|-------------|
| `debian:trixie-slim` | trixie (Debian 13) | `gh api https://registry.hub.docker.com/v2/repositories/library/debian/tags?name=trixie-slim` or Docker Hub — check if trixie is still the right target (stable vs testing) |

**Pinned tool versions (in `images/` Dockerfiles and `.devcontainer/Dockerfile`):**

| Tool | Current | Pin location | How to check |
|------|---------|-------------|-------------|
| Zellij | 0.43.1 | `ARG ZELLIJ_VERSION` in base + .devcontainer | `gh api repos/zellij-org/zellij/releases/latest --jq .tag_name` |
| Typst | 0.13.1 | `ARG TYPST_VERSION` in typst + python-typst | `gh api repos/typst/typst/releases/latest --jq .tag_name` |
| TeX Live | 2025/tlnet-final | `ARG CTAN_MIRROR` in latex + python-latex + rust-latex | Check CTAN for new yearly release: `https://ftp.math.utah.edu/pub/tex/historic/systems/texlive/` |
| Rust toolchain | stable (unpinned) | rustup in rust + .devcontainer | `curl -s https://static.rust-lang.org/dist/channel-rust-stable.toml \| grep -m1 'version ='` — note: we use `stable` intentionally, just verify it works |
| uv | latest (unpinned) | `COPY --from=ghcr.io/astral-sh/uv:latest` in python + .devcontainer | `gh api repos/astral-sh/uv/releases/latest --jq .tag_name` — pinning considered but `:latest` is intentional for now |
| Claude CLI | unpinned | `curl claude.ai/install.sh` in base + .devcontainer | No version check possible — installed via pipe-to-bash, always gets latest |
| lazygit | unpinned (apt) | apt in base + .devcontainer | `gh api repos/jesseduffield/lazygit/releases/latest --jq .tag_name` — installed via apt, version depends on Debian repo |
| gh CLI | unpinned (apt) | apt in base + .devcontainer | Installed via apt, version depends on Debian repo |
| MkDocs | <2 constraint | `uv tool install 'mkdocs<2'` in python + .devcontainer | Check if mkdocs 2.0 situation has changed: `pip index versions mkdocs` |

**What to do with findings:**
- If a pinned version has an update: propose the bump, note breaking changes,
  update the `ARG` in all affected Dockerfiles
- If Debian stable has shifted: evaluate whether to follow
- If an unpinned tool has a major version bump: note it, test in a build
- Report all findings to the user before proceeding with the release
- Record any version bumps in the changelog

#### Phase 1 — Prep (inside dev-container, Claude does this)

1. **Version bump** — update ALL version references:
   - `cli/Cargo.toml` → `version = "X.Y.Z"`
   - `docs/changelog.md` → add new version section at top
   - `docs/cli/configuration.md` → update version in example configs
   - Any other docs referencing the old version number

2. **Update documentation** — review and update all MkDocs pages for
   accuracy with the new release. New features need docs.

3. **Commit version bump** — single commit: `chore: bump version to vX.Y.Z, update docs`

4. **Run `./scripts/maintain.sh release X.Y.Z`** — this:
   - Runs fmt check, clippy, and all tests (fails if dirty tree)
   - Builds the native aarch64-unknown-linux-gnu binary
   - Builds all 8 container images (tagged `-vX.Y.Z` and `-latest`)
   - Creates git tag `vX.Y.Z`
   - Generates `dist/RELEASE-NOTES.md` (commit log + image table)
   - Generates `dist/RELEASE-PROMPT.md`

5. **Cross-compile x86_64 Linux binary**:
   ```bash
   cd cli
   CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
     cargo build --release --target x86_64-unknown-linux-gnu
   cp target/x86_64-unknown-linux-gnu/release/dev-box ../dist/dev-box-vX.Y.Z-x86_64-unknown-linux-gnu
   cd ../dist
   tar -czf dev-box-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz dev-box-vX.Y.Z-x86_64-unknown-linux-gnu
   rm dev-box-vX.Y.Z-x86_64-unknown-linux-gnu
   ```

6. **Push tag and commits**:
   ```bash
   git push origin main
   git push origin vX.Y.Z
   ```

7. **Create GitHub release** (uploads both Linux binaries):
   ```bash
   gh release create vX.Y.Z \
     --repo projectious-work/dev-box \
     --title "dev-box vX.Y.Z" \
     --notes-file dist/RELEASE-NOTES.md \
     dist/dev-box-vX.Y.Z-*.tar.gz
   ```
   **Note:** Always use `--notes-file`, never `--generate-notes` (produces empty).

8. **Deploy documentation**:
   ```bash
   ./scripts/maintain.sh docs-deploy
   ```

#### Phase 2 — Host commands (user runs on macOS)

After Phase 1, Claude must print these commands for the user to run on
the macOS host. The user copy-pastes and runs them.

```bash
# ── Step 1: Build macOS binaries (arm64 + x86_64) ─────────────────────
cd /path/to/dev-box
./scripts/build-macos.sh X.Y.Z

# ── Step 2: Upload macOS binaries to the GitHub release ───────────────
gh release upload vX.Y.Z dist/dev-box-vX.Y.Z-*-apple-darwin.tar.gz

# ── Step 3: Build and push container images to GHCR ──────────────────
# (auto-tags *-latest → *-vX.Y.Z, auto-logins via gh auth)
./scripts/maintain.sh push-images X.Y.Z
```

**Prerequisites for host commands:**
- Rust toolchain on macOS (`rustup` with both apple-darwin targets)
- `gh` CLI authenticated with `write:packages` scope:
  `gh auth refresh --scopes write:packages,read:packages`
- Docker/OrbStack running (for push-images)

#### Release Checklist Summary

| Step | Where | Command/Action |
|------|-------|----------------|
| Version bump | container | Edit Cargo.toml + docs |
| Commit | container | `git commit` |
| Build + tag | container | `./scripts/maintain.sh release X.Y.Z` |
| Cross-compile x86 | container | `cargo build --release --target x86_64-unknown-linux-gnu` |
| Push | container | `git push origin main && git push origin vX.Y.Z` |
| GitHub release | container | `gh release create vX.Y.Z ...` |
| Deploy docs | container | `./scripts/maintain.sh docs-deploy` |
| macOS binaries | host | `./scripts/build-macos.sh X.Y.Z` |
| Upload macOS | host | `gh release upload vX.Y.Z dist/*-apple-darwin.tar.gz` |
| Push images | host | `./scripts/maintain.sh push-images X.Y.Z` |

### Documentation

- Built and deployed locally with `mkdocs gh-deploy`
- Pushes to `gh-pages` branch, served by GitHub Pages
- No CI pipeline — run manually after doc changes

### Container Images

- Built and pushed locally to GHCR (`ghcr.io/projectious-work/dev-box`)
- `scripts/maintain.sh` handles multi-image builds

### GitHub Organization

- Repository owner: `projectious-work` GitHub org
- GHCR registry: `ghcr.io/projectious-work/dev-box`
- GitHub Pages: `https://projectious-work.github.io/dev-box/`
- The repo was originally under `bnaard` and transferred to `projectious-work`
  (GitHub maintains redirects from the old URL)

---

## Known Issues and Gotchas

- **Podman compose** output format varies by version; always use `inspect`
- **Stale image cache**: rebuild with `--no-cache` if container exits immediately
- **`.root/` must be in `.gitignore`** — contains SSH keys and personal config
- **SSH keys**: warn if `.root/.ssh/` is empty, non-fatal
- **Zellij pinned to 0.43.1**: change `ARG ZELLIJ_VERSION` to upgrade
- **`host.docker.internal`**: works on Docker Desktop and Podman pasta;
  bare Linux Docker may need `--add-host`
- **OrbStack virtiofs**: files mounted from macOS may lose execute permissions.
  Affects plugin cache scripts. Workaround: `chmod +x` inside container.
- **Claude Code OAuth in containers**: `claude auth` uses a random ephemeral
  port for the OAuth callback, which isn't forwarded from bridge-networked
  containers. Workaround: use `claude setup-token` or authenticate on the host
  (credentials shared via `.claude` mount). Upstream tracking:
  [anthropics/claude-code#14528](https://github.com/anthropics/claude-code/issues/14528)
  — check periodically for a fix (e.g., configurable callback port).
  **Decision: do NOT use `network_mode: host` as a workaround** — it breaks
  container network isolation.

---

## Implementation Plan

### Phase 1 — Foundation (COMPLETED 2026-03-16)

**Goal:** Restructure repo, create all Dockerfiles, scaffold Rust CLI with
core container management commands.

#### 1.1 Restructure Dockerfiles

- [x] Create `images/base/Dockerfile` — base with zellij, vim, git, lazygit,
      gh, claude CLI, audio (sox, pulseaudio-utils), .asoundrc
- [x] Create `images/python/Dockerfile` — FROM base + python 3.13 + uv + mkdocs-material
- [x] Create `images/latex/Dockerfile` — multi-stage TeX Live builder + FROM base
- [x] Create `images/rust/Dockerfile` — FROM base + rustup + cargo + clippy + rustfmt
- [x] Create `images/python-latex/Dockerfile` — TeX Live builder + FROM python
- [x] Create `images/rust-latex/Dockerfile` — TeX Live builder + FROM rust
- [x] Update `.devcontainer/Dockerfile` for THIS project: added Rust toolchain,
      Python 3.13 + uv, mkdocs-material, gh, audio (self-contained, not FROM base)
- [x] Created `images/base/config/` with all config templates (vimrc, gitconfig,
      zellij/config.kdl, zellij/themes/gruvbox.kdl, zellij/layouts/dev.kdl, asoundrc)

#### 1.2 Initialize Rust CLI

- [x] Created `cli/Cargo.toml` — Rust 2024 edition, all dependencies (clap, toml,
      serde, serde_yaml, serde_json, minijinja, tracing, tracing-subscriber, semver,
      ureq, dirs, anyhow)
- [x] Implemented `config.rs` — DevBoxConfig struct with ImageFlavor/ProcessFlavor
      enums (kebab-case serde), semver validation, env var overrides
- [x] Implemented `cli.rs` — clap derive with all 9 subcommands
- [x] Implemented `main.rs` — entry point, tracing setup, dispatch
- [x] Implemented `runtime.rs` — podman/docker detection, compose/inspect abstraction
- [x] Implemented `seed.rs` — .root/ seeding with embedded default configs
- [x] Implemented `output.rs` — ANSI-colored terminal output (info/ok/warn/error)

#### 1.3 Implement `generate` Command

- [x] Implemented `generate.rs` — generates Dockerfile, docker-compose.yml,
      devcontainer.json from dev-box.toml using string formatting
- [x] Generated files include header comment with version
- [ ] TODO (Phase 2): Migrate from string formatting to minijinja templates

#### 1.4 Implement Container Lifecycle Commands

- [x] Implemented `container.rs` — all lifecycle commands wired
- [x] `build` — loads config, generates, calls compose build
- [x] `start` — seeds .root/, generates, starts container, attaches via zellij
- [x] `stop` — compose stop with state detection
- [x] `attach` — exec into running container with zellij
- [x] `status` — reports running/stopped/missing
- [x] Readiness wait with 500ms polling, 7.5s timeout
- [x] `init` — creates dev-box.toml + generates files (basic, no context scaffolding yet)
- [x] `doctor` — placeholder with basic diagnostics
- [x] `update` — placeholder

#### 1.5 Install Dev Tools in This Project's Container

- [x] Installed Rust 1.94.0 + build-essential + pkg-config + libssl-dev
- [x] Updated `.devcontainer/Dockerfile` with Rust, Python/uv, MkDocs permanently
- [x] Verified `cargo build` compiles successfully
- [x] Verified CLI runs: `dev-box --help`, `dev-box generate`, `dev-box status`

#### 1.6 Known Issues from Phase 1

- `generate` writes to `.devcontainer/` unconditionally — for THIS project
  (hand-maintained), we must not run `generate`. Future: add a `managed = false`
  flag in dev-box.toml or skip generation when a sentinel exists.
- `init` creates dev-box.toml + devcontainer files but does not yet scaffold
  context/ directory — that's Phase 2.
- `doctor` and `update` are placeholders — Phase 3.
- The generate command uses string formatting instead of minijinja templates —
  works but should be migrated for maintainability in Phase 2.

### Phase 2 — Context System + Init + Documentation (COMPLETED 2026-03-16)

**Goal:** Implement work process flavors, `init` command, context scaffolding,
and MkDocs documentation.

#### 2.1 Work Process Templates

- [x] Created `templates/minimal/` — CLAUDE.md.template
- [x] Created `templates/managed/` — CLAUDE.md.template, DECISIONS.md, BACKLOG.md,
      STANDUPS.md, work-instructions/GENERAL.md (5 files)
- [x] Created `templates/research/` — CLAUDE.md.template, PROGRESS.md,
      research/.gitkeep, analysis/.gitkeep (4 files)
- [x] Created `templates/product/` — full set: CLAUDE.md.template, DECISIONS.md,
      BACKLOG.md, PROJECTS.md, STANDUPS.md, PRD.md, work-instructions/{GENERAL,
      DEVELOPMENT, TEAM}.md, project-notes/.gitkeep, ideas/.gitkeep (11 files)
- Total: 21 template files across 4 flavors

#### 2.2 Implement `init` Command (Enhanced)

- [x] Created `cli/src/context.rs` — scaffold_context() with embedded templates
      via include_str!(), {{project_name}} substitution, OWNER.md symlinking
- [x] `init` now: creates dev-box.toml → generates devcontainer files → scaffolds
      context/ with flavor-appropriate files → creates CLAUDE.md → creates
      .dev-box-version → updates .gitignore
- [x] OWNER.md: symlinks to ~/.config/dev-box/OWNER.md if exists, otherwise
      creates placeholder with instructions
- [x] Never overwrites existing files (safe for re-runs)
- [x] Smoke-tested all 4 process flavors: minimal, managed, research, product

#### 2.3 Context Schema Documents

- [x] Created `schemas/v1.0.0/context-schema.md` — 422 lines, includes:
      AI migration agent instructions, per-flavor file requirements,
      detailed file specifications with examples, migration principles

#### 2.4 OWNER.md Shared Config

- [x] Symlink mechanism implemented in context.rs
- [x] Placeholder with instructions created when ~/.config/dev-box/OWNER.md missing
- [x] Tip printed to terminal suggesting OWNER.md creation

#### 2.5 MkDocs Documentation

- [x] Created `mkdocs.yml` — Material theme with light/dark toggle, code copy,
      navigation tabs (top menu bar), admonition support
- [x] Created 15 documentation pages under `docs/`:
  - `index.md` — overview, "uv for AI work environments" vision
  - `getting-started/{installation, new-project, existing-project}.md`
  - `container/{base-image, flavors, audio}.md`
  - `context/{overview, work-processes, migration}.md`
  - `cli/{commands, configuration}.md`
  - `contributing.md`, `roadmap.md`, `changelog.md`
- [x] Top menu bar: Home, Getting Started, Container Images, Context System,
      CLI Reference, Contributing, Roadmap, Changelog
- [x] Deployment: `mkdocs gh-deploy` from local machine (no GitHub Actions).
      Builds locally and pushes to `gh-pages` branch. GitHub Pages serves
      static files from that branch.

### Phase 3 — Doctor + Update + CI (COMPLETED 2026-03-16)

**Goal:** Schema validation, version management, automated builds, tests.

#### 3.0 Tests

- [x] Added 68 unit tests across all modules (config: 22, generate: 15,
      context: 12, seed: 7, runtime: 6, output: 6 — using `#[cfg(test)]` modules)
- [x] Added 13 integration tests (`cli/tests/integration.rs`) testing CLI binary
      as subprocess: help, init, generate, doctor, invalid inputs, all flavors
- [x] All 81 tests pass, zero clippy warnings with `-D warnings`
- [x] Added dev-dependencies: `tempfile = "3"`, `serial_test = "3"`

#### 3.1 Implement `doctor` Command

- [x] Created `cli/src/doctor.rs` — full diagnostic command
- [x] Checks: config validity, container runtime, .root/ directories,
      .devcontainer/ files, context structure against process flavor
- [x] Reports missing files as errors, extra files as warnings
- [x] Schema version comparison: reads .dev-box-version vs config
- [x] Generates migration artifacts in `.dev-box/migration/` when versions differ:
      schema-current.md, schema-target.md, diff.md, migration-prompt.md
- [x] Embeds v1.0.0 schema via include_str!
- [x] Clear summary output with warning/error counts
- [x] Never modifies project files

#### 3.2 Implement `update` Command

- [x] Created `cli/src/update.rs`
- [x] `--check` mode: shows CLI version, config version, schema version
- [x] Default mode: shows manual update instructions
- [x] Registry checking marked as future work (prints informative message)

#### 3.3 CI & Deployment

- **No GitHub Actions workflows** — avoided due to cost. All builds and
  deploys are run locally.
- CLI binaries: built locally (or via `scripts/build-macos.sh`), attached
  to GitHub releases manually via `gh release create`
- Container images: built and pushed locally
- Documentation: `mkdocs gh-deploy` builds locally and pushes to `gh-pages` branch

### Phase 4 — Migration + Polish (COMPLETED 2026-03-16)

**Goal:** Generate dev-box.toml migration configs for existing projects,
validate end-to-end, update README.

- [x] Generated `dev-box.toml` + MIGRATION.md for kaits (python + product)
- [x] Generated `dev-box.toml` + MIGRATION.md for internal (python-latex + product)
- [x] Generated `dev-box.toml` + MIGRATION.md for kubernetes-learning (python-latex + research)
- [x] Generated `dev-box.toml` + MIGRATION.md for ai-learning (python-latex + research)
- [x] Generated `dev-box.toml` + MIGRATION.md for vim-cheatsheet (python-latex + research)
- [x] All configs validated with `dev-box doctor`
- [x] Updated `docs/getting-started/existing-project.md` with real migration
      examples and common gaps table
- [x] Rewrote `README.md` — new project overview, quickstart, image/process
      flavor tables, CLI reference, docs links

#### Gaps Identified During Migration

These are features that existing projects use but dev-box doesn't yet support:
- **postCreateCommand** — internal uses it for git identity; workaround is
  `.root/.config/git/config`
- **Node.js version pinning** — extra_packages gives Debian version, not
  NodeSource LTS; may need post-create script
- **Playwright chromium binary** — must be installed post-create
- **Third-party CLIs** (Gemini, Jules) — can be handled via extra_volumes
  mounting from host, or extra_packages for apt-installable ones
- **VS Code project-specific settings** — dev-box generates basic settings;
  projects can supplement with `.vscode/settings.json`

### Phase 5 — Code Review + Hardening (COMPLETED 2026-03-16)

**Goal:** Comprehensive code review using plugins (code-review, simplify,
security-guidance), then implement approved improvements.

#### 5.1 Code Review (3 parallel agents)

- [x] Agent 1 (Reuse): 9 findings — duplicated functions, dead code, unused deps,
      inconsistent registry URL, triplicated TeX Live builder
- [x] Agent 2 (Quality): 15 findings — stringly-typed paths, copy-paste scaffolding,
      doctor/scaffold sync risk, dead code, placeholder URLs
- [x] Agent 3 (Efficiency): 12 findings — unconditional regeneration, unused deps,
      unnecessary string allocations, TOCTOU patterns

#### 5.2 Fixes Applied

HIGH (all fixed):
- [x] Removed 3 unused Cargo dependencies (minijinja, serde_yaml, ureq) —
      reduces compile time and binary size
- [x] Removed dead `extensions_json` code + `let _ =` suppression in generate.rs
- [x] Consolidated triplicated `load_config` → `DevBoxConfig::from_cli_option()`
- [x] Consolidated duplicated `seed_file`/`write_if_missing` → single helper

MEDIUM (all fixed):
- [x] Fixed inconsistent registry URL → `IMAGE_REGISTRY` constant
- [x] Data-driven scaffolding: `expected_context_files()` shared between
      context.rs and doctor.rs (eliminates sync risk)
- [x] Shared path constants: `DEVCONTAINER_DIR`, `COMPOSE_FILE`, `DOCKERFILE`,
      `DEVCONTAINER_JSON` used throughout
- [x] Skip file writes when content unchanged → `write_if_changed()` helper
- [x] Merged duplicate Stopped/Missing match arms in container.rs

LOW (fixed where trivial):
- [x] Fixed placeholder URL in update.rs (your-org → projectious-work)
- [x] Fixed deploy script REPO_SLUG extraction (unconditional, guarded URL line)
- Skipped: string allocation optimization (premature for a CLI tool),
  TOCTOU in write_if_missing (acceptable in single-user CLI),
  timezone configurability (future feature), test helper dedup

#### 5.3 Verification

- [x] All 81 tests pass (68 unit + 13 integration)
- [x] Zero clippy warnings with `-D warnings`
- [x] TeX Live Dockerfile duplication noted as known trade-off of Docker's
      single-file model — added `ARG BASE_IMAGE` is sufficient

---

## Agent Strategy

Three agents, each with a clear scope. Run in parallel where possible.

### Agent 1: Image Builder
**Scope:** All Dockerfiles in `images/`, this project's `.devcontainer/Dockerfile`.
- Creates the 6 published image Dockerfiles
- Updates this project's Dockerfile with Rust + Python/uv + MkDocs
- Handles the TeX Live multi-stage build pattern
- Adds audio support to base image
- Tests that images build successfully

### Agent 2: CLI Developer
**Scope:** Everything in `cli/`, `templates/`, `schemas/`.
- Initializes the Rust project
- Implements all CLI commands
- Creates minijinja templates for file generation
- Creates work process flavor templates
- Creates context schema documents
- Writes tests

### Agent 3: Documentation + Integration
**Scope:** `docs/`, `mkdocs.yml`, README.md, CI workflows, integration testing.
- Sets up MkDocs project structure
- Writes all documentation pages
- Creates GitHub Actions workflows
- Tests end-to-end workflows (init → generate → build → doctor)
- Updates README.md

### Parallelization

- **Phase 1:** Agent 1 (Dockerfiles) and Agent 2 (CLI skeleton + config + generate)
  can run in parallel. Agent 2 depends on knowing the image names/tags but not
  on the actual Dockerfiles being written yet.
- **Phase 2:** Agent 2 (init + context) and Agent 3 (docs + MkDocs setup)
  can run in parallel.
- **Phase 3:** Agent 2 (doctor + update) and Agent 3 (CI workflows) can run
  in parallel. Agent 1 needed for CI image build testing.

---

## Reference: Learned Patterns from Derived Projects

These patterns were observed across 6 repositories (kaits, internal,
kubernetes-learning, ai-learning, vim-cheatsheet, network-learning) and
informed the design above.

### Duplication eliminated by dev-box
- Dockerfile base (zellij, vim, git, locale, timezone) — copied in 5/6 repos
- docker-compose.yml volume mounts (.root/, SSH, git, claude) — copied in 5/6 repos
- TeX Live multi-stage build — copied in 4/6 repos
- Config templates (vimrc, gitconfig, zellij/) — copied in 2/6 repos
- dev.sh script — copied in 2/6 repos
- Claude CLI installation — copied in 5/6 repos

### Context structure convergence
- OWNER.md: identical in kaits + internal (describes the human, not the project)
- DECISIONS.md: same format in kaits + internal (inverse chronological, IDs)
- BACKLOG.md: same format in kaits + internal (ID registry, status tracking)
- work-instructions/: mature in kaits (GENERAL, DEVELOPMENT, TEAM)
- context/ directory: evolving across projects (3 files → 8 files → full structure)

### Key technical patterns adopted
- Dataclass-based config with env var layering (from kaits)
- JSON logging with stdlib (from kaits)
- Multi-stage TeX Live build from CTAN (from kubernetes-learning et al.)
- PulseAudio audio bridging (from kubernetes-learning)
- `.dev-box-version` tracking (new, inspired by uv's lockfile concept)
- Config seeding with idempotency (from original dev.sh)
