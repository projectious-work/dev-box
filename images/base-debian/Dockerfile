# syntax=docker/dockerfile:1
# =============================================================================
# Builder stages — one per tool for parallel BuildKit downloads and granular
# cache invalidation. Bumping one version no longer re-downloads all others.
# =============================================================================
FROM debian:trixie-slim AS fetch-base

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates unzip \
    && rm -rf /var/lib/apt/lists/*

# ── Zellij ───────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-zellij
ARG ZELLIJ_VERSION=0.43.1
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/zellij-org/zellij/releases/download/v${ZELLIJ_VERSION}/zellij-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/zellij

# ── Yazi ─────────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-yazi
ARG YAZI_VERSION=25.4.8
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/sxyazi/yazi/releases/download/v${YAZI_VERSION}/yazi-${ARCH}-unknown-linux-musl.zip" \
      -o /tmp/yazi.zip && \
    unzip -q /tmp/yazi.zip -d /tmp/yazi && \
    mv /tmp/yazi/yazi-${ARCH}-unknown-linux-musl/yazi /usr/local/bin/yazi && \
    chmod +x /usr/local/bin/yazi && \
    rm -rf /tmp/yazi /tmp/yazi.zip

# ── ripgrep (no aarch64 musl build — use gnu) ────────────────────────────────
FROM fetch-base AS fetch-ripgrep
ARG RIPGREP_VERSION=14.1.1
RUN ARCH=$(uname -m) && \
    RG_LIBC=$(case $ARCH in x86_64) echo musl;; *) echo gnu;; esac) && \
    curl -fsSL \
      "https://github.com/BurntSushi/ripgrep/releases/download/${RIPGREP_VERSION}/ripgrep-${RIPGREP_VERSION}-${ARCH}-unknown-linux-${RG_LIBC}.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "ripgrep-${RIPGREP_VERSION}-${ARCH}-unknown-linux-${RG_LIBC}/rg" && \
    chmod +x /usr/local/bin/rg

# ── fd ───────────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-fd
ARG FD_VERSION=10.2.0
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/sharkdp/fd/releases/download/v${FD_VERSION}/fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl/fd" && \
    chmod +x /usr/local/bin/fd

# ── bat ──────────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-bat
ARG BAT_VERSION=0.25.0
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/sharkdp/bat/releases/download/v${BAT_VERSION}/bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl/bat" && \
    chmod +x /usr/local/bin/bat

# ── eza (no aarch64 musl build — use gnu) ────────────────────────────────────
FROM fetch-base AS fetch-eza
ARG EZA_VERSION=0.20.18
RUN ARCH=$(uname -m) && \
    EZA_SUFFIX=$(case $ARCH in x86_64) echo "unknown-linux-musl";; aarch64) echo "unknown-linux-gnu";; esac) && \
    curl -fsSL \
      "https://github.com/eza-community/eza/releases/download/v${EZA_VERSION}/eza_${ARCH}-${EZA_SUFFIX}.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/eza

# ── zoxide ───────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-zoxide
ARG ZOXIDE_VERSION=0.9.6
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/ajeetdsouza/zoxide/releases/download/v${ZOXIDE_VERSION}/zoxide-${ZOXIDE_VERSION}-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin zoxide && \
    chmod +x /usr/local/bin/zoxide

# ── fzf (uses amd64/arm64 instead of x86_64/aarch64) ────────────────────────
FROM fetch-base AS fetch-fzf
ARG FZF_VERSION=0.60.3
RUN ARCH=$(uname -m) && \
    FZF_ARCH=$(case $ARCH in aarch64) echo arm64;; x86_64) echo amd64;; esac) && \
    curl -fsSL \
      "https://github.com/junegunn/fzf/releases/download/v${FZF_VERSION}/fzf-${FZF_VERSION}-linux_${FZF_ARCH}.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/fzf

# ── delta (no aarch64 musl build — use gnu) ──────────────────────────────────
FROM fetch-base AS fetch-delta
ARG DELTA_VERSION=0.18.2
RUN ARCH=$(uname -m) && \
    DELTA_LIBC=$(case $ARCH in x86_64) echo musl;; *) echo gnu;; esac) && \
    curl -fsSL \
      "https://github.com/dandavison/delta/releases/download/${DELTA_VERSION}/delta-${DELTA_VERSION}-${ARCH}-unknown-linux-${DELTA_LIBC}.tar.gz" \
      | tar -xz --strip-components=1 -C /usr/local/bin \
        "delta-${DELTA_VERSION}-${ARCH}-unknown-linux-${DELTA_LIBC}/delta" && \
    chmod +x /usr/local/bin/delta

# ── starship ─────────────────────────────────────────────────────────────────
FROM fetch-base AS fetch-starship
ARG STARSHIP_VERSION=1.22.1
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/starship/starship/releases/download/v${STARSHIP_VERSION}/starship-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/starship

# ── Vim colorschemes ─────────────────────────────────────────────────────────
# Separate stage keeps network fetches out of the runtime build graph.
# Uses curl for dracula instead of git clone (~4 MB savings).
FROM fetch-base AS fetch-vim-colors
RUN mkdir -p /vim-colors/colors \
      /vim-colors/pack/themes/start/dracula/colors \
      /vim-colors/pack/themes/start/dracula/autoload && \
    curl -fsSL -o /vim-colors/colors/gruvbox.vim \
      https://raw.githubusercontent.com/morhetz/gruvbox/master/colors/gruvbox.vim && \
    curl -fsSL -o /vim-colors/colors/catppuccin_mocha.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_mocha.vim && \
    curl -fsSL -o /vim-colors/colors/catppuccin_latte.vim \
      https://raw.githubusercontent.com/catppuccin/vim/main/colors/catppuccin_latte.vim && \
    curl -fsSL -o /vim-colors/colors/tokyonight.vim \
      https://raw.githubusercontent.com/ghifarit53/tokyonight-vim/master/colors/tokyonight.vim && \
    curl -fsSL -o /vim-colors/colors/nord.vim \
      https://raw.githubusercontent.com/nordtheme/vim/main/colors/nord.vim && \
    curl -fsSL -o /vim-colors/pack/themes/start/dracula/colors/dracula.vim \
      https://raw.githubusercontent.com/dracula/vim/master/colors/dracula.vim && \
    curl -fsSL -o /vim-colors/pack/themes/start/dracula/autoload/dracula.vim \
      https://raw.githubusercontent.com/dracula/vim/master/autoload/dracula.vim

# =============================================================================
# Runtime — minimal dev-container base image
# =============================================================================
# Estimated layer sizes (arm64, approximate):
#   debian:trixie-slim base .............. ~75 MB
#   Core apt (curl, git, vim, etc.) ...... ~120 MB
#   Preview deps (ffmpeg, imagemagick) ... ~150 MB
#   Audio (sox, pulseaudio) .............. ~25 MB
#   Static binaries (10 tools) ........... ~95 MB
#   Vim colorschemes ..................... ~0.1 MB
#   Config files ......................... ~0.1 MB
#   ─────────────────────────────────────────────
#   Total estimate ....................... ~465 MB
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

# ── System packages ──────────────────────────────────────────────────────────
# Kept as a single RUN to share one apt-get update and minimize layer overhead.
# Groups are documented for future reference if slim image variants are needed.
RUN apt-get update && apt-get install -y --no-install-recommends \
    # ── Core utilities (~35 MB) ──
    curl \
    ca-certificates \
    tzdata \
    locales \
    jq \
    sudo \
    gosu \
    # ── Editor (~25 MB) ──
    vim \
    # ── Version control (~55 MB) ──
    git \
    lazygit \
    # ── Shell quality-of-life (~5 MB) ──
    less \
    unzip \
    file \
    bash-completion \
    # ── GitHub CLI (~15 MB) ──
    gh \
    # ── Yazi preview dependencies (~150 MB — heaviest group) ──
    chafa \
    poppler-utils \
    ffmpeg \
    imagemagick \
    # ── Audio support for Claude Code voice (~25 MB) ──
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

# ── Non-root user ───────────────────────────────────────────────────────────
RUN groupadd --gid 1000 aibox \
    && useradd --uid 1000 --gid aibox --shell /bin/bash --create-home aibox \
    && echo "aibox ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/aibox \
    && chmod 0440 /etc/sudoers.d/aibox

# ── Copy binaries from parallel builder stages ──────────────────────────────
COPY --from=fetch-zellij   /usr/local/bin/zellij   /usr/local/bin/
COPY --from=fetch-yazi     /usr/local/bin/yazi     /usr/local/bin/
COPY --from=fetch-ripgrep  /usr/local/bin/rg       /usr/local/bin/
COPY --from=fetch-fd       /usr/local/bin/fd       /usr/local/bin/
COPY --from=fetch-bat      /usr/local/bin/bat      /usr/local/bin/
COPY --from=fetch-eza      /usr/local/bin/eza      /usr/local/bin/
COPY --from=fetch-zoxide   /usr/local/bin/zoxide   /usr/local/bin/
COPY --from=fetch-fzf      /usr/local/bin/fzf      /usr/local/bin/
COPY --from=fetch-delta    /usr/local/bin/delta    /usr/local/bin/
COPY --from=fetch-starship /usr/local/bin/starship /usr/local/bin/

# ── Bake default configs into the image ───────────────────────────────────────
# These act as fallback defaults when no host directory is mounted.
# When docker-compose mounts .aibox-home/.vim / .aibox-home/.config/zellij etc.
# over these paths, the host copy takes full precedence.

# Vim — colorschemes from builder stage, vimrc from build context
RUN mkdir -p /home/aibox/.vim/undo
COPY --from=fetch-vim-colors /vim-colors/colors/                                 /home/aibox/.vim/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/colors/       /home/aibox/.vim/pack/themes/start/dracula/colors/
COPY --from=fetch-vim-colors /vim-colors/pack/themes/start/dracula/autoload/     /home/aibox/.vim/pack/themes/start/dracula/autoload/
COPY config/vimrc /home/aibox/.vim/vimrc

# Create config directories in a single layer
RUN mkdir -p /home/aibox/.config/zellij/themes /home/aibox/.config/zellij/layouts \
             /home/aibox/.config/yazi \
             /home/aibox/.config/git

# Zellij
COPY config/zellij/config.kdl          /home/aibox/.config/zellij/config.kdl
COPY config/zellij/themes/             /home/aibox/.config/zellij/themes/
COPY config/zellij/layouts/dev.kdl     /home/aibox/.config/zellij/layouts/dev.kdl
COPY config/zellij/layouts/focus.kdl   /home/aibox/.config/zellij/layouts/focus.kdl
COPY config/zellij/layouts/cowork.kdl  /home/aibox/.config/zellij/layouts/cowork.kdl

# Yazi
COPY config/yazi/yazi.toml             /home/aibox/.config/yazi/yazi.toml
COPY config/yazi/keymap.toml           /home/aibox/.config/yazi/keymap.toml
COPY config/yazi/theme.toml            /home/aibox/.config/yazi/theme.toml
COPY config/yazi/themes/               /home/aibox/.config/yazi/themes/

# Helper scripts (chmod via COPY --chmod to avoid extra layer)
COPY --chmod=755 config/bin/open-in-editor.sh /usr/local/bin/open-in-editor
COPY --chmod=755 config/bin/vim-loop.sh       /usr/local/bin/vim-loop

# Quick reference cheatsheet
COPY config/cheatsheet.txt             /home/aibox/.config/cheatsheet.txt

# Git — use XDG path so the mount target is a directory, not a single file
COPY config/gitconfig /home/aibox/.config/git/config

# Audio — PulseAudio config for Claude Code voice support
COPY config/asoundrc /home/aibox/.asoundrc

# Shell — bashrc with aliases and tool integrations
COPY config/bashrc /home/aibox/.bashrc

# Fix ownership of all config files copied as root
RUN chown -R aibox:aibox /home/aibox

# ── Entrypoint for UID/GID remapping ───────────────────────────────────────
COPY --chmod=755 config/entrypoint.sh /usr/local/bin/entrypoint.sh

# ── Ensure user-local bin is on PATH ────────────────────────────────────────
# AI coding agents (Claude, Aider, etc.) are installed per-project via addons,
# not baked into the base image — see DEC-016.
ENV PATH="/home/aibox/.local/bin:${PATH}"

# ── Entrypoint ────────────────────────────────────────────────────────────────
# Root so the entrypoint can remap UID/GID if needed before dropping to aibox
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; zellij is launched via the terminal profile
CMD ["sleep", "infinity"]
