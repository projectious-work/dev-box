FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime
# For local builds, use: FROM dev-box-base:latest

ENV DEBIAN_FRONTEND=noninteractive

# Build essentials for Rust compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain stable \
    --component clippy \
    --component rustfmt

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /workspace
CMD ["sleep", "infinity"]
