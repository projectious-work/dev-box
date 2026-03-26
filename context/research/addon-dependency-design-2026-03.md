# Addon Dependency Design — Research and Decisions

**Date:** 2026-03-26
**Task:** BACK-052
**Status:** Closed — findings documented, minimal fixes applied

---

## Current state

Addons can declare `requires: [name, ...]` in their YAML.  The field is
loaded into `LoadedAddon::requires` and consumed by `topological_sort()` in
`cli/src/addons.rs`.  The sort uses Kahn's algorithm and terminates with an
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

Maximum tree depth: **1**.  No chains exist (e.g., node and python have no
`requires` of their own).

---

## Question 1 — Circular dependency detection

**Finding:** Already implemented and correct.

Kahn's algorithm in `topological_sort()` is the canonical cycle detector for
DAGs: if any node remains with in-degree > 0 at the end, a cycle exists.  The
sorted-length check on line 91 (`if sorted.len() != addon_names.len()`) fires
with message `"Circular dependency detected among addons"` for:

- Two-node cycles (A → B → A) — covered by the pre-existing test
`topo_sort_circular_dependency_errors`
- Self-loops (A → A) — a self-reference raises in-degree on A before any
decrement can occur, so A never enters the zero-degree queue; caught by the
same check
- Three-or-more-node cycles (A → B → C → A)

**Action taken:** Added two new unit tests in `cli/src/addons.rs`:
`topo_sort_self_referential_dependency_errors` and
`topo_sort_three_node_cycle_errors`.  Both pass.

**Decision:** No code changes needed beyond the tests.  Circular dependency
detection is sound.

---

## Question 2 — Transitivity

**Finding:** `requires` is not transitive in the current implementation, and
that is correct for the present tree depth.

`topological_sort()` operates on the *explicitly listed* addon names from
`aibox.toml`.  It validates that every `requires` entry for a listed addon is
*also* listed.  It does not recursively resolve missing transitive
dependencies.

**What this means in practice today:** The tree is depth-1.  If a user adds
`docs-mkdocs`, the validation error tells them to add `python` explicitly.
There is no hidden transitive requirement they could miss.

**What it means if depth grows:** If we someday add a `python` addon that
requires `build-tools` (a hypothetical base package), and a user lists only
`docs-mkdocs` + `python`, they would receive an error telling them to add
`build-tools`.  This is correct behaviour — the error message is actionable.
There is no silent mis-ordering or panic.

**Should we auto-expand transitively?** Not yet.  Drawbacks of auto-expansion:
implicit side-effects are hard to inspect and debug; users can't see what was
silently pulled in; it couples the resolver to the YAML loading order.  The
current explicit-list model mirrors `apt` with `--no-install-recommends` and is
the right default at this scale.

**If the tree grows beyond depth 2–3**, consider adding an `aibox addon
resolve <name>` subcommand that prints the full transitive closure without
modifying `aibox.toml`, keeping expansion explicit.

**Decision:** Keep non-transitive validation.  Document the explicit-list
requirement in user-facing error messages (already done — message reads "Add
`[addons.<name>]` to your aibox.toml").  Revisit when depth > 2.

---

## Question 3 — Surfacing resolved order

**Finding:** `aibox addon info <name>` did not show `requires` at all before
this task.  `aibox init` output does not show install order.

**Action taken:** Added `Requires: <dep>, <dep>` line to `cmd_addon_info()` in
`cli/src/addon_cmd.rs`.  The line appears only when the addon has at least one
dependency, keeping clean output for the majority of addons.  Implementation
uses `addon_loader::get_addon()` directly since `AddonDef` (the registry
wrapper) does not carry `requires`.

**Full resolved order in `aibox init`:** Not added.  At depth-1 the order is
trivially obvious (dependency before dependent).  If depth increases and
`aibox addon resolve` is added, `aibox init` can call it to print the plan.

**Decision:** Show `requires` in `addon info`.  Defer full resolved-order
display in `init` until depth warrants it.

---

## Question 4 — Conflict detection

**Finding:** No mutual exclusions exist in the current addon corpus.

Potential candidates:
- `docs-docusaurus` vs `docs-mkdocs` — both are valid independently; a user
  can install both (unusual but not broken)
- Multiple cloud addons (`cloud-aws`, `cloud-azure`, `cloud-gcp`) — fully
  composable, no conflict
- Multiple doc-site frameworks — a project might generate two sites; no runtime
  conflict in the container

**Is a `conflicts` field warranted?** Not today.  The systemd model
(`Conflicts=`) exists because units are mutually exclusive at the *process*
level — you cannot run two services that bind the same port.  Addons install
packages and write files; the only real conflict would be two addons writing the
same file at the same path, which the Dockerfile build process would catch at
image build time.

**If we add runtime-config addons** (e.g., two addons that both write
`.zshrc` in incompatible ways), a `conflicts` field would be justified.
Until then, it is complexity without a use-case.

**Decision:** No `conflicts` field for now.  If a real conflict is discovered,
add the field to `AddonYaml`, validate during `topological_sort()` (or a
separate pre-flight check), and error with a clear message naming both addons.

---

## Reference: systemd unit-file fields

| systemd field | aibox equivalent | Status |
|---------------|-----------------|--------|
| `Requires=` | `requires:` in YAML | Implemented |
| `After=` / `Before=` | implicit from `requires` (deps come before dependents) | Sufficient |
| `Wants=` | auto-add optional dependencies | Deferred — tree is too shallow |
| `Conflicts=` | `conflicts:` field | Not needed yet |
| `PartOf=` | "remove me if my parent is removed" | Not needed (addons are independent) |

---

## Summary of actions taken

| Action | File(s) changed |
|--------|----------------|
| Added `topo_sort_self_referential_dependency_errors` test | `cli/src/addons.rs` |
| Added `topo_sort_three_node_cycle_errors` test | `cli/src/addons.rs` |
| Show `Requires:` line in `aibox addon info` | `cli/src/addon_cmd.rs` |
| This document | `context/research/addon-dependency-design-2026-03.md` |
| BACK-052 closed in backlog | `context/BACKLOG.md` |
