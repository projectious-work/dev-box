---
name: state-machine-management
description: |
  Define and customize StateMachine entities — the state/transition graphs used by WorkItems, DecisionRecords, and other primitives with lifecycle. Use when a project needs to customize the default state machine for a primitive (add states, change transitions) or define a state machine for a custom entity type.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-state-machine-management
    version: "1.0.0"
    created: 2026-04-06T00:00:00Z
    category: processkit
    layer: 3
    uses:
      - skill: event-log
        purpose: Log events to keep the audit trail accurate after every write.
    provides:
      primitives: [StateMachine]
      templates: [statemachine]
---

# State Machine Management

## Intro

A StateMachine defines the valid states of a primitive and the allowed
transitions between them. processkit ships default state machines for
WorkItem and DecisionRecord; projects override these or define new ones
for custom entities.

## Overview

### Shape

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: StateMachine
metadata:
  id: SM-workitem
  name: workitem
  created: 2026-04-06T00:00:00Z
spec:
  description: "WorkItem lifecycle with a formal review state."
  initial: backlog
  terminal: [done, cancelled]
  states:
    backlog:
      description: "Not started."
      transitions:
        - to: in-progress
        - to: cancelled
    in-progress:
      description: "Being actively worked on."
      transitions:
        - to: review
        - to: blocked
        - to: cancelled
    blocked:
      description: "Cannot proceed until a blocker is resolved."
      transitions:
        - to: in-progress
        - to: cancelled
    review:
      description: "Awaiting review."
      transitions:
        - to: done
        - to: in-progress
    done:
      description: "Complete."
      transitions: []
    cancelled:
      description: "Abandoned."
      transitions: []
---
```

### Overriding the default

To customize the workitem state machine for a specific project, put an
override at `context/state-machines/workitem.yaml`. The index MCP server
and any other validator prefer the project file over the processkit default.

Overrides must:
- Keep the same `initial` state (or migrate existing data).
- Not remove states that existing entities are currently in.
- Add new transitions only from states that already exist.

### Workflow

1. Copy the default from `src/primitives/state-machines/<name>.yaml`.
2. Edit states, transitions, and guards.
3. Save to `context/state-machines/<name>.yaml`.
4. Run `aibox lint` (once it validates state machines) — or audit manually
   to ensure no existing entity is stranded in a state the new machine
   doesn't know about.
5. Log `state-machine.updated`.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Cycles in the state graph without explicit handling.** A cycle
  (e.g., `in-review → in-progress → in-review`) is fine if it's
  intentional, but every cycle should be marked and have an exit
  criterion. Unmarked cycles cause infinite loops in process
  execution.
- **Terminal states with outgoing transitions.** A "terminal"
  state by definition has no outgoing edges. If `cancelled` has a
  transition to `re-opened`, then `cancelled` isn't terminal —
  rename it. Mislabeled terminals confuse downstream queries
  filtering by lifecycle stage.
- **Missing initial state declaration.** Every state machine
  needs an explicit `initial` state. Without it, new entities
  have no lawful starting point and the validator can't reject
  malformed creations.
- **Hand-editing state machine YAML instead of using the MCP
  tools.** Direct edits skip validation. The state-machine
  primitive enforces "no orphan states" and "all transitions
  reference declared states" at write time; bypassing it lets
  invalid graphs land on disk.
- **State machine that doesn't match the entity's actual
  lifecycle.** If the schema says a workitem can be cancelled but
  the state machine has no `cancelled` state, the two are out of
  sync and queries break. Schema and state-machine must agree —
  treat them as one unit when changing either.
- **Forgetting to update consumers when transitions change.**
  When you remove a transition (e.g., dropping `in-progress →
  done` direct), every workitem currently in `in-progress` is
  stranded. Audit current entities before removing transitions
  and write a Migration if any are affected.
- **Adding states without removing dead ones.** State machines
  accumulate cruft over time. When you introduce a new state
  that supersedes an old one, remove or deprecate the old one.
  Otherwise the graph becomes a dictionary of every state anyone
  ever proposed.
- **Project-level state machines that diverge silently from
  defaults.** If a project overrides a default state machine
  (e.g., `context/state-machines/workitem.yaml`), record the
  override as a DecisionRecord. Future maintainers need to know
  why the project's workitem flow differs from processkit's
  default.

## Full reference

### Fields

| Field         | Type         | Notes                                                |
|---------------|--------------|------------------------------------------------------|
| `description` | string       | What this machine is for                             |
| `initial`     | string       | Starting state for new entities                     |
| `terminal`    | list[string] | States with no outgoing transitions                  |
| `states`      | map          | `state_name → {description, transitions}`           |

### Transition object

```yaml
transitions:
  - to: <state-name>
    guard: "optional human-readable precondition"
    on_enter: <event-type>        # LogEntry event to emit
    required_role: <role-name>    # who is allowed to make the transition
```

Guards are advisory unless enforced by an MCP server that knows about them.

### Validation

- No cycles from terminal states.
- Every non-terminal state has at least one outgoing transition.
- Every state referenced in `transitions` must exist in `states`.
- `initial` must exist in `states`.

### Machine history

Changes to a state machine are themselves LogEntries
(`event_type: state-machine.updated`) rather than git commits alone — this
lets queries reason about "what did the valid states look like on date X?".

### Multiple machines for one kind

You can ship multiple state machines for the same primitive kind by using
distinct `name`s. Entities opt into a specific machine via
`spec.state_machine: <name>`. Useful when the same kind has fundamentally
different lifecycles (e.g. `WorkItem` with `bug-lifecycle` vs `story-lifecycle`).
