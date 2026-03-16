FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime
# For local builds, use: FROM dev-box-base:latest

ARG TYPST_VERSION=0.13.1

# Install Typst from GitHub release (static musl binary, no dependencies)
RUN ARCH=$(uname -m) && \
    curl -fsSL \
      "https://github.com/typst/typst/releases/download/v${TYPST_VERSION}/typst-${ARCH}-unknown-linux-musl.tar.xz" \
      | tar -xJ --strip-components=1 -C /usr/local/bin \
        "typst-${ARCH}-unknown-linux-musl/typst" && \
    chmod +x /usr/local/bin/typst

# Typst packages are downloaded on first use from https://packages.typst.org
# and cached at ~/.cache/typst/packages/. Mount .root/.cache/typst via compose
# to persist the package cache across container rebuilds.

WORKDIR /workspace
CMD ["sleep", "infinity"]
