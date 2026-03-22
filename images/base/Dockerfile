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

ARG ZELLIJ_VERSION=0.43.1
ARG YAZI_VERSION=25.4.8

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
COPY --from=builder /usr/local/bin/zellij /usr/local/bin/zellij
COPY --from=builder /usr/local/bin/yazi /usr/local/bin/yazi

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .root/.vim / .root/.config/zellij / .root/.config/git
# over these paths, the host copy takes full precedence.

# Vim — place vimrc inside ~/.vim/ so the single mount covers config + undo
RUN mkdir -p /root/.vim/undo /root/.vim/colors

# Download Vim colorschemes for all supported themes
RUN curl -fsSL -o /root/.vim/colors/gruvbox.vim \
      https://raw.githubusercontent.com/morhetz/gruvbox/master/colors/gruvbox.vim && \
    curl -fsSL -o /root/.vim/colors/catppuccin_mocha.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_mocha.vim && \
    curl -fsSL -o /root/.vim/colors/catppuccin_latte.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_latte.vim && \
    curl -fsSL -o /root/.vim/colors/tokyonight.vim \
      https://raw.githubusercontent.com/ghifarit53/tokyonight-vim/master/colors/tokyonight.vim && \
    curl -fsSL -o /root/.vim/colors/nord.vim \
      https://raw.githubusercontent.com/nordtheme/vim/main/colors/nord.vim && \
    mkdir -p /root/.vim/pack/themes/start && \
    git clone --depth 1 https://github.com/dracula/vim.git \
      /root/.vim/pack/themes/start/dracula

COPY config/vimrc /root/.vim/vimrc

# Zellij
RUN mkdir -p /root/.config/zellij/themes /root/.config/zellij/layouts
COPY config/zellij/config.kdl          /root/.config/zellij/config.kdl
COPY config/zellij/themes/             /root/.config/zellij/themes/
COPY config/zellij/layouts/dev.kdl     /root/.config/zellij/layouts/dev.kdl
COPY config/zellij/layouts/focus.kdl   /root/.config/zellij/layouts/focus.kdl
COPY config/zellij/layouts/cowork.kdl  /root/.config/zellij/layouts/cowork.kdl

# Yazi
RUN mkdir -p /root/.config/yazi
COPY config/yazi/yazi.toml             /root/.config/yazi/yazi.toml
COPY config/yazi/keymap.toml           /root/.config/yazi/keymap.toml
COPY config/yazi/theme.toml            /root/.config/yazi/theme.toml

# Helper scripts
COPY config/bin/open-in-editor.sh      /usr/local/bin/open-in-editor
COPY config/bin/vim-loop.sh            /usr/local/bin/vim-loop
RUN chmod +x /usr/local/bin/open-in-editor /usr/local/bin/vim-loop

# Quick reference cheatsheet
COPY config/cheatsheet.txt             /root/.config/cheatsheet.txt

# Git — use XDG path so the mount target is a directory, not a single file
RUN mkdir -p /root/.config/git
COPY config/gitconfig /root/.config/git/config

# Audio — PulseAudio config for Claude Code voice support
COPY config/asoundrc /root/.asoundrc

# Install code assistants
ENV PATH="/root/.local/bin:${PATH}"
RUN curl -fsSL https://claude.ai/install.sh | bash

# ── Entrypoint ────────────────────────────────────────────────────────────────
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; zellij is launched via the terminal profile
CMD ["sleep", "infinity"]
