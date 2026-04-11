---
id: NOTE-20260411_0000-LoyalSpruce-aibox-release-process
title: "aibox Release Process"
type: reference
status: permanent
created: 2026-04-11T00:00:00Z
tags: [release, process, operations]
skill: release-semver
---

# aibox Release Process

When asked to release version X.Y.Z, follow ALL steps in order.
Full canonical source: `context/work-instructions/RELEASE-PROCESS.md` (archived).

## Phase 0 — Dependency version check (Claude does this FIRST)

Before every release, check ALL upstream dependencies for updates.

### processkit

```bash
./scripts/maintain.sh sync-processkit
```

Queries GitHub for the latest processkit tag, patches `PROCESSKIT_DEFAULT_VERSION` in
`cli/src/processkit_vocab.rs` if newer, shows FORMAT.md diff so you can spot vocabulary
changes. If `processkit_vocab.rs` was patched: review diff, make CLI changes, run
`cargo test`, then commit everything before continuing.

### Pinned tool versions (in `images/base-debian/Dockerfile` and `.devcontainer/Dockerfile`)

| Tool | Pin location | How to check |
|------|-------------|-------------|
| Zellij | `ARG ZELLIJ_VERSION` in base + .devcontainer | `gh api repos/zellij-org/zellij/releases/latest --jq .tag_name` |
| Yazi | `ARG YAZI_VERSION` | `gh api repos/sxyazi/yazi/releases/latest --jq .tag_name` |
| ripgrep | `ARG RIPGREP_VERSION` | `gh api repos/BurntSushi/ripgrep/releases/latest --jq .tag_name` |
| fd | `ARG FD_VERSION` | `gh api repos/sharkdp/fd/releases/latest --jq .tag_name` |
| bat | `ARG BAT_VERSION` | `gh api repos/sharkdp/bat/releases/latest --jq .tag_name` |
| eza | `ARG EZA_VERSION` | `gh api repos/eza-community/eza/releases/latest --jq .tag_name` |
| fzf | `ARG FZF_VERSION` | `gh api repos/junegunn/fzf/releases/latest --jq .tag_name` |
| delta | `ARG DELTA_VERSION` | `gh api repos/dandavison/delta/releases/latest --jq .tag_name` |
| ouch | `ARG OUCH_VERSION` | `gh api repos/ouch-org/ouch/releases/latest --jq .tag_name` |
| starship | `ARG STARSHIP_VERSION` | `gh api repos/starship/starship/releases/latest --jq .tag_name` |
| zoxide | `ARG ZOXIDE_VERSION` | `gh api repos/ajeetdsouza/zoxide/releases/latest --jq .tag_name` |
| python3 | apt `python3` (Debian Trixie, ~3.13.x) | Check Trixie default |
| uv | `COPY --from=ghcr.io/astral-sh/uv:latest` (unpinned) | `gh api repos/astral-sh/uv/releases/latest --jq .tag_name` |
| Node.js | `COPY --from=node:22-slim` in .devcontainer | Check LTS status |

If a pinned version has an update, propose bump. Report all findings before proceeding.

## Phase 1 — In container (Claude does this)

1. **Version bump**: `cli/Cargo.toml`
2. **Update documentation** for new features
3. **Run tests and clippy**:
   ```bash
   cd cli && cargo test && cargo clippy -- -D warnings
   ```
4. **Audit dependencies**:
   ```bash
   cd cli && cargo audit
   ```
5. **Build Linux release binaries — both architectures**:
   ```bash
   cd /workspace/cli
   # Native aarch64 build
   cargo build --release
   # Cross-compile for x86_64
   CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
     cargo build --release --target x86_64-unknown-linux-gnu
   ```
6. **Package Linux binaries**:
   ```bash
   cd /workspace && mkdir -p dist && VERSION=X.Y.Z
   cp cli/target/release/aibox dist/aibox-v${VERSION}-aarch64-unknown-linux-gnu
   tar -czf dist/aibox-v${VERSION}-aarch64-unknown-linux-gnu.tar.gz \
     -C dist aibox-v${VERSION}-aarch64-unknown-linux-gnu
   rm dist/aibox-v${VERSION}-aarch64-unknown-linux-gnu
   cp cli/target/x86_64-unknown-linux-gnu/release/aibox dist/aibox-v${VERSION}-x86_64-unknown-linux-gnu
   tar -czf dist/aibox-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz \
     -C dist aibox-v${VERSION}-x86_64-unknown-linux-gnu
   rm dist/aibox-v${VERSION}-x86_64-unknown-linux-gnu
   ls -lh dist/aibox-v${VERSION}-*-linux-*.tar.gz   # verify both tarballs exist
   ```
7. **Write release notes** to `dist/RELEASE-NOTES.md`
8. **Commit, tag, push**:
   ```bash
   git add cli/Cargo.toml cli/Cargo.lock
   git commit -m "chore: bump version to vX.Y.Z"
   git tag vX.Y.Z
   git push origin main && git push origin vX.Y.Z
   ```
9. **Create GitHub release with Linux binaries attached**:
   ```bash
   gh release create vX.Y.Z --repo projectious-work/aibox \
     --title "aibox vX.Y.Z" --notes-file dist/RELEASE-NOTES.md \
     dist/aibox-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz \
     dist/aibox-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
   ```
   Always use `--notes-file`, never `--generate-notes`. macOS binaries added in Phase 2.
10. **Deploy documentation**:
    ```bash
    ./scripts/maintain.sh docs-deploy
    ```

## Phase 2 — On macOS host (user runs one command)

```bash
cd /path/to/aibox
./scripts/maintain.sh release-host X.Y.Z
```

Builds macOS binaries (arm64 + x86_64), uploads to the existing GitHub release, builds
container images, pushes to GHCR.

**Prerequisites:** Rust toolchain on macOS, `gh` authenticated with `write:packages` scope,
Docker/OrbStack running.

**Critical gotcha:** A fresh `gh auth login` only grants the default `repo` scope — GHCR
push fails with `denied: permission_denied`. Fix:
```bash
gh auth refresh -s read:packages,write:packages,delete:packages
```
The script is idempotent for the binary upload step — safe to retry after a partial run.

## Commit message convention for releases

- **Version bump only:** `chore: bump version to vX.Y.Z`
- **With real changes:** `fix(vX.Y.Z): <one-line summary>` + section-by-section body with
  `Refs: DEC-NNN, BACK-NNN` and `Co-Authored-By:` trailer.
