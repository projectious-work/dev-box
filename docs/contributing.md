# Contributing

Thank you for your interest in contributing to dev-box!

## Getting Started

1. Fork and clone the repository:

    ```bash
    git clone https://github.com/projectious-work/dev-box.git
    cd dev-box
    ```

2. Start the development container:

    ```bash
    cd .devcontainer
    docker compose up -d
    docker compose exec dev-box bash
    ```

    Or open in VS Code with the Dev Containers extension.

3. Build the CLI:

    ```bash
    cd cli
    cargo build
    ```

4. Run the tests:

    ```bash
    cargo test
    cargo clippy -- -D warnings
    cargo fmt -- --check
    ```

## Project Structure

- **`cli/`** — Rust CLI source code
- **`images/`** — Published container image Dockerfiles
- **`templates/`** — Work process flavor templates
- **`schemas/`** — Context schema documents
- **`docs/`** — MkDocs documentation source
- **`.devcontainer/`** — This project's own dev environment

See [CLAUDE.md](https://github.com/projectious-work/dev-box/blob/main/CLAUDE.md) for detailed architecture notes.

## Development Workflow

### CLI Changes

1. Make your changes in `cli/src/`
2. Run `cargo test` to verify all tests pass
3. Run `cargo clippy -- -D warnings` for lint checks
4. Run `cargo fmt` to format code

### Image Changes

1. Edit the relevant Dockerfile in `images/`
2. Build locally to verify: `docker build -t dev-box-test images/<flavor>/`
3. Test that derived images still build if you changed the base

### Documentation Changes

1. Edit or add pages in `docs/`
2. Update `mkdocs.yml` nav if adding new pages
3. Preview locally: `mkdocs serve`

## Pull Requests

- Keep PRs focused on a single change
- Include a clear description of what and why
- Ensure all tests pass and clippy is clean
- Update documentation if your change affects user-facing behavior

## Reporting Issues

File issues at [github.com/projectious-work/dev-box/issues](https://github.com/projectious-work/dev-box/issues).

When filing an issue, please:

- Use a descriptive title
- Label it: `bug` for broken behavior, `enhancement` for feature requests, `documentation` for doc gaps
- Include steps to reproduce (for bugs) or a use case description (for enhancements)
- Mention the dev-box version (`dev-box --version`) and container image flavor if relevant
