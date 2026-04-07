# aibox v0.16.0 — processkit becomes the content source

**Breaking change.** v0.16.0 enforces a hard boundary between aibox and
[processkit](https://github.com/projectious-work/processkit): aibox owns
containers + the processkit installer, processkit owns every skill,
primitive, process, package YAML, and the canonical `AGENTS.md`.

See **DEC-027** in `context/DECISIONS.md` for the full rationale.

## What's new

### The aibox ⇄ processkit boundary

- **aibox owns:** devcontainers, addons, the `[processkit]` config
  section, the install/diff/migrate machinery, and a slim project
  skeleton at init time (`.aibox-version`, `.gitignore`, an empty
  `context/`, a thin `CLAUDE.md` pointer to `AGENTS.md`,
  `.devcontainer/Dockerfile.local`, `docker-compose.override.yml`).
- **processkit owns:** every skill (`SKILL.md` + `references/` +
  `mcp/` + `templates/`), every primitive schema, every state machine,
  the canonical `scaffolding/AGENTS.md` template, the processes
  (`bug-fix.md`, `code-review.md`, …), and the package YAMLs
  (`packages/{minimal,managed,software,research,product}.yaml` with
  `extends:` composition).
- **Shared territory:** `context/`. aibox creates it, processkit fills
  it, the user edits in place. An immutable upstream reference is
  git-tracked under `context/templates/processkit/<version>/` for the
  three-way diff (DEC-026).

### `[processkit]` is the load-bearing config section

```toml
[processkit]
source   = "https://github.com/projectious-work/processkit.git"
version  = "v0.5.1"   # set to a real tag to install content
src_path = "src"
# branch = "main"     # tarball-first, branch as fallback
# release_asset_url_template = "..."  # for non-GitHub hosts
```

The default is `version = "unset"`, which means aibox sets up the
project skeleton but installs no processkit content. Pin a real tag
(e.g. `"v0.5.1"`) to install ~108 skills, primitives, processes,
state machines, and the canonical `AGENTS.md`.

### Provider thin pointers

`[ai] providers = ["claude", ...]` now drives a thin `CLAUDE.md` at
the project root that points at `AGENTS.md`. **Nothing is written
under `.claude/skills/`** — provider directories are no longer aibox
territory. Other providers (aider, gemini, mistral) use config files
scaffolded elsewhere; only Claude has a top-level markdown convention.

### Container deps queued from v0.15.0

| Tool   | Old    | New    |
|--------|--------|--------|
| zellij | 0.44.0 | 0.44.1 |
| fzf    | 0.70.0 | 0.71.0 |
| delta  | 0.19.1 | 0.19.2 |
| ouch   | 0.5.1  | 0.6.1  |

## What was removed

- `aibox skill` CLI subcommand (entirely)
- `cli/src/process_registry.rs` (705 lines, 13 hardcoded packages)
- `cli/src/skill_cmd.rs` (~250 lines)
- The bundled `templates/` tree (~85 `templates/skills/<name>/SKILL.md`
  files plus `templates/{minimal,managed,product,research}/`
  context-doc scaffolds plus `templates/processes/` plus
  `templates/agents/`)
- `context/AIBOX.md` (the auto-generated "universal baseline" — gone
  in both this repo and any project that aibox manages)
- `context::reconcile_skills`, `context::generate_aibox_md`,
  `context::check_agent_entry_points`, `expected_context_files`,
  `template_content_for_key`, `setup_owner_md`, `ALL_SKILL_DEFS` and
  141 `include_str!` calls in `cli/src/context.rs` (~2000 of 2368
  lines gone — the file is now ~530 lines)
- `effective_skill_includes` (config) and `skills_for_addons`
  (addon-loader)
- `context/AIBOX.md` and `.claude/skills/` from the sync perimeter

Net diff: **+1,124 / −26,733** lines across 239 files.

## Migration impact (breaking)

Any project on aibox v0.15.x:

- **`context/AIBOX.md` is gone.** Delete the file. Anything that
  pointed at it should now point at `AGENTS.md`.
- **`aibox skill` is gone.** Edit `context/skills/<name>/SKILL.md`
  files in place per Strawman D.
- **`[skills] include = [...]` parses but no-ops.** Today's installer
  installs every skill processkit ships, regardless of which package
  is selected. The field is reserved for a future release that may
  re-introduce filtering.
- **Set `[processkit].version = "v0.5.1"`** (or another real tag) for
  skills/processes/AGENTS.md to land. The default is `"unset"`, which
  produces an empty install.
- **`AGENTS.md` will appear at the project root** on the next
  `aibox init`. If you already have a hand-written `AGENTS.md`, the
  install is `write_if_missing` and your file is preserved.
- The **default `[context].packages`** is now `["managed"]` (the old
  `["core"]` preset name is no longer valid).
- **Old preset names** (`core`, `tracking`, `standups`, `handover`,
  `code`, `architecture`, `design`, `data`, `operations`,
  `documentation`, `full-product`, `research-project`) all map to one
  of the five processkit packages: `minimal`, `managed`, `software`,
  `research`, `product`.

## Quality gates

- 426/426 tests pass (`cargo test`)
- Zero clippy warnings (`cargo clippy --all-targets -- -D warnings`)
- `cargo audit` clean

## Linked decisions

- **DEC-027** — aibox v0.16.0: rip the bundled process layer
- **DEC-026** — Cache-tracked processkit reference under
  `context/templates/processkit/<version>/`
- **DEC-025** — Generic content-source release-asset fetcher (v0.15.0)
