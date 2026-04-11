---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-LuckyFinch-docusaurus-addon-improvements-research
  created: 2026-04-11
spec:
  title: "Docusaurus Addon Improvements — Research Report"
  type: reference
  state: permanent
  tags: [docusaurus, addon, npm, npx, scaffolding, docs, version-management]
  skill: research-with-confidence
  source_file: docusaurus-addon-improvements-2026-03.md
---

# Docusaurus Addon Improvements — Research Report

**Date:** 2026-03-26
**Task:** BACK-047

---

## 1. Key Finding

The `docs-docusaurus` addon installs `create-docusaurus` globally via `npm install -g`, but:
- `create-docusaurus` is a one-time scaffolding tool (~50 MB) never used after initial project creation
- Docusaurus official docs recommend `npx create-docusaurus@latest` instead
- All major React meta-frameworks use the same `npx` pattern (Next.js, Remix, Astro)

**Same problem applies to `docs-starlight`** — both should be updated together.

---

## 2. Recommendations (Prioritized)

### P0 — Remove global install (minimal change, clear improvement)

Change the addon runtime from:
```yaml
runtime: |
  # Addon: docs-docusaurus
  RUN npm install -g create-docusaurus@latest
```

To:
```yaml
runtime: |
  # Addon: docs-docusaurus
  # Docusaurus projects are initialized with: npx create-docusaurus@latest <dir> classic
  # No global install needed — the node addon provides npm/npx.
```

The addon's value shifts from "install a tool" to "declare the Node.js dependency chain and skill mapping" — a valid and useful function.

### P1 — Update user documentation

Update `docs-site/docs/addons/documentation.md`: change "npm" to "npx (on use)", add quick-start snippet with the full `npx` command.

### P2 — Align version metadata with reality

Consider adding an `init_command` field to addon YAML so `aibox addon info docs-docusaurus` can display "To initialize: `npx create-docusaurus@latest my-docs classic`".

### P3 — Port declaration (blocked on PROJ-004)

When addon service/port feature is designed, add:
```yaml
ports:
  - name: docs-dev
    default: 3000
    description: "Docusaurus dev server"
```

---

## 3. Scaffolding Decision: No

**Do not scaffold.** Consistent with all other docs addons (mkdocs, hugo, mdbook) — addons install tools, users initialize projects. Scaffolding would require workspace mutation, a significant architectural change.

If scaffolding is desired in the future: explicit interactive `aibox init --docs docusaurus` command, not an implicit addon side-effect.

---

## 4. Docs Addon Comparison

| Addon | Install method | Version pinning | Type |
|---|---|---|---|
| `docs-docusaurus` | `npm install -g create-docusaurus@latest` | None (`@latest`) | One-time scaffolder |
| `docs-starlight` | `npm install -g create-starlight` | None | One-time scaffolder |
| `docs-mkdocs` | `uv tool install 'mkdocs<2' --with mkdocs-material` | `<2` upper bound | Runtime tool |
| `docs-mdbook` | Binary download from GitHub | Pinned: v0.4.43 | Static binary |
| `docs-hugo` | Binary download from GitHub | Pinned: 0.141.0 | Static binary |

Node-based addons are outliers: they install scaffolders instead of runtime tools, and have no version constraints.

---

## 5. Node Dependency

The `requires: [node]` declaration is correct and working as designed. No change needed. The current explicit-list model (user adds both `node` and `docs-docusaurus`) is correct per the addon dependency design.
