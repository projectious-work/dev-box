# processkit-task-router MCP server

## Tools

### `route_task(task_description: str) -> dict`

Route a natural-language task description to the matching processkit
skill, project-specific process override, and MCP tool.

**Parameters**

| Name | Type | Description |
|---|---|---|
| `task_description` | string | What the agent or user wants to do |

**Returns — on match**

```json
{
  "skill": "workitem-management",
  "skill_description_excerpt": "Create, transition, and query WorkItems...",
  "process_override": "context/processes/PROC-release.md",
  "server": "processkit-workitem-management",
  "tool": "create_workitem",
  "tool_qualified": "processkit-workitem-management__create_workitem",
  "domain_group": "workitem",
  "confidence": 0.87,
  "routing_basis": "keyword_match",
  "candidate_tools": [
    {
      "tool_qualified": "processkit-workitem-management__create_workitem",
      "score": 0.87,
      "rationale": "task vocabulary matches tool name 'create_workitem'"
    },
    {
      "tool_qualified": "processkit-workitem-management__transition_workitem",
      "score": 0.0,
      "rationale": "in domain group; not vocabulary-matched"
    },
    {
      "tool_qualified": "processkit-workitem-management__query_workitems",
      "score": 0.0,
      "rationale": "in domain group; not vocabulary-matched"
    }
  ]
}
```

`process_override` is omitted when no project-specific process file
exists for the matched skill.

**Returns — low confidence**

Same shape, but `routing_basis == "needs_llm_confirm"` and
`confidence < 0.5`. Surface `candidate_tools` to the user.

**Returns — no match**

```json
{"error": "no matching skill or tool found", "hint": "..."}
```

## Confidence scoring

`confidence` is the geometric mean of the Phase 1 group score and
the Phase 2 tool score. Range 0.0–1.0.

| Range | Interpretation |
|---|---|
| ≥ 0.7 | High — proceed directly |
| 0.5–0.69 | Medium — proceed with awareness |
| < 0.5 | Low — confirm with user or LLM before acting |

## Tool naming convention

`tool_qualified` uses the `{server}__{tool}` double-underscore prefix
(MetaMCP de-facto standard for collision-free tool identification
across aggregated MCP servers).

## Routing basis values

| Value | Meaning |
|---|---|
| `keyword_match` | Matched via domain group + Phase 2 tool scoring |
| `skill_finder_trigger_table` | Fallback: matched via skill-finder |
| `needs_llm_confirm` | Confidence < 0.5; human or LLM confirmation advised |
