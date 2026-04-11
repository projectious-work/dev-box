---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-GentleOwl-addon-dependency-design-research
  created: 2026-04-11
spec:
  title: "Addon Dependency Design — Research and Decisions"
  type: reference
  state: permanent
  tags: [addons, dependencies, topology, implementation]
  skill: research-with-confidence
  source_file: addon-dependency-design-2026-03.md
---

# Addon Dependency Design — Research and Decisions

**Date:** 2026-03-26
**Task:** BACK-052
**Status:** Closed — findings documented, minimal fixes applied

---

## Current state

Addons can declare `requires: [name, ...]` in their YAML. The field is
loaded into `LoadedAddon::requires` and consumed by `topological_sort()` in
`cli/src/addons.rs`. The sort uses Kahn's algorithm and terminates with an
error when the sorted list is shorter than the input, which catches all cycles
including self-loops.

### Dependency map (full addon corpus, 2026-03-26)

| Addon | Requires |
|-------|----------|
| docs-docusaurus | node |
| docs-starlight | node |
| docs-mkdocs | python |
| docs-zensical | python |
| all others (17) | — |

Maximum tree depth: **1**. No chains exist.

---

## Question 1 — Circular dependency detection

**Finding:** Already implemented and correct.

Kahn's algorithm in `topological_sort()` is the canonical cycle detector for
DAGs: if any node remains with in-degree > 0 at the end, a cycle exists. The
sorted-length check fires for:
- Two-node cycles (A → B → A) — covered by pre-existing test
- Self-loops (A → A)
- Three-or-more-node cycles (A → B → C → A)

**Action taken:** Added two new unit tests: `topo_sort_self_referential_dependency_errors` and
`topo_sort_three_node_cycle_errors`. Both pass.

**Decision:** No code changes needed beyond the tests.

---

## Question 2 — Transitivity

**Finding:** `requires` is not transitive in the current implementation, and that is correct for the present tree depth.

`topological_sort()` operates on the explicitly listed addon names from `aibox.toml`.
It validates that every `requires` entry for a listed addon is also listed.
It does not recursively resolve missing transitive dependencies.

**Should we auto-expand transitively?** Not yet. Drawbacks:
- Implicit side-effects are hard to inspect and debug
- Users can't see what was silently pulled in
- Couples the resolver to the YAML loading order

The current explicit-list model mirrors `apt` with `--no-install-recommends`.

**If the tree grows beyond depth 2–3**: consider adding `aibox addon resolve <name>`
subcommand that prints the full transitive closure without modifying `aibox.toml`.

**Decision:** Keep non-transitive validation. Revisit when depth > 2.

---

## Question 3 — Surfacing resolved order

**Action taken:** Added `Requires: <dep>, <dep>` line to `cmd_addon_info()` in
`cli/src/addon_cmd.rs`. The line appears only when the addon has at least one dependency.

**Decision:** Show `requires` in `addon info`. Defer full resolved-order display.

---

## Question 4 — Conflict detection

**Finding:** No mutual exclusions exist in the current addon corpus.

A `conflicts` field would be justified if two addons write the same file at the same path,
but the Dockerfile build process would catch this at image build time.

**Decision:** No `conflicts` field for now. Add if a real conflict is discovered.

---

## Reference: systemd unit-file fields

| systemd field | aibox equivalent | Status |
|---------------|-----------------|--------|
| `Requires=` | `requires:` in YAML | Implemented |
| `After=` / `Before=` | implicit from `requires` | Sufficient |
| `Wants=` | auto-add optional dependencies | Deferred — tree is too shallow |
| `Conflicts=` | `conflicts:` field | Not needed yet |
| `PartOf=` | "remove me if my parent is removed" | Not needed |

---

## Summary of actions taken

| Action | File(s) changed |
|--------|----------------|
| Added `topo_sort_self_referential_dependency_errors` test | `cli/src/addons.rs` |
| Added `topo_sort_three_node_cycle_errors` test | `cli/src/addons.rs` |
| Show `Requires:` line in `aibox addon info` | `cli/src/addon_cmd.rs` |
