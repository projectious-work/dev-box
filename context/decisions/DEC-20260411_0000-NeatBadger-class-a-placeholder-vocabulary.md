---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-NeatBadger-class-a-placeholder-vocabulary
  created: '2026-04-10T22:35:15+00:00'
spec:
  title: Class A placeholder vocabulary for AGENTS.md templating
  state: accepted
  decision: 'aibox uses a three-class placeholder model for installed templated files
    (currently only `scaffolding/AGENTS.md`). Class A (11 keys, aibox-rendered from
    `aibox.toml` + install context): `{{PROJECT_NAME}}`, `{{CONTAINER_HOSTNAME}}`,
    `{{CONTAINER_USER}}`, `{{AIBOX_VERSION}}`, `{{AIBOX_BASE}}`, `{{PROCESSKIT_SOURCE}}`,
    `{{PROCESSKIT_VERSION}}`, `{{INSTALL_DATE}}`, `{{ADDONS}}`, `{{AI_PROVIDERS}}`,
    `{{CONTEXT_PACKAGES}}`. Class B (owner-supplied, pass-through). Class C (discoverable
    by agent, pass-through). Templated files use `write_if_missing` semantics — first
    install writes rendered content; subsequent syncs leave the file alone to protect
    owner edits.'
  context: processkit's AGENTS.md template contains project-specific placeholders
    that only the owner knows (description, purpose, code style, PR conventions) alongside
    facts aibox can render at install time. aibox cannot ask 30 questions at `init`
    time; processkit's onboarding skill fills Class B/C after install.
  rationale: 'Three-class split lets each filling mechanism do what it''s best at:
    aibox renders known facts, the onboarding skill interviews the owner for project-specific
    content, and agents discover build/test/lint commands from the codebase. The HashMap-based
    renderer scales to N placeholders without growing the function signature. `write_if_missing`
    semantics match `CLAUDE.md` scaffolding — first install sets up a starting point,
    owner edits freely thereafter. The lowercase `{{project_name}}` alias from v0.16.3
    was deliberately removed (clean break, no version compat).'
  alternatives:
  - option: Render templated files unconditionally (clobber on every sync)
    rejected_because: Violates edit-in-place principle; surprises owners who edited
      AGENTS.md
  - option: Render into templates mirror and run three-way diff against rendered content
    rejected_because: Correct long-term answer but requires non-trivial diff machinery
      changes; implemented as DEC-034 in v0.16.5 as follow-up
  consequences: Adding a new Class A key is a breaking change requiring an aibox release.
    processkit's AGENTS.md template uses 6 Class A + 5 Class B + 3 Class C placeholders.
    `InstallAction::InstallTemplated` variant added to `content_install.rs`. `build_substitution_map`
    in `context.rs` is the single source of truth for Class A keys.
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:35:15+00:00'
---
