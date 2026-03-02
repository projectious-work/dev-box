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

# TARGETARCH is injected automatically by Docker BuildKit: "amd64" or "arm64"
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

# Suppress debconf warnings during build — no TTY available in docker build.
# Unset after apt so runtime behaviour is unaffected.
ENV DEBIAN_FRONTEND=noninteractive

# Set timezone early so tzdata installs silently
ENV TZ=Europe/Berlin

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
    && sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen \
    && locale-gen \
    && ln -snf /usr/share/zoneinfo/${TZ} /etc/localtime \
    && echo ${TZ} > /etc/timezone \
    && rm -rf /var/lib/apt/lists/* \
    # Remove build-time frontend override — runtime should behave normally
    && unset DEBIAN_FRONTEND || true

# Set locale vars only after locale-gen has run, so perl/dpkg never see
# an LC_ALL that points to a non-existent locale during the build
ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8 \
    TERM=xterm-256color \
    COLORTERM=truecolor

# Copy Zellij binary from builder stage
COPY --from=builder /usr/local/bin/zellij /usr/local/bin/zellij

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

# ── Entrypoint ────────────────────────────────────────────────────────────────
WORKDIR /workspace

# Keep the container alive — VS Code execs into it; zellij is launched via the terminal profile
CMD ["sleep", "infinity"]