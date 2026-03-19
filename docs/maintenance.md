# Maintenance

Internal procedures for building, releasing, and deploying dev-box.
All builds and deploys are run locally — there are no GitHub Actions.

## The `maintain.sh` Script

All maintenance tasks are driven by `scripts/maintain.sh`:

```bash
./scripts/maintain.sh <command> [options]
```

| Command | Purpose |
|---------|---------|
| `test` | Run `cargo fmt --check`, `clippy -D warnings`, and all tests |
| `build-images [--no-cache]` | Build all 8 published container images locally |
| `push-images <version>` | Push images to GHCR (requires login) |
| `docs-serve` | Preview documentation at `http://localhost:8000` |
| `docs-deploy [--dry-run]` | Build and push docs to `gh-pages` branch |
| `release <version>` | Full release prep: test, build CLI, build images, tag, generate release notes |

## Release Checklist

A full release covers three artifacts: CLI binaries, container images, and documentation.

!!! tip "Always start with `maintain.sh release`"
    The `release` command is the single entry point. It runs tests, builds
    artifacts, creates the git tag, and generates release notes and a step-by-step
    prompt (`dist/RELEASE-PROMPT.md`) for the remaining manual steps.
    Don't skip it and assemble releases by hand — you'll miss release notes.

### 1. Prepare the release

```bash
./scripts/maintain.sh release 0.3.5
```

This runs tests, builds the CLI binary for the current platform, builds all 8 container images (if a runtime is available), creates a git tag, and generates `dist/RELEASE-NOTES.md` and `dist/RELEASE-PROMPT.md`.

!!! warning "Don't forget Cargo.toml"
    Update the version in `cli/Cargo.toml` **before** running the release command.
    The `--version` flag and release artifacts derive from it.

### 2. Build CLI binaries for other platforms

The release command only builds for the current architecture. Cross-platform
binaries must be built on their respective machines (or via cross-compilation):

| Target | Where to build |
|--------|---------------|
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-apple-darwin` | macOS Intel (or cross-compile on Apple Silicon) |
| `aarch64-unknown-linux-gnu` | Linux ARM64 (e.g., this dev container) |
| `x86_64-unknown-linux-gnu` | Linux x86_64 |

For macOS builds there's a helper script:

```bash
./scripts/build-macos.sh 0.3.5
```

Attach additional binaries to the release after creation:

```bash
gh release upload v0.3.5 dist/dev-box-v0.3.5-x86_64-apple-darwin.tar.gz
```

### 3. Push the tag and create the GitHub release

```bash
git push origin v0.3.5

gh release create v0.3.5 \
  --title "dev-box v0.3.5" \
  --notes-file dist/RELEASE-NOTES.md \
  dist/dev-box-v0.3.5-*.tar.gz
```

!!! warning "Always use `--notes-file`, never `--generate-notes`"
    The `release` command generates `dist/RELEASE-NOTES.md` with commit history,
    image tags, and binary listings. Always use `--notes-file dist/RELEASE-NOTES.md`
    when creating the GitHub release. The `--generate-notes` flag only produces
    a bare diff link with no useful content.

### 4. Push container images to GHCR

First, authenticate with the GitHub Container Registry:

```bash
echo $GITHUB_TOKEN | podman login ghcr.io -u <username> --password-stdin
```

The token needs the `write:packages` scope. Create one at
[github.com/settings/tokens](https://github.com/settings/tokens).

Then push all 8 images:

```bash
./scripts/maintain.sh push-images 0.3.5
```

This pushes both versioned tags (`python-v0.3.5`) and `latest` tags (`python-latest`)
for each flavor.

!!! note "Build order matters"
    Derived images (`python-latex`, `rust-latex`, etc.) depend on their base
    flavors. `build-images` handles the correct order automatically.
    If you're building manually, always build and push `base` first.

### 5. Deploy documentation

```bash
./scripts/maintain.sh docs-deploy
```

This builds the MkDocs site and force-pushes to the `gh-pages` branch.
GitHub Pages serves the site at
[projectious-work.github.io/dev-box](https://projectious-work.github.io/dev-box/).

### 6. Verify

After a release, verify:

- [ ] `curl -fsSL .../install.sh | bash` installs the new version
- [ ] `dev-box --version` shows the correct version
- [ ] `podman pull ghcr.io/projectious-work/dev-box:base-v0.3.5` succeeds
- [ ] Documentation site reflects changes

## Container Images

Eight images are published to `ghcr.io/projectious-work/dev-box`:

| Image | Tag pattern | Depends on |
|-------|------------|------------|
| base | `base-vX.Y.Z` | debian:trixie-slim |
| python | `python-vX.Y.Z` | base |
| rust | `rust-vX.Y.Z` | base |
| latex | `latex-vX.Y.Z` | base |
| typst | `typst-vX.Y.Z` | base |
| python-latex | `python-latex-vX.Y.Z` | python |
| python-typst | `python-typst-vX.Y.Z` | python |
| rust-latex | `rust-latex-vX.Y.Z` | rust |

Build all locally:

```bash
./scripts/maintain.sh build-images
```

Build a single image manually:

```bash
podman build -t ghcr.io/projectious-work/dev-box:python-v0.3.5 images/python/
```

## Documentation

Documentation uses MkDocs with the Material theme. Source lives in `docs/`.

```bash
# Preview locally
./scripts/maintain.sh docs-serve

# Deploy to GitHub Pages
./scripts/maintain.sh docs-deploy

# Dry run (build only, don't push)
./scripts/maintain.sh docs-deploy --dry-run
```

## Running Tests

```bash
# Full test suite (fmt + clippy + tests)
./scripts/maintain.sh test

# Or individually
cd cli
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

The test suite includes 68 unit tests and 13 integration tests.
Integration tests run the `dev-box` binary as a subprocess.
