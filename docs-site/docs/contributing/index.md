---
sidebar_position: 1
title: Contributing
---

# Contributing

Thank you for your interest in contributing to aibox!

## Getting Started

1. Fork and clone the repository:

    ```bash
    git clone https://github.com/projectious-work/aibox.git
    cd aibox
    ```

2. Start the development container:

    ```bash
    cd .devcontainer
    docker compose up -d
    docker compose exec aibox bash
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
- **`addons/`** — Addon definitions (language runtimes, tool bundles, AI agents)
- **`docs-site/`** — Docusaurus documentation source
- **`.devcontainer/`** — This project's own dev environment

> Process content (skills, packages, primitives, processes, the canonical
> `AGENTS.md`) lives in **[processkit](https://github.com/projectious-work/processkit)**,
> not in this repository. As of v0.16.0 aibox no longer ships a `templates/`
> directory or a `schemas/` directory — both have moved upstream to processkit.

See [CLAUDE.md](https://github.com/projectious-work/aibox/blob/main/CLAUDE.md) for detailed architecture notes.

## Development Workflow

### CLI Changes

1. Make your changes in `cli/src/`
2. Run `cargo test` to verify all tests pass
3. Run `cargo clippy -- -D warnings` for lint checks
4. Run `cargo fmt` to format code

### Image Changes

1. Edit the relevant Dockerfile in `images/`
2. Build locally to verify: `docker build -t aibox-test images/<flavor>/`
3. Test that derived images still build if you changed the base

### Documentation Changes

1. Edit or add pages in `docs-site/docs/`
2. Update `docs-site/sidebars.js` if adding new pages
3. Preview locally: `cd docs-site && npm run start`

## Pull Requests

- Keep PRs focused on a single change
- Include a clear description of what and why
- Ensure all tests pass and clippy is clean
- Update documentation if your change affects user-facing behavior

## Reporting Issues

File issues at [github.com/projectious-work/aibox/issues](https://github.com/projectious-work/aibox/issues).

When filing an issue, please:

- Use a descriptive title
- Label it: `bug` for broken behavior, `enhancement` for feature requests, `documentation` for doc gaps
- Include steps to reproduce (for bugs) or a use case description (for enhancements)
- Mention the aibox version (`aibox --version`) and container image flavor if relevant
