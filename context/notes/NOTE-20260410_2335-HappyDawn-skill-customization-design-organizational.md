---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-HappyDawn-skill-customization-design-organizational
  created: 2026-04-11
spec:
  title: "Skill Customization Design — Organizational Learning via Skills"
  type: reference
  state: permanent
  tags: [skills, customization, conventions, overlays, organizational-learning]
  skill: research-with-confidence
  source_file: skill-customization-design-2026-03.md
---

# Skill Customization Design — Organizational Learning via Skills

**Date:** 2026-03-26
**Task:** BACK-051
**Status:** Draft

---

## 1. Problem Statement

aibox ships curated skills that encode generic best practices. Real teams accumulate
specific conventions that are equally important:

- **Library choices:** "prefer clap over structopt for CLI args"
- **Error handling patterns:** "always wrap errors in our `AppError` enum"
- **Naming conventions:** "REST endpoints use plural nouns, database tables use singular"
- **Architectural rules:** "no direct database access from HTTP handlers"
- **CI/CD patterns:** "all PRs must have a `Test plan` section"

Today, teams repeat these rules every session or hope the agent infers them from code.
Neither scales. Conventions should accumulate over time, attach to the relevant skill,
and persist across sessions, developers, and machines.

**Core tension:** Skills must remain curated and updatable (aibox owns the base content),
while organizations and projects must be able to extend them without forking.

---

## 2. Design Options

### Option A: Skill Overlays (per-project files) [Recommended Phase 1]

A `CUSTOM.md` file alongside the curated `SKILL.md`:

```
.claude/skills/rust-conventions/
  SKILL.md          # curated (owned by aibox, overwritten on sync)
  CUSTOM.md         # project-specific (owned by team, never overwritten)
  references/       # curated reference files
```

Claude Code already reads all `.md` files in a skill directory — `CUSTOM.md` is
automatically included in the skill's context window. No concatenation needed.

**CLI:**
```bash
aibox skill customize rust-conventions   # opens $EDITOR on CUSTOM.md
aibox skill customize rust-conventions --show  # prints current overlay
```

### Option D: `aibox skill learn` Command (interactive append) [Recommended Phase 1]

```bash
aibox skill learn rust-conventions "prefer clap over structopt"
aibox skill learn rust-conventions "use tracing, not log"
aibox skill learned rust-conventions     # list current rules
aibox skill unlearn rust-conventions "prefer clap over structopt"
```

Appends to `.claude/skills/<name>/CUSTOM.md` under a `## Learned Rules` heading.

### Option B: Layered Skill System (org + project) [Phase 2]

Multiple skill sources with merge priority: `curated < org < project`.

```toml
[skills]
org_source = "git@github.com:myorg/aibox-skills.git"
```

Org skill repo provides `CUSTOM.md` files per skill. `aibox sync` clones/pulls
org repo to cache, merges curated + org + project overlays.

### Option C: CLAUDE.md Integration (not recommended as sole solution)

Already works today but doesn't scale — CLAUDE.md becomes a dumping ground.
Good complement, not a replacement.

---

## 3. Recommendation

**Phase 1 (implement now): Options A + D combined.**

Two ways to customize skills:
1. `aibox skill customize <name>` — opens `CUSTOM.md` in `$EDITOR` for structured, multi-paragraph conventions.
2. `aibox skill learn <name> "<rule>"` — quick one-liner for capturing conventions mid-session.

Both write to the same `CUSTOM.md` file, git-committed and never overwritten by `aibox sync`.

**Phase 2 (if demand exists): Option B.**
Org-level skill distribution via git. Deferred until multiple organizations request it.

---

## 4. Implementation Sketch (Phase 1)

### File Convention

```
.claude/skills/rust-conventions/
  SKILL.md        # curated, owned by aibox
  CUSTOM.md       # project-owned, never overwritten
  references/     # curated references
```

### CLI Subcommands

```
aibox skill customize <name>           # open CUSTOM.md in $EDITOR
aibox skill customize <name> --show    # print current CUSTOM.md to stdout
aibox skill learn <name> "<rule>"      # append a rule to CUSTOM.md
aibox skill learned <name>             # list rules in CUSTOM.md
aibox skill unlearn <name> "<rule>"    # remove a rule (exact match)
```

### Changes to `aibox sync`

Existing logic already does not touch unknown files in skill directories.
Only `SKILL.md` and reference files are written. `CUSTOM.md` is preserved automatically.

Report customized skills in sync output:
```
[ok] Deployed 3 missing skills
[info] 2 skills have project customizations (rust-conventions, git-workflow)
```

### Rust Implementation

New file: `cli/src/skill_customize.rs`

```rust
fn custom_md_path(skill_name: &str) -> PathBuf {
    Path::new(".claude/skills").join(skill_name).join("CUSTOM.md")
}

pub fn cmd_skill_customize(name: &str, show: bool) -> Result<()>
pub fn cmd_skill_learn(name: &str, rule: &str) -> Result<()>
pub fn cmd_skill_learned(name: &str) -> Result<()>
pub fn cmd_skill_unlearn(name: &str, rule: &str) -> Result<()>
pub fn has_custom_overlay(skill_name: &str) -> bool
```

### Testing Plan

- Unit tests: ensure/create/append/remove/read operations correct
- Integration tests: full cycle learn/verify/unlearn; sync preserves CUSTOM.md
- Edge cases: skill not in active set, duplicate rules, non-existent rules

---

## 5. Guideline for Users

**Put in skill overlay if:** domain-specific convention (language, framework, infrastructure).
**Put in CLAUDE.md if:** project-wide workflow rule (cross-cutting).

**Memory vs skill overlays:**
- Memory: "user prefers concise responses" (personal, per-user)
- Skill overlay: "this project uses clap for CLI args" (team, per-project)
