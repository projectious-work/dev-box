# =============================================================================
# Stage 1: Builder — fetch official Zellij release binary
# =============================================================================
FROM debian:trixie-slim AS builder

# Suppress debconf warnings during apt — no TTY available in docker build
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    unzip \
    && rm -rf /var/lib/apt/lists/*

ARG ZELLIJ_VERSION=0.44.0
ARG YAZI_VERSION=26.1.22

# Use uname -m for arch detection — works reliably across Docker, Podman, Buildah
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/zellij-org/zellij/releases/download/v${ZELLIJ_VERSION}/zellij-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/zellij && \
    curl -fsSL \
      "https://github.com/sxyazi/yazi/releases/download/v${YAZI_VERSION}/yazi-${ARCH}-unknown-linux-musl.zip" \
      -o /tmp/yazi.zip && \
    unzip -q /tmp/yazi.zip -d /tmp/yazi && \
    mv /tmp/yazi/yazi-${ARCH}-unknown-linux-musl/yazi /usr/local/bin/yazi && \
    chmod +x /usr/local/bin/yazi && \
    rm -rf /tmp/yazi /tmp/yazi.zip

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
    sudo \
    gosu \
    # Editor
    vim \
    # Version control
    git \
    lazygit \
    # Shell quality-of-life
    unzip \
    less \
    bash-completion \
    # GitHub CLI
    gh \
    # SSH client (for E2E test companion container access)
    openssh-client \
    # Audio support (for Claude Code voice)
    sox \
    libsox-fmt-pulse \
    pulseaudio-utils \
    libasound2-plugins \
    # Build essentials (for Rust compilation)
    build-essential \
    pkg-config \
    libssl-dev \
    # Cross-compilation for x86_64 Linux binary from aarch64 host
    gcc-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
    # Yazi preview dependencies
    chafa \
    poppler-utils \
    ffmpeg \
    imagemagick \
    ghostscript \
    timg \
    mupdf-tools \
    entr \
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

# ── Non-root user ───────────────────────────────────────────────────────────
RUN groupadd --gid 1000 aibox \
    && useradd --uid 1000 --gid aibox --shell /bin/bash --create-home aibox \
    && echo "aibox ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/aibox \
    && chmod 0440 /etc/sudoers.d/aibox

# Copy binaries from builder stage (system paths, must happen as root)
COPY --from=builder /usr/local/bin/zellij /usr/local/bin/zellij
COPY --from=builder /usr/local/bin/yazi /usr/local/bin/yazi

# ── uv — fast Python package manager ─────────────────────────────────────────
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv
COPY --from=ghcr.io/astral-sh/uv:latest /uvx /usr/local/bin/uvx

# ── Node.js (for Docusaurus docs build) — multi-stage copy, no headers ──────
COPY --from=node:22-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:22-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .aibox-home/.vim / .aibox-home/.config/zellij etc.
# over these paths, the host copy takes full precedence.

# Vim — place vimrc inside ~/.vim/ so the single mount covers config + undo
RUN mkdir -p /home/aibox/.vim/undo
COPY config/vimrc /home/aibox/.vim/vimrc

# Zellij
RUN mkdir -p /home/aibox/.config/zellij/themes /home/aibox/.config/zellij/layouts
COPY config/zellij/config.kdl          /home/aibox/.config/zellij/config.kdl
COPY config/zellij/themes/gruvbox.kdl  /home/aibox/.config/zellij/themes/gruvbox.kdl
COPY config/zellij/layouts/dev.kdl     /home/aibox/.config/zellij/layouts/dev.kdl
COPY config/zellij/layouts/focus.kdl   /home/aibox/.config/zellij/layouts/focus.kdl

# Yazi
RUN mkdir -p /home/aibox/.config/yazi/plugins
COPY config/yazi/yazi.toml             /home/aibox/.config/yazi/yazi.toml
COPY config/yazi/keymap.toml           /home/aibox/.config/yazi/keymap.toml
COPY config/yazi/theme.toml            /home/aibox/.config/yazi/theme.toml
COPY config/yazi/plugins/              /home/aibox/.config/yazi/plugins/

# Helper scripts
COPY config/bin/open-in-editor.sh      /usr/local/bin/open-in-editor
RUN chmod +x /usr/local/bin/open-in-editor

# Quick reference cheatsheet
COPY config/cheatsheet.txt             /home/aibox/.config/cheatsheet.txt

# Git — use XDG path so the mount target is a directory, not a single file
RUN mkdir -p /home/aibox/.config/git
COPY config/gitconfig /home/aibox/.config/git/config

# Audio — PulseAudio config for Claude Code voice support
COPY config/asoundrc /home/aibox/.asoundrc

# Fix ownership of all config files copied as root
RUN chown -R aibox:aibox /home/aibox

# ── Entrypoint for UID/GID remapping ───────────────────────────────────────
COPY --chmod=755 config/entrypoint.sh /usr/local/bin/entrypoint.sh

# ── Switch to non-root user for user-space installs ────────────────────────
USER aibox

# asciinema for terminal recordings (installed via uv)
RUN uv tool install asciinema

# ── Rust toolchain via rustup ─────────────────────────────────────────────────
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain stable \
    --component clippy \
    --component rustfmt \
    && . "$HOME/.cargo/env" \
    && rustup target add x86_64-unknown-linux-gnu \
    && cargo install cargo-audit

# Install code assistants and runtimes
ENV PATH="/home/aibox/.cargo/bin:/home/aibox/.local/bin:/home/aibox/.bun/bin:${PATH}"
RUN curl -fsSL https://claude.ai/install.sh | bash
RUN curl -fsSL https://bun.sh/install | bash

# ── Entrypoint ────────────────────────────────────────────────────────────────
# Switch back to root so the entrypoint can remap UID/GID if needed
USER root
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; zellij is launched via the terminal profile
CMD ["sleep", "infinity"]
