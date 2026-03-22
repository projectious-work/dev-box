# Installation

## Prerequisites

dev-box requires a container runtime on your host machine.

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

!!! note "Podman vs Docker"
    dev-box auto-detects which runtime is available. If both are installed, Podman takes priority. The generated `docker-compose.yml` files work with both.

## Installing the CLI

### Install script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/scripts/install.sh | bash
```

This downloads the correct pre-built binary for your platform (Linux or macOS, x86_64 or ARM64) and installs it to `~/.local/bin/`.

!!! tip "Manual download"
    If you prefer to download manually from the [releases page](https://github.com/projectious-work/dev-box/releases), each tarball (e.g., `dev-box-v0.7.0-aarch64-apple-darwin.tar.gz`) contains a single binary named `dev-box-v0.7.0-aarch64-apple-darwin`. After extracting, rename it to `dev-box` and move it to a directory in your `PATH`:

    ```bash
    tar xzf dev-box-v0.7.0-aarch64-apple-darwin.tar.gz
    mv dev-box-v0.7.0-aarch64-apple-darwin ~/.local/bin/dev-box
    chmod +x ~/.local/bin/dev-box
    ```

**Options:**

```bash
# Install a specific version
curl -fsSL .../install.sh | VERSION=0.1.0 bash

# Install to a custom directory
curl -fsSL .../install.sh | INSTALL_DIR=/usr/local/bin sudo -E bash
```

!!! note "Requires a GitHub release"
    The install script downloads from [GitHub releases](https://github.com/projectious-work/dev-box/releases). If no release exists yet, use the "from source" method below.

### From source

If you have Rust installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/projectious-work/dev-box.git
cd dev-box

# Build and install
cargo install --path cli
```

This places the `dev-box` binary in `~/.cargo/bin/`, which should already be in your `PATH`.

## Verifying Installation

```bash
dev-box --version
```

Expected output:

```
dev-box 0.7.0
```

To see all available commands:

```bash
dev-box --help
```

## Shell Completions

dev-box can generate shell completion scripts for bash, zsh, fish, powershell, and elvish.

```bash
dev-box completions bash
dev-box completions zsh
dev-box completions fish
```

### Setup

Add to your shell profile for persistent completions:

**Bash** (`~/.bashrc`):

```bash
eval "$(dev-box completions bash)"
```

**Zsh** (`~/.zshrc`):

```bash
eval "$(dev-box completions zsh)"
```

**Fish** (`~/.config/fish/config.fish`):

```bash
dev-box completions fish | source
```

## Next Steps

- [Create a new project](new-project.md)
- [Add dev-box to an existing project](existing-project.md)
