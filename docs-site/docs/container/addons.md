---
sidebar_position: 2
title: "Addons"
---

{/* Note: This file was renamed from flavors.md. The content describes image flavors but may need updating to better reflect the "addons" terminology. */}

# Image Flavors

aibox provides ten container image flavors. All build on top of the [base image](base-image.md).

## Overview

| Image | Base | Adds | Primary Use Case |
|-------|------|------|-----------------|
| `base` | Debian Trixie Slim | Zellij, Vim, Git, lazygit, Claude CLI, audio | General development, shell scripting |
| `python` | base | Python 3.13, uv, MkDocs Material | Python projects, documentation |
| `latex` | base | TeX Live (basic scheme + packages) | Academic papers, technical documents |
| `typst` | base | Typst | Modern typesetting, technical documents |
| `rust` | base | Rust toolchain (stable), clippy, rustfmt | Rust projects |
| `node` | base | Node.js LTS | JavaScript/TypeScript projects |
| `go` | base | Go toolchain | Go projects |
| `python-latex` | python | Python + TeX Live | Data science with LaTeX reports |
| `python-typst` | python | Python + Typst | Data science with Typst reports |
| `rust-latex` | rust | Rust + TeX Live | Rust projects with LaTeX documentation |

## base

The foundation image. Everything else builds on this.

**Who it is for:** Shell scripting, DevOps work, projects that install their own language runtimes, or projects where the base tools are sufficient.

**Select it with:**

```toml
[aibox]
image = "base"
```

See [Base Image](base-image.md) for full details on what is included.

## python

Adds Python 3.13 with modern tooling on top of base.

**What it includes:**

- Python 3.13 (Debian package)
- pip and venv
- [uv](https://github.com/astral-sh/uv) -- fast Python package manager and project tool
- uvx -- uv's tool runner
- MkDocs Material (installed via `uv tool install`)

**Who it is for:** Python application development, data science, documentation projects using MkDocs.

**Select it with:**

```toml
[aibox]
image = "python"
```

**Usage notes:**

- Use `uv` instead of `pip` for package management -- it is significantly faster
- MkDocs is available globally via `mkdocs serve`, `mkdocs build`, etc.
- Create virtual environments with `uv venv` or `python3 -m venv`

## latex

Adds TeX Live for document compilation.

**What it includes:**

- TeX Live installed from CTAN (basic scheme)
- Common LaTeX packages added on top of the basic scheme
- `latexmk`, `biber`, and other compilation tools

**Who it is for:** Academic papers, technical documentation, any project that compiles LaTeX to PDF.

**Select it with:**

```toml
[aibox]
image = "latex"
```

**Usage notes:**

- Compile with `latexmk -pdf document.tex`
- Add missing packages with `tlmgr install <package-name>` (persists until container rebuild)
- For persistent package additions, use `extra_packages` in `aibox.toml` or extend the image

### TeX Live Multi-Stage Build

The LaTeX images use a multi-stage build strategy to keep the final image smaller:

1. **Stage 1 (texlive-builder):** Installs TeX Live from CTAN in a minimal Debian container with only `perl`, `wget`, and `fontconfig`
2. **Stage 2 (runtime):** Copies the entire `/usr/local/texlive` tree from the builder into the base image

This avoids installing TeX Live's build dependencies (perl, wget) in the final image. The `option_doc 0` and `option_src 0` profile settings skip documentation and source files, reducing the TeX Live footprint significantly.

## rust

Adds the Rust toolchain for systems programming.

**What it includes:**

- Rust stable toolchain via rustup
- clippy (linter)
- rustfmt (formatter)
- build-essential, pkg-config, libssl-dev (for compiling crates with native dependencies)

**Who it is for:** Rust application and library development.

**Select it with:**

```toml
[aibox]
image = "rust"
```

**Usage notes:**

- The toolchain is installed in `/root/.cargo/`
- Use `rustup` to add components or switch toolchains
- `cargo` is available on PATH immediately

## node

Adds Node.js LTS for JavaScript and TypeScript development.

**What it includes:**

- Node.js LTS (via NodeSource)
- npm

**Who it is for:** JavaScript and TypeScript application development, frontend projects, full-stack web development.

**Select it with:**

```toml
[aibox]
image = "node"
```

**Usage notes:**

- Use `npm` or `npx` for package management
- For Yarn or pnpm, install via `extra_packages` or `post_create_command`

## go

Adds the Go toolchain for Go development.

**What it includes:**

- Go toolchain (official binary distribution)

**Who it is for:** Go application and service development.

**Select it with:**

```toml
[aibox]
image = "go"
```

**Usage notes:**

- `go`, `gofmt`, and standard Go tools are available on PATH
- Module caching is handled by Go's built-in module system

## typst

Adds [Typst](https://typst.app/), a modern typesetting system that is simpler and faster than LaTeX.

**What it includes:**

- Typst binary (static musl build from GitHub releases)

**Who it is for:** Technical documents, academic papers, presentations — anyone who wants LaTeX-quality output with a modern, readable markup language.

**Select it with:**

```toml
[aibox]
image = "typst"
```

**Usage notes:**

- Compile with `typst compile document.typ`
- Watch mode: `typst watch document.typ` (recompiles on save)
- Packages are downloaded automatically on first use from [Typst Universe](https://typst.app/universe/) — no manual install step
- Package cache lives at `~/.cache/typst/packages/` — persist it via `.aibox-home/.cache/typst` in compose mounts
- Import packages in your `.typ` files: `#import "@preview/package-name:1.0.0"`

## python-typst

Combines Python and Typst in a single image.

**What it includes:** Everything from both `python` and `typst`.

**Who it is for:** Data science projects that generate Typst reports, Python analysis with Typst writeups.

**Select it with:**

```toml
[aibox]
image = "python-typst"
```

## python-latex

Combines Python and TeX Live in a single image.

**What it includes:** Everything from both `python` and `latex`.

**Who it is for:** Data science projects that generate LaTeX reports, Jupyter-to-PDF workflows, academic projects with Python analysis and LaTeX writeups.

**Select it with:**

```toml
[aibox]
image = "python-latex"
```

## rust-latex

Combines Rust and TeX Live in a single image.

**What it includes:** Everything from both `rust` and `latex`.

**Who it is for:** Rust projects that include LaTeX documentation, technical books or papers with Rust code examples.

**Select it with:**

```toml
[aibox]
image = "rust-latex"
```

## Adding Project-Specific Packages

Any image can be extended with additional apt packages via `aibox.toml`:

```toml
[container]
extra_packages = ["universal-ctags", "graphviz", "postgresql-client"]
```

These packages are installed during `aibox sync` via the generated Dockerfile. They persist across container restarts but are reinstalled on image rebuild.

:::tip When to use a different flavor vs extra_packages

Use image flavors for language runtimes (Python, Rust, TeX Live) -- they handle complex multi-stage builds and toolchain setup. Use `extra_packages` for simple apt packages that do not require special installation steps.

:::

## Image Size Considerations

| Image | Approximate Size |
|-------|-----------------|
| `base` | ~300 MB |
| `python` | ~450 MB |
| `latex` | ~600 MB |
| `typst` | ~310 MB |
| `rust` | ~800 MB |
| `node` | ~400 MB |
| `go` | ~600 MB |
| `python-latex` | ~750 MB |
| `python-typst` | ~460 MB |
| `rust-latex` | ~1.1 GB |

:::note Sizes are approximate

Actual sizes depend on architecture (amd64 vs arm64) and the specific versions of packages installed. TeX Live size varies based on which additional packages are included.

:::
