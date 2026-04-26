<!-- pk-compliance v2 -->

## processkit Compliance Contract

Call `route_task(task_description)` before any `create_*`,
`transition_*`, `link_*`, `record_*`, or `open_*` tool call.

If there is even a 1% chance a processkit skill applies to the current
task, consult `skill-finder` (or call `find_skill`) before acting.

When you decide to create a WorkItem, DecisionRecord, Note, or Artifact,
call the tool in the same turn — deferred entity creation is lost.

Write entities through MCP tools, not by hand-editing files under
`context/` — hand edits bypass schema validation, state-machine
enforcement, and the event-log auto-entry.

Read entities through `index-management` (`query_entities`,
`get_entity`, `search_entities`) — do not use `ls`, `grep`, or raw
filesystem walks under `context/`.

Log an event after any state change that an MCP write did not already
produce automatically.

After a cross-cutting recommendation is accepted, call `record_decision`
in the same turn.

When the last five user messages contain explicit decision language
(approved / decided / ship it / let's go / ok / yes / confirmed),
either call `record_decision` in the same turn or call
`skip_decision_record(reason=...)` to acknowledge the skip.

Do not edit any file under `context/templates/` — it is a read-only
upstream mirror used as a diff baseline.

Do not hand-edit the generated harness MCP config — edit the per-skill
`mcp-config.json` and let the installer re-merge.
