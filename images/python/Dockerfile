FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime
# For local builds, use: FROM dev-box-base:latest

ENV DEBIAN_FRONTEND=noninteractive

# Python 3.13 + pip + venv
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    python3-venv \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# uv — fast Python package manager
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv
COPY --from=ghcr.io/astral-sh/uv:latest /uvx /usr/local/bin/uvx

# MkDocs Material (installed via uv into a global tool environment)
RUN uv tool install 'mkdocs<2' --with mkdocs-material

ENV PATH="/root/.local/bin:${PATH}"

WORKDIR /workspace
CMD ["sleep", "infinity"]
