# syntax=docker/dockerfile:1
# =============================================================================
# Stage 1: Builder — fetch static binaries for shell tools
# =============================================================================
FROM debian:trixie-slim AS builder

# Suppress debconf warnings during apt — no TTY available in docker build
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    unzip \
    && rm -rf /var/lib/apt/lists/*

ARG ZELLIJ_VERSION=0.43.1
ARG YAZI_VERSION=25.4.8
ARG RIPGREP_VERSION=14.1.1
ARG FD_VERSION=10.2.0
ARG BAT_VERSION=0.25.0
ARG EZA_VERSION=0.20.18
ARG ZOXIDE_VERSION=0.9.6
ARG FZF_VERSION=0.60.3
ARG DELTA_VERSION=0.18.2
ARG STARSHIP_VERSION=1.22.1

# Use uname -m for arch detection — works reliably across Docker, Podman, Buildah
RUN ARCH=$(uname -m) && \
    # ── Zellij ──────────────────────────────────────────────────────────────
    curl -fsSL \
      "https://github.com/zellij-org/zellij/releases/download/v${ZELLIJ_VERSION}/zellij-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/zellij && \
    # ── Yazi ────────────────────────────────────────────────────────────────
    curl -fsSL \
      "https://github.com/sxyazi/yazi/releases/download/v${YAZI_VERSION}/yazi-${ARCH}-unknown-linux-musl.zip" \
      -o /tmp/yazi.zip && \
    unzip -q /tmp/yazi.zip -d /tmp/yazi && \
    mv /tmp/yazi/yazi-${ARCH}-unknown-linux-musl/yazi /usr/local/bin/yazi && \
    chmod +x /usr/local/bin/yazi && \
    rm -rf /tmp/yazi /tmp/yazi.zip && \
    # ── ripgrep (rg) — no aarch64 musl build, use gnu ───────────────────────
    RG_LIBC=$(case $ARCH in x86_64) echo musl;; *) echo gnu;; esac) && \
    curl -fsSL \
      "https://github.com/BurntSushi/ripgrep/releases/download/${RIPGREP_VERSION}/ripgrep-${RIPGREP_VERSION}-${ARCH}-unknown-linux-${RG_LIBC}.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "ripgrep-${RIPGREP_VERSION}-${ARCH}-unknown-linux-${RG_LIBC}/rg" && \
    chmod +x /usr/local/bin/rg && \
    # ── fd ──────────────────────────────────────────────────────────────────
    curl -fsSL \
      "https://github.com/sharkdp/fd/releases/download/v${FD_VERSION}/fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl/fd" && \
    chmod +x /usr/local/bin/fd && \
    # ── bat ─────────────────────────────────────────────────────────────────
    curl -fsSL \
      "https://github.com/sharkdp/bat/releases/download/v${BAT_VERSION}/bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl/bat" && \
    chmod +x /usr/local/bin/bat && \
    # ── eza (musl for x86_64, gnu for aarch64 — no aarch64 musl build) ────
    EZA_SUFFIX=$(case $ARCH in x86_64) echo "unknown-linux-musl";; aarch64) echo "unknown-linux-gnu";; esac) && \
    curl -fsSL \
      "https://github.com/eza-community/eza/releases/download/v${EZA_VERSION}/eza_${ARCH}-${EZA_SUFFIX}.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/eza && \
    # ── zoxide ──────────────────────────────────────────────────────────────
    curl -fsSL \
      "https://github.com/ajeetdsouza/zoxide/releases/download/v${ZOXIDE_VERSION}/zoxide-${ZOXIDE_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin zoxide && \
    chmod +x /usr/local/bin/zoxide && \
    # ── fzf (uses amd64/arm64 instead of x86_64/aarch64) ───────────────────
    FZF_ARCH=$(case $ARCH in aarch64) echo arm64;; x86_64) echo amd64;; esac) && \
    curl -fsSL \
      "https://github.com/junegunn/fzf/releases/download/v${FZF_VERSION}/fzf-${FZF_VERSION}-linux_${FZF_ARCH}.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/fzf && \
    # ── delta — no aarch64 musl build, use gnu ───────────────────────────────
    DELTA_LIBC=$(case $ARCH in x86_64) echo musl;; *) echo gnu;; esac) && \
    curl -fsSL \
      "https://github.com/dandavison/delta/releases/download/${DELTA_VERSION}/delta-${DELTA_VERSION}-${ARCH}-unknown-linux-${DELTA_LIBC}.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "delta-${DELTA_VERSION}-${ARCH}-unknown-linux-${DELTA_LIBC}/delta" && \
    chmod +x /usr/local/bin/delta && \
    # ── starship ────────────────────────────────────────────────────────────
    curl -fsSL \
      "https://github.com/starship/starship/releases/download/v${STARSHIP_VERSION}/starship-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/starship

# =============================================================================
# Stage 2: Runtime — minimal dev-container base image
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
    unzip \
    file \
    bash-completion \
    # GitHub CLI
    gh \
    # Audio support (for Claude Code voice)
    sox \
    libsox-fmt-pulse \
    pulseaudio-utils \
    libasound2-plugins \
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

# Copy binaries from builder stage
# Copy all tool binaries from builder (single layer)
COPY --from=builder /usr/local/bin/zellij /usr/local/bin/yazi \
    /usr/local/bin/rg /usr/local/bin/fd /usr/local/bin/bat \
    /usr/local/bin/eza /usr/local/bin/zoxide /usr/local/bin/fzf \
    /usr/local/bin/delta /usr/local/bin/starship /usr/local/bin/

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .root/.vim / .root/.config/zellij / .root/.config/git
# over these paths, the host copy takes full precedence.

# Vim — place vimrc inside ~/.vim/ so the single mount covers config + undo
# Download all colorschemes in one layer; use curl for dracula instead of
# git clone to avoid pulling the entire repo (~4 MB savings)
RUN mkdir -p /root/.vim/undo /root/.vim/colors \
      /root/.vim/pack/themes/start/dracula/colors \
      /root/.vim/pack/themes/start/dracula/autoload && \
    curl -fsSL -o /root/.vim/colors/gruvbox.vim \
      https://raw.githubusercontent.com/morhetz/gruvbox/master/colors/gruvbox.vim && \
    curl -fsSL -o /root/.vim/colors/catppuccin_mocha.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_mocha.vim && \
    curl -fsSL -o /root/.vim/colors/catppuccin_latte.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_latte.vim && \
    curl -fsSL -o /root/.vim/colors/tokyonight.vim \
      https://raw.githubusercontent.com/ghifarit53/tokyonight-vim/master/colors/tokyonight.vim && \
    curl -fsSL -o /root/.vim/colors/nord.vim \
      https://raw.githubusercontent.com/nordtheme/vim/main/colors/nord.vim && \
    curl -fsSL -o /root/.vim/pack/themes/start/dracula/colors/dracula.vim \
      https://raw.githubusercontent.com/dracula/vim/master/colors/dracula.vim && \
    curl -fsSL -o /root/.vim/pack/themes/start/dracula/autoload/dracula.vim \
      https://raw.githubusercontent.com/dracula/vim/master/autoload/dracula.vim

COPY config/vimrc /root/.vim/vimrc

# Create config directories in a single layer
RUN mkdir -p /root/.config/zellij/themes /root/.config/zellij/layouts \
             /root/.config/yazi \
             /root/.config/git

# Zellij
COPY config/zellij/config.kdl          /root/.config/zellij/config.kdl
COPY config/zellij/themes/             /root/.config/zellij/themes/
COPY config/zellij/layouts/dev.kdl     /root/.config/zellij/layouts/dev.kdl
COPY config/zellij/layouts/focus.kdl   /root/.config/zellij/layouts/focus.kdl
COPY config/zellij/layouts/cowork.kdl  /root/.config/zellij/layouts/cowork.kdl

# Yazi
COPY config/yazi/yazi.toml             /root/.config/yazi/yazi.toml
COPY config/yazi/keymap.toml           /root/.config/yazi/keymap.toml
COPY config/yazi/theme.toml            /root/.config/yazi/theme.toml
COPY config/yazi/themes/               /root/.config/yazi/themes/

# Helper scripts (chmod via COPY --chmod to avoid extra layer)
COPY --chmod=755 config/bin/open-in-editor.sh /usr/local/bin/open-in-editor
COPY --chmod=755 config/bin/vim-loop.sh       /usr/local/bin/vim-loop

# Quick reference cheatsheet
COPY config/cheatsheet.txt             /root/.config/cheatsheet.txt

# Git — use XDG path so the mount target is a directory, not a single file
COPY config/gitconfig /root/.config/git/config

# Audio — PulseAudio config for Claude Code voice support
COPY config/asoundrc /root/.asoundrc

# Shell — bashrc with aliases and tool integrations
COPY config/bashrc /root/.bashrc

# Install code assistants
ENV PATH="/root/.local/bin:${PATH}"
RUN curl -fsSL https://claude.ai/install.sh | bash

# ── Entrypoint ────────────────────────────────────────────────────────────────
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; zellij is launched via the terminal profile
CMD ["sleep", "infinity"]
