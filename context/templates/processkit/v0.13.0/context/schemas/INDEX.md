# primitives/schemas/

One YAML file per primitive kind. Each schema is itself a processkit entity
(`kind: Schema`) and validates the `spec` field of entities of the target kind.

Schema files use **JSON Schema (draft 2020-12) expressed in YAML** for the
`spec_schema` field. This is the format consumed by the `index-management`
MCP server (Phase 3).

## Shape

```yaml
apiVersion: processkit.projectious.work/v1
kind: Schema
metadata:
  id: SCHEMA-<kind>
  target_kind: <Kind>            # which primitive kind this schema validates
  version: "1.0.0"
spec:
  description: "One-line summary of the primitive."
  id_prefix: <PREFIX>             # the ID prefix entities of this kind use
  state_machine: <name>           # optional — references state-machines/<name>.yaml
  spec_schema:                    # JSON Schema for the entity's spec field
    type: object
    required: [...]
    properties:
      ...
```

## Shipped schemas

| File | Kind | Prefix | State machine? |
|---|---|---|---|
| `workitem.yaml` | WorkItem | BACK | yes |
| `logentry.yaml` | LogEntry | LOG | no (immutable, append-only) |
| `decisionrecord.yaml` | DecisionRecord | DEC | yes |
| `migration.yaml` | Migration | MIG | yes |
| `actor.yaml` | Actor | ACTOR | no |
| `role.yaml` | Role | ROLE | no |
| `scope.yaml` | Scope | SCOPE | yes |
| `gate.yaml` | Gate | GATE | no |
| `discussion.yaml` | Discussion | DISC | yes |
| `binding.yaml` | Binding | BIND | no |
| `category.yaml` | Category | CAT | no |
| `metric.yaml` | Metric | METRIC | no |
| `schedule.yaml` | Schedule | SCHED | no |
| `constraint.yaml` | Constraint | CONST | no |
| `context.yaml` | Context | CTX | no |
| `process.yaml` | Process | PROC | no |
| `statemachine.yaml` | StateMachine | SM | no |
| `artifact.yaml` | Artifact | ART | no |

`CrossReference` is intentionally NOT a file primitive — it lives as
frontmatter cross-references on other entities (see `binding-management`
SKILL.md for the rule). All other primitives have a schema as of v0.5.0.
