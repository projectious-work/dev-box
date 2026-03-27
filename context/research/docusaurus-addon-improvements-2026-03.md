# Docusaurus Addon Improvements — Research Report

**Date:** 2026-03-26
**Task:** BACK-047
**Status:** Complete

---

## 1. Current State

The `docs-docusaurus` addon (`/workspace/addons/docs/docs-docusaurus.yaml`) is minimal:

```yaml
name: docs-docusaurus
version: "1.0.0"
requires:
  - node
skills:
  - documentation
tools:
  - name: docusaurus
    default_enabled: true
    default_version: "3"
    supported_versions: ["3"]
builder: null
runtime: |
  # Addon: docs-docusaurus
  RUN npm install -g create-docusaurus@latest
```

What it does today:
- Declares a dependency on the `node` addon via `requires: [node]`
- Installs `create-docusaurus` globally via `npm install -g`
- Tags `@latest` resolves at image build time to whatever version npm returns
- Provides no builder step, no scaffolding, no version pinning
- The user documentation (`docs-site/docs/addons/documentation.md`) already tells users to run `npx create-docusaurus@latest docs classic` to initialize a project

**Key observation:** The global install of `create-docusaurus` is a scaffolding tool only. It generates the initial project structure. Once a Docusaurus project exists, users run `npm install` / `npm start` / `npm build` from their project directory. The globally installed `create-docusaurus` is never used again after initial scaffolding.

---

## 2. npx vs Global Install

### The problem with global install

`create-docusaurus` is a project generator (like `create-react-app`). Installing it globally:

- Wastes image space for a one-time-use tool (~50 MB of node_modules)
- The installed version becomes stale as the image ages — users may scaffold with an outdated template
- Conflicts with the standard Docusaurus documentation, which recommends `npx`

### Docusaurus team's recommended approach

The official Docusaurus documentation (docusaurus.io/docs/installation) has consistently recommended:

```bash
npx create-docusaurus@latest my-website classic
```

This is the canonical installation method. The `npx` approach:
- Always fetches the latest version at execution time
- Requires no global install
- Is the pattern used by all major React meta-frameworks (Next.js, Remix, Astro, etc.)
- Works identically whether the user has `npm`, `pnpm`, or `yarn`

### Comparison with docs-starlight

The `docs-starlight` addon has the same pattern — `npm install -g create-starlight`. Starlight (Astro) similarly recommends `npm create astro@latest` / `npx create-starlight`. Both addons should be updated together.

### Recommendation

**Remove the global install entirely.** The addon's runtime Dockerfile line should be deleted or replaced with a no-op comment. The `node` addon already provides `npm` and `npx`, which is all that's needed.

The addon's value shifts from "pre-install a CLI tool" to "declare the documentation skill and Node dependency, and provide version metadata." This is a valid role — not every addon needs to install a binary.

If the team wants a fast offline experience (no network fetch on `npx`), a compromise is to pre-cache the package:

```dockerfile
RUN npx --yes create-docusaurus@latest --help >/dev/null 2>&1 || true
```

This populates the npm cache without polluting the global PATH, but the benefit is marginal and adds image size.

---

## 3. Scaffolding

### Should `aibox addon add docs-docusaurus` scaffold a docs-site?

Arguments for scaffolding:
- Reduces friction: user gets a working docs site immediately
- Ensures consistent project structure across aibox users
- Could set sensible defaults (site title from `aibox.toml` project name, theme matching aibox brand)

Arguments against scaffolding:
- Too opinionated — users have strong preferences about directory names (`docs/`, `docs-site/`, `website/`), templates (classic vs custom), and TypeScript vs JavaScript config
- Scaffolding is a one-time action that `npx create-docusaurus` already handles well
- Addons currently never modify the user's workspace — they only affect the container image. Introducing workspace mutation is a significant architectural change
- The scaffolded output would need maintenance as Docusaurus templates evolve
- Other docs addons (mkdocs, hugo, mdbook) do not scaffold; consistency matters

### Comparison with other addons

No existing addon scaffolds project files:
- `docs-mkdocs` installs mkdocs; user runs `mkdocs new .`
- `docs-hugo` installs hugo; user runs `hugo new site .`
- `docs-mdbook` installs mdbook; user runs `mdbook init .`

The pattern is consistent: addons install tools, users initialize projects.

### Recommendation

**Do not scaffold.** Keep the current separation where addons install tools and users initialize projects. Instead, improve the documentation to include a quick-start section with the exact `npx` command and recommended directory structure.

If scaffolding is desired in the future, it should be a separate feature (`aibox init --docs docusaurus`) that is explicit and interactive, not an implicit side-effect of adding an addon.

---

## 4. Node Addon Dependency

### Current behavior

The addon declares `requires: [node]` in its YAML. The dependency system (Kahn's algorithm in `topological_sort()`) ensures:
- If the user lists `docs-docusaurus` without `node`, they get an actionable error
- `node` is always ordered before `docs-docusaurus` in the Dockerfile

### Should it auto-add the node addon?

Per the addon dependency design research (`context/research/addon-dependency-design-2026-03.md`):

> **Should we auto-expand transitively?** Not yet. Drawbacks of auto-expansion: implicit side-effects are hard to inspect and debug; users can't see what was silently pulled in.

The current explicit-list model is correct. The user adds both `node` and `docs-docusaurus` to `aibox.toml`. The error message when `node` is missing is clear.

### Recommendation

**No change needed.** The `requires: [node]` declaration and explicit validation are working as designed. The dependency tree is depth-1 and the UX is clear.

---

## 5. Comparison with Other Docs Addons

| Addon | Requires | Install method | Version pinning | Notes |
|-------|----------|---------------|-----------------|-------|
| `docs-docusaurus` | node | `npm install -g create-docusaurus@latest` | None (`@latest`) | One-time scaffolder installed globally |
| `docs-starlight` | node | `npm install -g create-starlight` | None (latest) | Same anti-pattern as docusaurus |
| `docs-mkdocs` | python | `uv tool install 'mkdocs<2' --with mkdocs-material` | `<2` upper bound | Runtime tool, not just a scaffolder |
| `docs-zensical` | python | `uv tool install zensical` | None (latest) | Runtime tool |
| `docs-mdbook` | (none) | Binary download from GitHub releases | Pinned: v0.4.43 | Static binary, no runtime dependency |
| `docs-hugo` | (none) | Binary download from GitHub releases | Pinned: 0.141.0 | Static binary, no runtime dependency |

Key patterns:
- **Binary-based addons** (mdbook, hugo) pin exact versions — good practice for reproducibility
- **Python-based addons** (mkdocs) use upper-bound constraints — reasonable for pip/uv ecosystem
- **Node-based addons** (docusaurus, starlight) use `@latest` with no constraint — weakest reproducibility

The Node addons are outliers in two ways: they install scaffolders instead of runtime tools, and they have no version constraints.

---

## 6. Version Management

### Current state

The addon YAML declares:
```yaml
default_version: "3"
supported_versions: ["3"]
```

But this metadata is not used in the Dockerfile — the runtime line installs `@latest` regardless. The version fields exist for `aibox addon info` display purposes only.

### Docusaurus versioning

Docusaurus 3.x has been stable since late 2023. The framework follows semver and major versions introduce breaking changes (Docusaurus 2 to 3 required MDX upgrades, React 18, Node 18+).

### Options for version pinning

1. **Pin to a specific minor version** (e.g., `create-docusaurus@3.7`): Maximum reproducibility but requires manual bumps. Not practical for a scaffolder that runs once.

2. **Pin to a major version** (e.g., `create-docusaurus@3`): Allows patch/minor updates while preventing breaking changes. This is the right granularity for a major-version-aware tool.

3. **Keep `@latest`**: Current behavior. Acceptable if we remove the global install (since `npx` always fetches fresh anyway).

### Recommendation

**If keeping the global install:** Pin to major version `@3` instead of `@latest`, matching the `default_version` field. This prevents a future Docusaurus 4.0 release from breaking existing images.

**If removing the global install (preferred):** The version fields in the YAML become documentation-only metadata indicating which Docusaurus version the addon is tested against. When Docusaurus 4.0 ships, update `supported_versions` to `["3", "4"]` and bump the addon version.

---

## 7. Integration with aibox (Preview Companion)

### docs-dev script or compose service

The preview companion design (PROJ-004, `context/research/preview-companion-design-2026-03.md`) describes a companion container for live preview. For Docusaurus specifically:

- `npm run start` already provides a dev server with hot-reload on port 3000
- This is the standard Docusaurus development workflow
- A companion container is unnecessary for Docusaurus — the built-in dev server is superior

### What the addon could provide

1. **Port forwarding hint:** The addon could declare a default port (3000) so that `aibox start` can include it in SSH port-forwarding instructions.

2. **Zellij layout integration:** A `docs-dev` pane in the Zellij layout that auto-runs `npm start` in the docs directory. This ties into the addon's `skills: [documentation]` field.

3. **Build script:** A standardized `aibox docs build` command that finds the docs directory and runs the build. Low priority — `npm run build` is already simple.

### Recommendation

Port declaration and Zellij layout integration are the highest-value additions. These should be designed as part of a broader "addon-provided services" feature rather than Docusaurus-specific.

---

## 8. Prioritized Recommendations

### P0 — Remove global install (minimal change, clear improvement)

Replace the runtime line:

```yaml
runtime: |
  # Addon: docs-docusaurus
  # Docusaurus projects are initialized with: npx create-docusaurus@latest <dir> classic
  # No global install needed — the node addon provides npm/npx.
```

Or if an empty runtime is preferred, set `runtime: null` and add a comment in the YAML.

**Rationale:** The global `create-docusaurus` install wastes ~50 MB of image space for a tool that runs once. The official docs already recommend `npx`. The user documentation already tells users to use `npx`. The addon's value is the dependency declaration and skill tagging, not the global binary.

**Apply the same change to `docs-starlight`.**

### P1 — Update user documentation

Update `docs-site/docs/addons/documentation.md`:
- Change the "Install Method" column for Docusaurus from "npm" to "npx (on use)"
- Add a quick-start snippet showing the full `npx` command
- Note that the addon provides Node.js (via its `node` dependency) and Docusaurus version metadata

### P2 — Align version metadata with reality

Ensure `default_version` and `supported_versions` reflect actual tested compatibility. Consider adding a `notes` or `init_command` field to addon YAML so the CLI can display "To initialize: `npx create-docusaurus@latest my-docs classic`" via `aibox addon info docs-docusaurus`.

### P3 — Port declaration for future SSH forwarding

When the addon service/port feature is designed (related to PROJ-004), add:

```yaml
ports:
  - name: docs-dev
    default: 3000
    description: "Docusaurus dev server"
```

This is blocked on the broader port/service design.

### P4 — Zellij layout pane for docs development

When addon-provided Zellij layout fragments are supported, the documentation skill could contribute a "docs" pane. This is a broader feature that benefits all docs addons, not just Docusaurus.

---

## Summary

The most impactful change is the simplest: **remove the global `create-docusaurus` install and rely on `npx`**. This aligns with upstream recommendations, saves image space, ensures users always get the latest scaffolder version, and matches how the addon is already documented. The same fix applies to `docs-starlight`.

The addon's role shifts from "install a tool" to "declare a dependency chain and skill mapping" — which is a perfectly valid and useful function within the aibox addon system.
