FROM ubuntu:26.04 as builder

ENV DEBIAN_FRONTEND=noninteractive

RUN    apt-get update \ 
    && apt-get install -y \
       git \
       bash \
       curl \
       build-essential \
       zig \
       ffmpeg \
       7zip \
       jq \ 
       poppler-utils \
       fd-find \
       ripgrep \
       fzf \
       zoxide \
       imagemagick

RUN apt-get install -y \
    pkg-config \
    libfreetype6-dev \ 
    libfontconfig1-dev 

RUN    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /tmp

# Build Helix 
# not pre-built binary because it is built with 
# a helix default runtime directory pointing to ~/.config/helix/runtime
# which is not suitable for a containerized dev-environment with the 
# helix config shared with the host as it fills the config with runtime files.
ENV HELIX_DEFAULT_RUNTIME=/usr/share/helix/runtime
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN git clone https://github.com/helix-editor/helix \
    && cd helix \
    && CARGO_BUILD_JOBS=2 cargo install \
       --profile opt \
       --config 'build.rustflags=["-C", "target-cpu=native"]' \
       --path helix-term \
       --locked

# Build fancy-cat with zig
# RUN mkdir /tmp/fancy-cat \
#     && cd /tmp/fancy-cat \ 
#     && git clone --recursive https://github.com/freref/fancy-cat.git . \
#     && zig build --release=small

RUN apt-get install -y \
    unzip \
    python3 \
    clang \
    llvm-dev \
    libclang-dev

# Build tdf with rust
RUN mkdir /tmp/tdf \
    && cd /tmp/tdf \ 
    && git clone https://github.com/itsjunetime/tdf.git . \
    && cargo build --release


# Build yazi filemanagaer
RUN mkdir /tmp/yazi \
    && cd /tmp/yazi \ 
    && git clone https://github.com/sxyazi/yazi.git \
    && cargo build --release --locked

# RUN cargo install --git https://github.com/itsjunetime/tdf.git

# RUN    hx --grammar fetch \
#     && hx --grammar build 
    # && rm -fr /root/.config/helix/runtime/grammars/sources

RUN cargo install --git https://github.com/latex-lsp/texlab --locked --tag "v5.24.0"

RUN cargo install --locked zellij

RUN curl -fsSL https://opencode.ai/install | bash

# CMD ["cp", "zig-out/bin/fancy-cat", "/output/"]



FROM ubuntu:26.04

RUN    apt-get update \ 
    && apt-get install -y \
       git \
       bash \
       lazygit \
       fontconfig \  
       ffmpeg \
       7zip \
       jq \
       poppler-utils \ 
       fd-find \
       ripgrep \
       fzf \
       zoxide \
       imagemagick \  
    && apt-get clean


# Tooling from builder stage
COPY --from=builder /root/.cargo/bin/zellij /usr/bin/zellij
COPY --from=builder /tmp/tdf/target/release/tdf /usr/local/bin/tdf
COPY --from=builder /tmp/yazi/target/release/yazi /usr/local/bin/yazi
COPY --from=builder /tmp/yazi/target/release/ya /usr/local/bin/ya

COPY --from=builder /tmp/helix/runtime/ /usr/share/helix/runtime/
COPY --from=builder /tmp/helix/target/opt/hx /usr/bin/hx
COPY --from=builder /root/.opencode/bin/* /usr/bin/

# Language servers
COPY --from=builder /root/.cargo/bin/texlab /usr/local/bin/texlab

# Formatters
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hougesen/kdlfmt/releases/latest/download/kdlfmt-installer.sh | sh

RUN git config --global --add safe.directory /workspace
WORKDIR /workspace

CMD ["zellij"]


# awk-language-server
# bash-language-server
# bibtex-tidy
# clangd              
# lldb-dap
# vscode-css-language-server
# docker-langserver
# dot-language-server
# gopls
# dlv (go debugger)
# terraform-ls
# scode-html-language-server
# typescript-language-server
# jq-lsp
# vscode-json-language-server
# lua-language-server
# marksman
# perlnavigator
# ruff
# regols
# ruby-lsp
# rust-analyzer
# vscode-css-language-server
# taplo
# yaml-language-server
# ansible-language-server