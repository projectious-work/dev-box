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
curl -fsSL https://raw.githubusercontent.com/bnaard/dev-box/main/scripts/install.sh | bash
```

This downloads the correct pre-built binary for your platform (Linux or macOS, x86_64 or ARM64) and installs it to `~/.local/bin/`.

**Options:**

```bash
# Install a specific version
curl -fsSL .../install.sh | VERSION=0.1.0 bash

# Install to a custom directory
curl -fsSL .../install.sh | INSTALL_DIR=/usr/local/bin sudo -E bash
```

!!! note "Requires a GitHub release"
    The install script downloads from [GitHub releases](https://github.com/bnaard/dev-box/releases). If no release exists yet, use the "from source" method below.

### From source

If you have Rust installed, you can build from source:

```bash
# Clone the repository
git clone https://github.com/bnaard/dev-box.git
cd dev-box

# Build and install
cargo install --path cli
```

This places the `dev-box` binary in `~/.cargo/bin/`, which should already be in your `PATH`.

## Verifying Installation

```bash
dev-box --help
```

Expected output:

```
Manage AI-ready development container environments

Usage: dev-box [OPTIONS] <COMMAND>

Commands:
  init      Initialize a new project with dev-box.toml and generated files
  generate  Re-generate devcontainer files from dev-box.toml
  build     Build the container image
  start     Start container and attach via zellij
  stop      Stop the container
  attach    Attach to running container
  status    Show container status
  doctor    Validate context structure and produce migration artifacts
  update    Check for or apply version updates
  help      Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>      Path to dev-box.toml (default: ./dev-box.toml)
      --log-level <LOG_LEVEL>  Log level (trace, debug, info, warn, error) [default: info]
  -h, --help                 Print help
```

## Shell Completions (planned)

Shell completion support is planned for a future release. It will cover bash, zsh, and fish.

## Next Steps

- [Create a new project](new-project.md)
- [Add dev-box to an existing project](existing-project.md)
