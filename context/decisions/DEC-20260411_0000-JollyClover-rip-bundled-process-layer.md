---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-JollyClover-rip-bundled-process-layer
  created: '2026-04-10T22:33:00+00:00'
spec:
  title: Rip bundled process layer — processkit becomes the sole content source
  state: accepted
  decision: 'v0.16.0 removed the entire bundled process layer from aibox: the `aibox
    skill` subcommand, `process_registry.rs`, `skill_cmd.rs`, the `templates/` tree
    (~85 SKILL.md files, context-doc scaffolds, processes, agents pointer), `context/AIBOX.md`,
    and all 141 `include_str!` calls (~2000 lines from `context.rs`). processkit is
    now the sole owner of every skill, primitive schema, state machine, canonical
    AGENTS.md template, and package YAML. aibox owns only the install/diff/migrate
    machinery, containers, addons, and the slim project skeleton.'
  context: aibox had grown a bundled process layer (85+ skills, processes, context
    docs) that was duplicating and conflicting with processkit's evolving content.
    Every processkit update required manual synchronization with aibox's bundled copy.
    The boundary between "what aibox owns" and "what processkit owns" was unclear,
    making both projects harder to maintain and extend.
  rationale: Clean separation of concerns. processkit owns all content; aibox owns
    all infrastructure. Any future skill, process, or primitive improvement goes into
    processkit, not aibox. The split makes aibox a stable infrastructure layer and
    processkit a rapidly-evolving content layer without either blocking the other.
    The large net diff (−25,000/+5,000 lines) permanently reduced aibox's maintenance
    surface.
  alternatives:
  - option: Keep bundled skills as a fallback when processkit is not pinned
    rejected_because: Creates two competing implementations of the same skills, maintenance
      burden doubles, and users are confused about which version applies
  - option: Gradual deprecation — mark skills as deprecated, remove over multiple
      releases
    rejected_because: Prolongs the maintenance burden and the unclear boundary; user
      prefers clean break over drawn-out migration
  consequences: processkit becomes a required dependency for all substantive process
    content. Projects that relied on aibox's bundled skills must migrate to processkit.
    The `aibox skill` subcommand no longer exists — use `aibox kit skill` (added later
    in BACK-123).
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:33:00+00:00'
---
