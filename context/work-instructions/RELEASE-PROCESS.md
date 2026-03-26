# Release Process

When asked to release version X.Y.Z, follow ALL steps in order.

## Phase 0 — Dependency version check (Claude does this FIRST)

Before every release, check ALL upstream dependencies for updates.

**Base image (`images/base-debian/Dockerfile`):**

| Dependency | Current | How to check |
|-----------|---------|-------------|
| `debian:trixie-slim` | trixie (Debian 13) | Docker Hub — check if trixie is still the right target |

**Pinned tool versions (in `images/base-debian/Dockerfile` and `.devcontainer/Dockerfile`):**

| Tool | Current | Pin location | How to check |
|------|---------|-------------|-------------|
| Zellij | 0.44.0 | `ARG ZELLIJ_VERSION` in base + .devcontainer | `gh api repos/zellij-org/zellij/releases/latest --jq .tag_name` |
| Yazi | 26.1.22 | `ARG YAZI_VERSION` | `gh api repos/sxyazi/yazi/releases/latest --jq .tag_name` |
| ripgrep | 15.1.0 | `ARG RIPGREP_VERSION` | `gh api repos/BurntSushi/ripgrep/releases/latest --jq .tag_name` |
| fd | 10.4.2 | `ARG FD_VERSION` | `gh api repos/sharkdp/fd/releases/latest --jq .tag_name` |
| bat | 0.26.1 | `ARG BAT_VERSION` | `gh api repos/sharkdp/bat/releases/latest --jq .tag_name` |
| eza | 0.23.4 | `ARG EZA_VERSION` | `gh api repos/eza-community/eza/releases/latest --jq .tag_name` |
| fzf | 0.70.0 | `ARG FZF_VERSION` | `gh api repos/junegunn/fzf/releases/latest --jq .tag_name` |
| delta | 0.19.1 | `ARG DELTA_VERSION` | `gh api repos/dandavison/delta/releases/latest --jq .tag_name` |
| starship | 1.24.2 | `ARG STARSHIP_VERSION` | `gh api repos/starship/starship/releases/latest --jq .tag_name` |
| zoxide | 0.9.9 | `ARG ZOXIDE_VERSION` | `gh api repos/ajeetdsouza/zoxide/releases/latest --jq .tag_name` |
| Rust | stable (unpinned) | rustup in .devcontainer | Verify stable works |
| uv | latest (unpinned) | `COPY --from=ghcr.io/astral-sh/uv:latest` | `gh api repos/astral-sh/uv/releases/latest --jq .tag_name` |
| Node.js | 22 | `COPY --from=node:22-slim` in .devcontainer | Check LTS status |

**Actions:** If a pinned version has an update, propose bump. Report all findings before proceeding.

## Phase 1 — In container (Claude does this)

1. **Version bump**: `cli/Cargo.toml`
2. **Update documentation** for new features
3. **Run tests and clippy**:
   ```bash
   cd cli && cargo test && cargo clippy -- -D warnings
   ```
4. **Audit dependencies** (advisories must be clean before tagging the release):
   ```bash
   cd cli && cargo audit
   ```
6. **Build Linux release binaries** (both architectures — container is aarch64, cross-compile for x86_64):
   ```bash
   cd /workspace/cli
   cargo build --release
   CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
     cargo build --release --target x86_64-unknown-linux-gnu
   ```
7. **Package Linux binaries**:
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
8. **Write release notes** to `dist/RELEASE-NOTES.md`
9. **Commit, tag, push**:
   ```bash
   git add cli/Cargo.toml cli/Cargo.lock
   git commit -m "chore: bump version to vX.Y.Z"
   git tag vX.Y.Z
   git push origin main && git push origin vX.Y.Z
   ```
10. **Create GitHub release with Linux binaries attached**:
    ```bash
    gh release create vX.Y.Z --repo projectious-work/aibox \
      --title "aibox vX.Y.Z" --notes-file dist/RELEASE-NOTES.md \
      dist/aibox-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz \
      dist/aibox-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
    ```
    Note: Always use `--notes-file`, never `--generate-notes`. macOS binaries are added in Phase 2.
11. **Deploy documentation** (requires Node.js/Docusaurus — runs in container):
   ```bash
   ./scripts/maintain.sh docs-deploy
   ```

## Phase 2 — On macOS host (user runs one command)

```bash
cd /path/to/aibox
./scripts/maintain.sh release-host X.Y.Z
```

This single command does:
1. Builds macOS binaries (arm64 + x86_64) via `build-macos.sh`
2. Uploads macOS binaries to the existing GitHub release
3. Builds container images
4. Pushes container images to GHCR

**Prerequisites:** Rust toolchain on macOS, `gh` authenticated with `write:packages` scope, Docker/OrbStack running.

**Note:** If `gh` is missing `read:packages` or `write:packages` scopes (happens after `/login` in Claude Code), run:
```bash
gh auth refresh -s read:packages,delete:packages
```
