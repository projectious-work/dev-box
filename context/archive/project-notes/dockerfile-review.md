# Dockerfile Review Report

Date: 2026-03-22
Reference: `/workspace/context/work-instructions/DOCKERFILE-PRACTICES.md`

## Summary Table

| Dockerfile | Status | Priority | Key Issues |
|---|---|---|---|
| `base/Dockerfile` | Needs attention | Medium | Missing BuildKit syntax, no checksum verification, cache mount not used, multiple COPY layers consolidable |
| `python/Dockerfile` | Needs attention | High | Unpinned `uv:latest` tag, `unzip` redundant (already in base) |
| `latex/Dockerfile` | Needs attention | Low | Missing fontconfig link (present in python-latex but not here), TeX Live builder duplicated across 3 files |
| `typst/Dockerfile` | Good | Low | Minor: could use cache mount for apt |
| `rust/Dockerfile` | Needs attention | Medium | Rustup piped from curl with no verification |
| `python-latex/Dockerfile` | Needs attention | Medium | Full TeX Live builder copy-pasted from latex, missing fontconfig registration |
| `python-typst/Dockerfile` | Good | Low | Same minor issues as typst |
| `rust-latex/Dockerfile` | Needs attention | Medium | Full TeX Live builder copy-pasted from latex, missing fontconfig registration |

## Findings by Dockerfile

### 1. `images/base/Dockerfile`

**a) Missing `# syntax=docker/dockerfile:1` directive (Medium)**
Best practices document recommends adding the BuildKit syntax directive at the top for cache mount support and other BuildKit features. Not present.

**b) No checksum verification for downloaded binaries (Medium)**
The practices doc says "Always verify checksums for downloaded binaries." All 10 tool downloads (Zellij, Yazi, ripgrep, fd, bat, eza, zoxide, fzf, delta, starship) use `curl | tar` with no checksum verification. This is a supply-chain security concern.

Recommended fix:
```dockerfile
ARG ZELLIJ_SHA256_X86=abc123...
ARG ZELLIJ_SHA256_ARM=def456...
RUN ARCH=$(uname -m) && \
    curl -fsSL -o /tmp/zellij.tar.gz \
      "https://github.com/zellij-org/zellij/releases/download/v${ZELLIJ_VERSION}/zellij-${ARCH}-unknown-linux-musl.tar.gz" && \
    EXPECTED=$(case $ARCH in x86_64) echo "${ZELLIJ_SHA256_X86}";; aarch64) echo "${ZELLIJ_SHA256_ARM}";; esac) && \
    echo "${EXPECTED}  /tmp/zellij.tar.gz" | sha256sum -c - && \
    tar -xzf /tmp/zellij.tar.gz -C /usr/local/bin && \
    rm /tmp/zellij.tar.gz
```

**c) Cache mounts not used for apt-get (Low)**
Both builder and runtime stages use `rm -rf /var/lib/apt/lists/*` instead of cache mounts. The practices doc recommends:
```dockerfile
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends ...
```
Note: this requires the `# syntax=docker/dockerfile:1` directive.

**d) Multiple COPY --from=builder layers (Low)**
Lines 150-159 have 10 separate COPY instructions. These could be consolidated since all binaries are in the same source directory:
```dockerfile
COPY --from=builder /usr/local/bin/zellij /usr/local/bin/yazi \
    /usr/local/bin/rg /usr/local/bin/fd /usr/local/bin/bat \
    /usr/local/bin/eza /usr/local/bin/zoxide /usr/local/bin/fzf \
    /usr/local/bin/delta /usr/local/bin/starship /usr/local/bin/
```
This reduces the layer count by 9. Each COPY creates a separate layer.

**e) Vim colorscheme download and git clone in separate RUN from mkdir (Low)**
Lines 167-182: the `mkdir -p` on line 167 is a separate RUN from the downloads on lines 170-182. These could be combined into one layer.

**f) Base image not pinned by digest (Low)**
`FROM debian:trixie-slim` is used without a `@sha256:...` digest pin. The practices doc recommends pinning by digest for production.

**g) Claude install script piped from curl (Low)**
Line 221: `curl -fsSL https://claude.ai/install.sh | bash` -- no verification. Standard practice for first-party install scripts, but worth noting.

### 2. `images/python/Dockerfile`

**a) `uv:latest` tag is unpinned (High)**
Line 15-16: `COPY --from=ghcr.io/astral-sh/uv:latest` uses a floating tag. A breaking change in uv could silently break builds. Pin to a specific version:
```dockerfile
ARG UV_VERSION=0.7.12
COPY --from=ghcr.io/astral-sh/uv:${UV_VERSION} /uv /usr/local/bin/uv
COPY --from=ghcr.io/astral-sh/uv:${UV_VERSION} /uvx /usr/local/bin/uvx
```

**b) `unzip` is redundant (Low)**
Line 11: `unzip` is already installed in the base image (line 125 of base/Dockerfile). Installing it again is harmless but adds noise and a small layer cost (apt will recognize it's already installed, but the `apt-get update` metadata is still fetched for this layer).

**c) `mkdocs<2` version constraint is broad (Low)**
Line 19: `'mkdocs<2'` allows any 1.x release. Consider pinning more tightly (e.g., `mkdocs==1.6.1`) for reproducibility.

**d) Cache mounts not used for apt-get (Low)**
Same as base image finding.

### 3. `images/latex/Dockerfile`

**a) TeX Live builder stage is duplicated (Medium)**
The entire TeX Live builder stage (lines 1-95) is copy-pasted identically across `latex/Dockerfile`, `python-latex/Dockerfile`, and `rust-latex/Dockerfile`. This is a maintenance burden -- any package list change must be applied in 3 places.

Recommended fix: Extract the TeX Live builder into a shared base image that all three can reference:
```dockerfile
# Build a shared texlive-builder image, then in each consumer:
COPY --from=ghcr.io/projectious-work/aibox:texlive-builder /usr/local/texlive /usr/local/texlive
```

**b) Missing fontconfig registration in latex/Dockerfile (Medium)**
The standalone `latex/Dockerfile` builder stage does NOT include the fontconfig registration that was added for issue #13:
```dockerfile
# This block is in latex/Dockerfile builder (line 94-95):
RUN ln -sf /usr/local/texlive/texmf-dist/fonts/opentype /usr/share/fonts/opentype-texlive \
    && fc-cache -f
```
However, this registration is done in the *builder* stage, not the *runtime* stage. The symlink and font cache in the builder are discarded because only `/usr/local/texlive` is copied to the runtime stage. The runtime stage needs its own fontconfig registration:
```dockerfile
# In the runtime stage, AFTER the COPY --from=texlive-builder:
RUN ln -sf /usr/local/texlive/texmf-dist/fonts/opentype /usr/share/fonts/opentype-texlive \
    && fc-cache -f
```
This same issue exists in `python-latex/Dockerfile` and `rust-latex/Dockerfile`, which don't even have the builder-side registration.

**c) `wget` used instead of `curl` in builder (Low)**
The builder installs `wget` to download the TeX Live installer, while the project convention (and all other Dockerfiles) uses `curl -fsSL`. Minor inconsistency.

**d) Cache mounts not used for apt-get (Low)**
Same as base image finding.

### 4. `images/typst/Dockerfile`

**Good overall.** Concise, single concern, version pinned via ARG.

**a) Cache mounts not used for apt-get (Low)**
Minor -- only installs `xz-utils`.

### 5. `images/rust/Dockerfile`

**a) Rustup installed via piped curl (Medium)**
Line 14: `curl ... | sh -s -- -y` is the standard rustup install method, but the practices doc recommends checksum verification for downloaded binaries. Consider downloading first, verifying, then executing:
```dockerfile
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh && \
    echo "<sha256>  /tmp/rustup-init.sh" | sha256sum -c - && \
    sh /tmp/rustup-init.sh -y --default-toolchain stable --component clippy --component rustfmt && \
    rm /tmp/rustup-init.sh
```

**b) Rust toolchain version not pinned (Medium)**
`--default-toolchain stable` installs whatever the current stable is. For reproducibility, pin to a specific version:
```dockerfile
ARG RUST_VERSION=1.87.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain ${RUST_VERSION} \
    --component clippy \
    --component rustfmt
```

**c) Cache mounts not used for apt-get (Low)**
Same as base image finding.

### 6. `images/python-latex/Dockerfile`

**a) TeX Live builder fully duplicated (Medium)**
Identical to latex/Dockerfile builder. See latex finding (a).

**b) Missing fontconfig registration in runtime (Medium)**
The builder stage here does NOT include the fontconfig registration present in `latex/Dockerfile` (lines 94-95). TeX Live OpenType fonts will not be discoverable by fontconfig in the runtime image. See latex finding (b).

### 7. `images/python-typst/Dockerfile`

**Good overall.** Identical structure to `typst/Dockerfile` but based on `python-latest`. No significant issues beyond the global cache mount finding.

### 8. `images/rust-latex/Dockerfile`

**a) TeX Live builder fully duplicated (Medium)**
Identical to latex/Dockerfile builder. See latex finding (a).

**b) Missing fontconfig registration in runtime (Medium)**
Same as python-latex. See latex finding (b).

## Cross-Cutting Issues

### 1. No `# syntax=docker/dockerfile:1` in any Dockerfile (Medium)
None of the 8 Dockerfiles include the BuildKit syntax directive. This is required to use cache mounts and other BuildKit features recommended in the practices doc.

### 2. `rm -rf /var/lib/apt/lists/*` used everywhere instead of cache mounts (Medium)
The practices doc explicitly says "With cache mounts, do NOT use `rm -rf /var/lib/apt/lists/*`." All Dockerfiles use the old pattern. Switching to cache mounts would speed up rebuilds when adding packages.

### 3. TeX Live builder duplicated in 3 files (Medium)
`latex/Dockerfile`, `python-latex/Dockerfile`, and `rust-latex/Dockerfile` share an identical ~90-line TeX Live builder stage. A change to the LaTeX package list must be applied in 3 places. Consider publishing a `texlive-builder` intermediate image or using a shared Dockerfile with build args.

### 4. No base image digest pinning (Low)
`debian:trixie-slim` is used by tag in 4 Dockerfiles. The GHCR base images (`ghcr.io/projectious-work/aibox:base-latest`, etc.) are also referenced by floating `latest` tags. For full reproducibility, pin by digest.

### 5. No checksum verification for any downloaded binary (Medium)
Affects: base (10 tools), typst, python-typst, rust (rustup). The practices doc requires "Always verify checksums for downloaded binaries."

## Priority Ranking

1. **High** -- Pin `uv:latest` to a specific version in `python/Dockerfile` (quick fix, high impact on reproducibility)
2. **Medium** -- Add fontconfig registration to runtime stages of latex, python-latex, rust-latex (functional bug -- fonts may not render correctly)
3. **Medium** -- Extract shared TeX Live builder to eliminate 3-way duplication (maintainability)
4. **Medium** -- Add checksum verification for downloaded binaries in base/Dockerfile (security)
5. **Medium** -- Pin Rust toolchain version in rust/Dockerfile (reproducibility)
6. **Medium** -- Add `# syntax=docker/dockerfile:1` and switch to cache mounts (build performance)
7. **Low** -- Consolidate COPY layers in base/Dockerfile (minor layer reduction)
8. **Low** -- Remove redundant `unzip` from python/Dockerfile (cleanup)
9. **Low** -- Pin base images by digest (full reproducibility)
