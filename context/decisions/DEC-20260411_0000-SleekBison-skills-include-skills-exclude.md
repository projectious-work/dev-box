---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-SleekBison-skills-include-skills-exclude
  created: '2026-04-10T22:34:41+00:00'
spec:
  title: '[skills].include / [skills].exclude filtering activated in v0.16.5'
  state: accepted
  decision: 'v0.16.5 makes the `[skills]` config section functional. Effective skill
    set = union of all package `spec.includes.skills` (with `extends:` expansion,
    cycle-protected via visited-set) + `[skills].include` additions − `[skills].exclude`
    removals. Install path filters against this set; MCP registration filters by the
    same set. Skills with `metadata.processkit.core: true` install unconditionally
    regardless of include/exclude. First-install special case: filter is `None` and
    all skills install; filtering takes effect from the second sync onward (requires
    templates mirror to exist). `aibox doctor` warns on unknown skill names in include/exclude
    and on excluded core skills.'
  context: 'The `[skills]` section was added to the schema in v0.16.0 but was reserved/no-op.
    The MCP registration design surfaced the need: "MCP server registration should
    respect [skills] filtering". Activating both install and MCP registration at the
    same time keeps them consistent.'
  rationale: A skill filtered out at install time should also have no registered MCP
    server — inconsistency between install and registration would mean files missing
    on disk while their servers appear in the harness config. Diamond inheritance
    in `extends:` expansion collapses correctly with a visited-set (no semver-style
    resolution needed; packages are versioned with processkit, not independently).
  alternatives:
  - option: Apply filter only at MCP registration, not at install time
    rejected_because: Inconsistent — user expects filtered skill to be absent from
      disk entirely
  - option: Apply filter at first install too
    rejected_because: Requires building effective set from fresh cache before mirror
      exists; 50 extra lines of plumbing for a UX nicety; deferred to future release
  consequences: '`[skills].include`/`[skills].exclude` are now enforced. Projects
    with empty `[skills]` see no change. New `build_effective_skill_set` function
    in `content_init.rs`. `validate_skill_overrides` for doctor typo-check. `core:
    true` skills always install — currently only `skill-finder`.'
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:34:41+00:00'
---
