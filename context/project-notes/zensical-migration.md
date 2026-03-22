# Zensical Migration Plan

## Status: Ready to implement

## What is Zensical

Zensical is a static site generator by the Material for MkDocs team. It replaces MkDocs.

- **Install:** `pip install zensical` or `uv add --dev zensical`
- **Config:** `zensical.toml` (TOML, not YAML)
- **Commands:** `zensical serve`, `zensical build`, `zensical new`
- **License:** MIT

## Key Difference: Config Format

MkDocs uses `mkdocs.yml` (YAML). Zensical uses `zensical.toml` (TOML).

The earlier blog post claimed Zensical "can natively read mkdocs.yml" — but the actual docs show it uses its own `zensical.toml` format. Migration requires converting the config.

## Migration Steps

### 1. Install Zensical

```bash
# In our dev container (python image)
uv add --dev zensical
```

### 2. Convert mkdocs.yml → zensical.toml

Our current `mkdocs.yml` needs to be converted. Key mappings:

```toml
# zensical.toml
[project]
site_name = "dev-box v0.4.2"
site_url = "https://projectious-work.github.io/dev-box/"
site_description = "dev-box — manage AI-ready development container environments"

# Theme, nav, extensions — need to check Zensical equivalents
```

### 3. Test Compatibility

Features we use that need verification:
- `pymdownx.tabbed` (used in cheatsheet — `=== "Tab"` syntax)
- `pymdownx.superfences` (code blocks)
- `pymdownx.highlight` (syntax highlighting)
- `admonition` (`!!! note`, `!!! tip`, `!!! danger`)
- `pymdownx.details` (collapsible blocks)
- Material theme features (navigation tabs, dark/light toggle, etc.)
- `toc` with permalinks

### 4. Update Build Scripts

In `scripts/maintain.sh`:
- `mkdocs serve` → `zensical serve`
- `mkdocs build --strict --clean` → `zensical build`
- `mkdocs` dependency check → `zensical`

### 5. Update GitHub Pages Deployment

The `cmd_docs_deploy` function in maintain.sh builds the site and pushes to gh-pages. Update the build command.

## Risk Assessment

- **Low risk:** Zensical is by the same team, designed as a replacement
- **Medium risk:** Config format change requires manual conversion
- **Unknown:** Whether all pymdownx extensions work identically
- **Mitigation:** Test locally with `zensical serve` before switching

## Recommendation

Do a test migration in a branch. If any pymdownx extensions don't work, we may need to wait for Zensical to mature. MkDocs 1.x support runs until Nov 2026 — we have time.
