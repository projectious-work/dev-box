# syntax=docker/dockerfile:1
FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Go toolchain
ARG GO_VERSION=1.26.1
RUN ARCH=$(dpkg --print-architecture) && \
    curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-${ARCH}.tar.gz" \
    | tar -xz -C /usr/local && \
    ln -sf /usr/local/go/bin/* /usr/local/bin/

ENV GOPATH="/root/go"
ENV PATH="${GOPATH}/bin:${PATH}"

WORKDIR /workspace
CMD ["sleep", "infinity"]
