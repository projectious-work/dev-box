# src/

**The content processkit ships to consumers.** Treat this like source code: it
is the canonical material that gets installed into a project when pinned to a
processkit tag.

## Layout

`src/` is a **literal mirror of a fresh consumer project root**. The paths
here map 1:1 to what the consumer sees after install:

```
src/
├── AGENTS.md              → installed at <project>/AGENTS.md
├── context/
│   ├── schemas/           → installed at <project>/context/schemas/
│   ├── state-machines/    → installed at <project>/context/state-machines/
│   ├── processes/         → installed at <project>/context/processes/
│   └── skills/
│       ├── _lib/          → installed at <project>/context/skills/_lib/
│       │   └── processkit/   (the shared Python library used by MCP servers)
│       └── <skill>/       → installed at <project>/context/skills/<skill>/
├── .processkit/           → NOT installed; processkit catalog tooling only
│   ├── packages/          ← package tier definitions (minimal, software, …)
│   ├── FORMAT.md          ← format spec documentation
│   ├── primitives-INDEX.md ← primitives catalog index
│   └── INDEX.md           ← this catalog's own docs
└── INDEX.md               ← this file
```

## Rules

**Rule 1 — `src/` vs everything else.** Anything in `src/` is shipped.
Anything outside `src/` is about *this repo* (its dev environment, its
own context, its own docs). Do not mix the two.

- `.devcontainer/`, `aibox.toml`, `CLAUDE.md`, `AGENTS.md` → THIS repo's
  infrastructure (the AGENTS.md at the repo root is processkit's own;
  the shipped template lives at `src/AGENTS.md`)
- `context/` → this repo's own project-management artifacts
- `docs-site/` → processkit's user-facing documentation
- `scripts/` → developer-facing scripts (release, smoke test, diff)
- `src/` → **the content shipped to consumers**

**Rule 2 — provider-neutral and aibox-neutral.** Content under `src/`
must work for a consumer who never installs aibox and who uses a
non-Claude agent harness.

**Rule 3 — the mirror invariant.** Every path under `src/context/` and
`src/AGENTS.md` must be valid as a drop-in at the consumer's project root.
Content under `src/.processkit/` is tooling-only and never installed.

## PROVENANCE.toml

`src/PROVENANCE.toml` maps every shipped file to the git tag in which its
content last changed. It is the input to `scripts/processkit-diff.sh` and
to aibox's migration generator.

The release process regenerates it via `scripts/stamp-provenance.sh
<next-version>` before tagging. Do not edit by hand.

## Privacy convention for files installed into a project's `context/`

Per `src/.processkit/FORMAT.md`, entities with `privacy: user-private`
MUST live under a directory named `private/` somewhere within `context/`.
The default `.gitignore` shipped by aibox includes:

```
context/**/private/
```

## AGENTS.md placeholder contract

`src/AGENTS.md` is rendered into a consumer project's root by the installer
at install time. See `src/INDEX.md` → `## AGENTS.md placeholder contract`
(this section) and `src/.processkit/FORMAT.md` for the full contract.

Class A (installer-rendered) names currently used: `{{PROJECT_NAME}}`,
`{{PROCESSKIT_SOURCE}}`, `{{PROCESSKIT_VERSION}}`, `{{CONTEXT_PACKAGES}}`,
`{{AI_PROVIDERS}}`, `{{INSTALL_DATE}}`.
