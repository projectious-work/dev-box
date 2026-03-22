# syntax=docker/dockerfile:1
FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Node.js 22 LTS via NodeSource
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y --no-install-recommends nodejs && \
    rm -rf /var/lib/apt/lists/* && \
    npm install -g pnpm

WORKDIR /workspace
CMD ["sleep", "infinity"]
