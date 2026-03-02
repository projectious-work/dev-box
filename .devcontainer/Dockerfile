# =============================================================================
# Stage 1: Builder — fetch official Zellij release binary
# =============================================================================
FROM debian:trixie-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# TARGETARCH is injected automatically by Docker BuildKit: "amd64" or "arm64"
ARG ZELLIJ_VERSION=0.43.1

RUN ARCH=$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch64/') && \
    curl -fsSL \
      "https://github.com/zellij-org/zellij/releases/download/v${ZELLIJ_VERSION}/zellij-${ARCH}-unknown-linux-musl.tar.gz" \
      | tar -xz -C /usr/local/bin && \
    chmod +x /usr/local/bin/zellij


# =============================================================================
# Stage 2: Runtime — minimal dev-container
# =============================================================================
FROM debian:trixie-slim AS runtime

# ── Locale & timezone ────────────────────────────────────────────────────────
ENV LANG=en_US.UTF-8 \
    LANGUAGE=en_US:en \
    LC_ALL=en_US.UTF-8 \
    TZ=Europe/Berlin \
    TERM=xterm-256color \
    COLORTERM=truecolor

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
    && rm -rf /var/lib/apt/lists/*

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

CMD ["zellij", "--layout", "dev"]
