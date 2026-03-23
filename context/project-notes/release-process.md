# Release Process

When asked to release version X.Y.Z, follow ALL steps in order.

## Phase 0 — Dependency version check (Claude does this FIRST)

Before every release, check ALL upstream dependencies for updates.

**Base image:**

| Dependency | Current | How to check |
|-----------|---------|-------------|
| `debian:trixie-slim` | trixie (Debian 13) | Docker Hub — check if trixie is still the right target |

**Pinned tool versions (in `images/` Dockerfiles and `.devcontainer/Dockerfile`):**

| Tool | Current | Pin location | How to check |
|------|---------|-------------|-------------|
| Zellij | 0.43.1 | `ARG ZELLIJ_VERSION` in base + .devcontainer | `gh api repos/zellij-org/zellij/releases/latest --jq .tag_name` |
| Typst | 0.13.1 | `ARG TYPST_VERSION` in typst + python-typst | `gh api repos/typst/typst/releases/latest --jq .tag_name` |
| TeX Live | 2025/tlnet-final | `ARG CTAN_MIRROR` in latex + python-latex + rust-latex | Check CTAN yearly release |
| Rust | stable (unpinned) | rustup in rust + .devcontainer | Verify stable works |
| uv | latest (unpinned) | `COPY --from=ghcr.io/astral-sh/uv:latest` | `gh api repos/astral-sh/uv/releases/latest --jq .tag_name` |
| Claude CLI | unpinned | `curl claude.ai/install.sh` | Always gets latest |
| Zensical | unpinned | `uv tool install zensical` | `pip index versions zensical` |

**Actions:** If pinned version has update, propose bump. Report all findings before proceeding.

## Phase 1 — Prep (inside dev-container)

1. **Version bump**: `cli/Cargo.toml`, `docs/changelog.md`, `docs/cli/configuration.md`
2. **Update documentation** for new features
3. **Commit and push**:
   ```bash
   cargo generate-lockfile --manifest-path cli/Cargo.toml
   git add cli/Cargo.toml cli/Cargo.lock docs/
   git commit -m "chore: bump version to vX.Y.Z, update docs"
   git push origin main
   ```
4. **Visual smoke tests**: `./scripts/maintain.sh test-visual`
5. **Build**: `./scripts/maintain.sh release X.Y.Z`
5. **Cross-compile x86_64**:
   ```bash
   cd cli
   CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
     cargo build --release --target x86_64-unknown-linux-gnu
   cp target/x86_64-unknown-linux-gnu/release/dev-box ../dist/dev-box-vX.Y.Z-x86_64-unknown-linux-gnu
   cd ../dist && tar -czf dev-box-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz dev-box-vX.Y.Z-x86_64-unknown-linux-gnu
   rm dev-box-vX.Y.Z-x86_64-unknown-linux-gnu
   ```
6. **Push tag**: `git push origin main && git push origin vX.Y.Z`
7. **GitHub release**: `gh release create vX.Y.Z --repo projectious-work/dev-box --title "dev-box vX.Y.Z" --notes-file dist/RELEASE-NOTES.md dist/dev-box-vX.Y.Z-*.tar.gz`
   Note: Always use `--notes-file`, never `--generate-notes`.
8. **Deploy docs**: `./scripts/maintain.sh docs-deploy`

## Phase 2 — Host commands (user runs on macOS)

```bash
cd /path/to/dev-box
./scripts/build-macos.sh X.Y.Z
gh release upload vX.Y.Z dist/dev-box-vX.Y.Z-*-apple-darwin.tar.gz
./scripts/maintain.sh push-images X.Y.Z
```

Prerequisites: Rust toolchain on macOS, `gh` with `write:packages` scope, Docker/OrbStack running.
