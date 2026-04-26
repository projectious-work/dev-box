# skill-finder MCP server

Navigation aid for the processkit skill catalog. Maps natural-language
task descriptions to the right SKILL.md via trigger-phrase matching.
Layer 0 — no entity writes, read-only.

## Tools

| Tool | Purpose |
|---|---|
| `find_skill(task_description)` | Return the best-matching skill name, its SKILL.md path, and a confidence score. Also surfaces close runners-up via `also_consider`. |
| `list_skills(category?)` | List all skills in the catalog, optionally filtered by category. Returns name, one-line description, category, and `has_mcp` flag. |

## Notes

- `find_skill` parses the trigger-phrase table in
  `context/skills/processkit/skill-finder/SKILL.md` and scores each
  entry against the task description using substring + token-overlap
  matching. The score is deterministic — no LLM calls.
- A `match_confidence` of `1.0` means an exact substring match against
  at least one trigger phrase. Scores below `0.5` are worth double-
  checking with `list_skills`.
- `list_skills` walks `context/skills/` on disk, so it reflects the
  installed catalog even if skill-finder's trigger table is stale.
- Both tools are `readOnlyHint: true` — they never write files or
  mutate entities.

## Running

```bash
uv run context/skills/processkit/skill-finder/mcp/server.py
```
