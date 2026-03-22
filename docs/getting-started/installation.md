# Installation

## Prerequisites

dev-box requires a container runtime on your host machine:

=== "Podman (recommended)"

    ```bash
    # macOS
    brew install podman
    podman machine init
    podman machine start

    # Fedora / RHEL
    sudo dnf install podman podman-compose

    # Ubuntu / Debian
    sudo apt install podman podman-compose
    ```

=== "Docker"

    ```bash
    # macOS
    brew install --cask docker
    # Then launch Docker Desktop

    # Linux — follow the official install guide
    # https://docs.docker.com/engine/install/
    ```

dev-box auto-detects which runtime is available. If both are installed, Podman takes priority.

## Install script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/scripts/install.sh | bash
```

Downloads the correct pre-built binary for your platform (Linux or macOS, x86_64 or ARM64) and installs it to `~/.local/bin/`.

Options:

```bash
# Install a specific version
curl -fsSL .../install.sh | VERSION=0.8.0 bash

# Install to a custom directory
curl -fsSL .../install.sh | INSTALL_DIR=/usr/local/bin sudo -E bash
```

## Manual download

Download the binary for your platform from the [releases page](https://github.com/projectious-work/dev-box/releases):

```bash
# Example for macOS ARM64
tar xzf dev-box-v0.8.0-aarch64-apple-darwin.tar.gz
mv dev-box-v0.8.0-aarch64-apple-darwin ~/.local/bin/dev-box
chmod +x ~/.local/bin/dev-box
```

Available binaries:

| Platform | File |
|----------|------|
| macOS ARM64 (Apple Silicon) | `dev-box-v0.8.0-aarch64-apple-darwin.tar.gz` |
| macOS x86_64 (Intel) | `dev-box-v0.8.0-x86_64-apple-darwin.tar.gz` |
| Linux ARM64 | `dev-box-v0.8.0-aarch64-unknown-linux-gnu.tar.gz` |
| Linux x86_64 | `dev-box-v0.8.0-x86_64-unknown-linux-gnu.tar.gz` |

## Build from source

Requires a [Rust toolchain](https://rustup.rs/):

```bash
git clone https://github.com/projectious-work/dev-box.git
cd dev-box
cargo install --path cli
```

Installs the binary to `~/.cargo/bin/`.

## Verify

```bash
dev-box --version
# dev-box 0.8.0
```

## Shell completions

```bash
# Add to your shell profile for persistent completions:

# Bash (~/.bashrc)
eval "$(dev-box completions bash)"

# Zsh (~/.zshrc)
eval "$(dev-box completions zsh)"

# Fish (~/.config/fish/config.fish)
dev-box completions fish | source
```

## Next steps

- [Create a new project](new-project.md)
- [Add dev-box to an existing project](existing-project.md)
