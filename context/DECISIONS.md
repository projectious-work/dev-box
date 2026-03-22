# Decisions Log

Inverse chronological. Each decision has a rationale and alternatives considered.

## DEC-014 — Skills Library: curated quality over marketplace quantity (2026-03-22)

**Decision:** Ship 83 curated skills with reference files rather than providing a marketplace integration or a smaller "starter" set. Skills are embedded in the binary via `include_str!` and scaffolded on `dev-box init`. No external download step.

**Rationale:** Marketplace research (SkillsMP: 97K skills, Skills.sh: 40K, ClawHub: 13.7K) revealed that 46.3% of publicly available skills are duplicates or near-duplicates (HuggingFace analysis). The ecosystem's #1 problem is quality, not quantity. A curated library with progressive disclosure (SKILL.md < 150 lines, reference files for depth) differentiates dev-box from "skill slop." Embedding in the binary ensures skills work offline and are version-locked to the CLI.

**Categories chosen (14):** Process, Development, Language, Infrastructure, Architecture, Design & Visual, Data & Analytics, AI & ML, API & Integration, Security, Observability, Database, Performance, Framework & SEO. Based on marketplace demand analysis: infrastructure and data skills are vastly underserved relative to frontend/framework skills.

**Alternatives:** Marketplace-first (ClawHub/Skills.sh integration — deferred to backlog), smaller starter set with `dev-box skill install` (adds complexity, network dependency), external file download during init (fragile, no offline support).

## DEC-013 — Granular vim mounts preserve image colorschemes (2026-03-22)

**Decision:** The docker-compose template mounts `.vim/vimrc` and `.vim/undo` individually instead of the entire `.vim/` directory. This preserves the image-baked `~/.vim/colors/` and `~/.vim/pack/` directories.

**Rationale:** The base image downloads 6 vim colorscheme files (gruvbox, catppuccin-mocha, catppuccin-latte, tokyonight, nord, dracula) into `/root/.vim/colors/` during Docker build. When the entire `.vim/` was mounted from the host (`.dev-box-home/.vim/`), the image's `colors/` directory was shadowed. Result: `E185: Cannot find color scheme 'gruvbox'` in derived projects. Mounting only the two files we actually persist (vimrc and undo/) leaves the image's baked-in directories visible.

**Alternatives:** Copy colorschemes into `.dev-box-home/.vim/colors/` during seed (duplicates files, maintenance burden), embed colorschemes via `include_str!` in the binary (bloat, version drift), post-create command to copy (fragile).

## DEC-012 — Reference file scaffolding via SkillDef type (2026-03-22)

**Decision:** Extend `scaffold_skills()` to deploy reference files alongside SKILL.md. Changed the skills data structure from `(&str, &str)` to a `SkillDef` type alias: `(&str, &str, &[(&str, &str)])` = `(name, content, [(ref_filename, ref_content)])`. Reference files go in `.claude/skills/<name>/references/`.

**Rationale:** 8 of the original 26 skills had reference files on disk (11 files total) but they were never deployed to derived projects — `scaffold_skills()` only wrote SKILL.md. With the expansion to 83 skills and 57 reference files, fixing this was a prerequisite. The `SkillDef` type alias satisfies clippy's type_complexity lint while keeping the flat `include_str!` embedding pattern.

**Alternatives:** Struct-based SkillDef (heavier, overkill for static data), dynamic file discovery at runtime (fragile, no compile-time guarantees), separate scaffolding function for references (unnecessary split).

## DEC-011 — Skills + Processes architecture: separate WHAT from HOW (2026-03-22)

**Decision:** Process declarations in context/ define WHAT processes exist ("there shall be backlog management"). Skills (SKILL.md standard) define HOW they're executed. Context stores the resulting artifacts (BACKLOG.md, DECISIONS.md, etc.). Skills come in flavors (e.g., backlog-context vs backlog-github) that users choose.

**Rationale:** Today's process presets bake both "what" and "how" into context template files. This makes them rigid — you can't swap from a context-file backlog to GitHub Issues without restructuring. By separating concerns: process declarations become thin ("there shall be X"), skills become the executable implementation, and artifacts remain in context/. This enables: swappable implementations, testable skills (via SKILL.md eval framework), thinner dev-box scaffolding, and a clear boundary between dev-box (infrastructure + curated skills) and derived projects (tailoring + execution).

**Relationship to SKILL.md standard:** The open standard at agentskills.io/specification provides the skill format. dev-box provides curated, vetted skills. External marketplaces (ClawHub) are user responsibility.

**Implications:** Process presets (minimal/managed/research/product) become skill compositions. dev-box.toml gains a [skills] section mapping processes to skill flavors. dev-box doctor checks consistency between declared processes and installed skills.

**Alternatives:** Keep current monolithic process templates (simpler but rigid). Full framework integration like SAFe/PMBOK (too heavy for dev-box scope — that's kaits territory).

## DEC-010 — context/shared/ directory for cross-environment files (2026-03-21)

**Decision:** The `context/` directory has a `shared/` subdirectory that is NOT copied during environment switches. Everything else in `context/` is per-environment. Default scaffolding places only `OWNER.md` in `shared/`. Users can move any file into `shared/` to share it across environments.

**Rationale:** Rather than hardcoding which files are shared (e.g., only OWNER.md) or adding include/exclude flags, a directory boundary lets the user decide. No templating, no merge logic. The AI agent sees all files under `context/` as usual — `shared/` is just a subdirectory. Inspired by Kustomize's base/overlay pattern, but simplified because our content is narrative markdown, not structured YAML that can be mechanically merged.

**Alternatives:** Hardcoded shared file list (inflexible), include/exclude flags in config (complexity), structural split into two parallel directories with merge (authoring nightmare for markdown content).

## DEC-009 — Environment switching uses plain directory copying (2026-03-21)

**Decision:** For Phase 2 environment management (`dev-box env`), per-environment state (dev-box.toml, CLAUDE.md, context/) is stored as plain file copies in `.dev-box-env/<name>/`. Switching copies files to/from the project root. `.devcontainer/` is regenerated (derived), `.dev-box-home/` is shared (user prefs).

**Rationale:** Evaluated four approaches:
- **uv-style** (config-only, environment disposable) — fails because `context/` contains irreplaceable user-written state, not just derived artifacts
- **Nix-style** (content-addressed store + symlinks) — symlinks break in container bind mounts, Nix assumes immutable store items but context files are mutable
- **OCI-style** (overlay filesystem layers) — OverlayFS is a Linux kernel feature, not portable to macOS host
- **Git branches** (nested repo for context) — a nested `.git` inside an already git-tracked project confuses VS Code, lazygit, and CLI git; hiding in `.tar.gz` defeats atomic switching

Plain copying wins because: context files are tiny (< 50 KB total), no tooling interference (gitignored directory), fully portable, debuggable (plain files), AI-agent friendly. The Nix/OCI models solve problems we don't have (deduplicating large binaries).

**Alternatives considered:** See rationale above.

## DEC-008 — backup and reset as standalone commands (2026-03-21)

**Decision:** `dev-box backup` and `dev-box reset` implemented as standalone commands (Phase 1). Backup saves timestamped snapshots to `.dev-box-backup/`. Reset backs up then deletes all dev-box files. `.gitignore` is backed up but not deleted. Container is stopped before reset.

**Rationale:** Safety nets for destructive experiments, major upgrades, and complete dev-box removal. Phase 2 environment management builds on different storage (`.dev-box-env/`) — backup remains useful as disaster recovery independent of environments.

**Alternatives:** Combining backup into env management — rejected because backup serves a different purpose (safety net vs. workflow switching).

## DEC-007 — Rename .root/ to .dev-box-home/ (2026-03-19)

**Decision:** Rename the host-side persisted config directory from `.root/` to `.dev-box-home/` for clarity. Backward compat: CLI falls back to `.root/` if it exists and `.dev-box-home/` doesn't.

**Rationale:** `.root/` is ambiguous — doesn't convey purpose. `.dev-box-home/` clearly indicates it's the container user's home directory content.

**Alternatives:** `.dev-box-config/`, `.config-mount/`, `.container-home/`

## DEC-006 — OWNER.md created locally, no symlink (2026-03-19)

**Decision:** OWNER.md is always created as a local file in `context/`, not symlinked from `~/.config/dev-box/`. The `owner` field was removed from `[context]` in dev-box.toml.

**Rationale:** The symlink pattern was confusing. Users can still share OWNER.md content across projects manually if they want.

## DEC-005 — No GitHub Actions (2026-03-16)

**Decision:** All builds and deploys are local. No GitHub Actions workflows.

**Rationale:** Cost avoidance. Local builds are fast enough for the project's scale.

## DEC-004 — Multi-stage TeX Live build (2026-03-16)

**Decision:** TeX Live installed via multi-stage Docker build from CTAN. Builder stage discarded (~2GB install).

**Rationale:** Keeps runtime image size manageable. CTAN install gives full control over package selection.

**Alternatives:** Debian texlive packages (too large, less control), TinyTeX (too minimal)

## DEC-003 — `uname -m` for arch detection (2026-03-16)

**Decision:** Use `uname -m` instead of `TARGETARCH` for architecture detection in Dockerfiles.

**Rationale:** Podman doesn't inject `TARGETARCH` build arg. `uname -m` works everywhere.

## DEC-002 — Alt-based zellij keybindings (2026-03-16)

**Decision:** All zellij keybindings use `Alt` modifier.

**Rationale:** Avoids conflicts with vim and TUI applications that use Ctrl.

## DEC-001 — Rust for CLI, not Python (2026-03-16)

**Decision:** dev-box CLI written in Rust, not Python.

**Rationale:** Single static binary, no runtime dependencies, fast cold start. Matches the "uv" inspiration.

**Alternatives:** Python (runtime dependency), Go (viable but less familiar to owner)
