# Landscape Artifact Resolution — Reference

## Three-level precedence

When any `team-create`, `team-review`, or `team-rebalance` command
resolves the landscape snapshot, the following precedence applies:

```
1. Explicit --landscape-artifact <ART-id>   (highest priority)
   │
   └─ Skip all discovery; load the named artifact directly.
      Emit a warning if the artifact lacks the landscape-summary tag.

2. Project-tagged artifact (landscape-summary-project)
   │
   └─ Query artifact index for records carrying BOTH tags:
        landscape-summary  AND  landscape-summary-project
      whose spec.project_id matches the current project context.
      Select the most-recently-created match (by metadata.created desc).
      If multiple matches: pick the newest; log the others as ignored.

3. Kit default (landscape-summary)
   │
   └─ Query artifact index for the most-recently-created artifact
      tagged landscape-summary (without landscape-summary-project).
      If none found, abort with the standard error message.
```

The alias `--landscape` (existing) is accepted as a synonym for
`--landscape-artifact`. Both flags trigger path 1.

## Project-tagged artifact convention

A derived project's landscape artifact must include these fields:

```yaml
tags: [landscape, landscape-summary, landscape-summary-project]
project_id: <project-slug>     # e.g. "edge-lab-q2-2026"
```

`project_id` is matched against the current project context (read
from `context/project.yaml` field `slug`, or from the CLI environment
variable `PROCESSKIT_PROJECT_ID` if the file is absent).

## Staleness warning

Regardless of which path resolves the artifact: if the artifact's
`metadata.created` date is more than 90 days before the run date,
emit a warning and continue. Record the artifact ID and date in the
chartering DecisionRecord `inputs_snapshot`.

## Audit trail

The resolved artifact ID and the resolution path taken (`"explicit"`,
`"project-tag"`, or `"kit-default"`) are recorded in the chartering
DecisionRecord under `inputs_snapshot.landscape_artifact_source`.
