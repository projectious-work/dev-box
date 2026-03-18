# =============================================================================
# Stage 1: Builder — fetch official Zellij release binary
# =============================================================================
FROM debian:trixie-slim AS builder

# Suppress debconf warnings during apt — no TTY available in docker build
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG ZELLIJ_VERSION=0.43.1

# Use uname -m for arch detection — works reliably across Docker, Podman, Buildah
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/zellij-org/zellij/releases/download/v${ZELLIJ_VERSION}/zellij-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/zellij

# =============================================================================
# Stage 2: Runtime — minimal dev-container
# =============================================================================
FROM debian:trixie-slim AS runtime

# Suppress debconf warnings during build — no TTY available in docker build
ENV DEBIAN_FRONTEND=noninteractive

# Set timezone early so tzdata installs silently
ENV TZ=Europe/Berlin

# Explicitly declare path exclusions before any package is installed.
# debian:trixie-slim already strips man pages, but making it explicit here
# silences the update-alternatives warnings about missing man page symlinks
# that would otherwise appear during vim installation.
RUN printf 'path-exclude=/usr/share/man/*\n\
path-exclude=/usr/share/groff/*\n\
path-exclude=/usr/share/info/*\n\
path-exclude=/usr/share/lintian/*\n' \
    > /etc/dpkg/dpkg.cfg.d/excludes

RUN apt-get update && apt-get install -y --no-install-recommends \
    # Core utilities
    curl \
    ca-certificates \
    tzdata \
    locales \
    jq \
    # Editor
    vim \
    # Version control
    git \
    lazygit \
    # Shell quality-of-life
    less \
    bash-completion \
    # GitHub CLI
    gh \
    # Audio support (for Claude Code voice)
    sox \
    pulseaudio-utils \
    # Build essentials (for Rust compilation)
    build-essential \
    pkg-config \
    libssl-dev \
    # Python
    python3 \
    python3-pip \
    python3-venv \
    && sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \
    && locale-gen \
    && ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime \
    && echo ${TZ} > /etc/timezone \
    && rm -rf /var/lib/apt/lists/*

# Set locale vars only after locale-gen has run, so perl/dpkg never see
# an LC_ALL pointing to a non-existent locale during the build
ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8 \
    TERM=xterm-256color \
    COLORTERM=truecolor

# Copy Zellij binary from builder stage
COPY --from=builder /usr/local/bin/zellij /usr/local/bin/zellij

# ── uv — fast Python package manager ─────────────────────────────────────────
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv
COPY --from=ghcr.io/astral-sh/uv:latest /uvx /usr/local/bin/uvx

# MkDocs Material (installed via uv into a global tool environment)
RUN uv tool install 'mkdocs<2' --with mkdocs-material

# ── Rust toolchain via rustup ─────────────────────────────────────────────────
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain stable \
    --component clippy \
    --component rustfmt

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .root/.vim / .root/.config/zellij / .root/.config/git
# over these paths, the host copy takes full precedence.

# Vim — place vimrc inside ~/.vim/ so the single mount covers config + undo
RUN mkdir -p /root/.vim/undo
COPY config/vimrc /root/.vim/vimrc

# Zellij
RUN mkdir -p /root/.config/zellij/themes /root/.config/zellij/layouts
COPY config/zellij/config.kdl          /root/.config/zellij/config.kdl
COPY config/zellij/themes/gruvbox.kdl  /root/.config/zellij/themes/gruvbox.kdl
COPY config/zellij/layouts/dev.kdl     /root/.config/zellij/layouts/dev.kdl

# Git — use XDG path so the mount target is a directory, not a single file
RUN mkdir -p /root/.config/git
COPY config/gitconfig /root/.config/git/config

# Audio — PulseAudio config for Claude Code voice support
COPY config/asoundrc /root/.asoundrc

# Install code assistants
ENV PATH="/root/.cargo/bin:/root/.local/bin:${PATH}"
RUN curl -fsSL https://claude.ai/install.sh | bash

# ── Entrypoint ────────────────────────────────────────────────────────────────
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; zellij is launched via the terminal profile
CMD ["sleep", "infinity"]