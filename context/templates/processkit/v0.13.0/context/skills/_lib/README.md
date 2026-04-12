# src/lib/processkit/

Shared Python utilities used by every processkit MCP server. This is
**internal infrastructure**, not a primitive. It is shipped under `src/`
because consumers (and aibox Phase 4.3 install logic) need it alongside
the MCP servers — but it has no SKILL.md and is not in any package tier.

## Modules

| Module          | Purpose                                                          |
|-----------------|------------------------------------------------------------------|
| `entity.py`     | Parse and write entity files (apiVersion/kind/metadata/spec)     |
| `frontmatter.py`| Split YAML frontmatter from Markdown body                        |
| `ids.py`        | Generate IDs (word/uuid × with/without slug)                     |
| `paths.py`      | Find project root, resolve directories per primitive kind        |
| `schema.py`     | Load schemas from `src/primitives/schemas/`, validate spec       |
| `state_machine.py` | Load + validate state machine transitions                     |
| `index.py`      | SQLite indexer used by the index-management MCP server           |
| `config.py`     | Read processkit.toml settings (ids, sharding, directories, …)    |

## Import strategy

Each MCP server (`src/skills/<name>/mcp/server.py`) injects this directory
into `sys.path` at startup before importing:

```python
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "lib"))
from processkit import entity, ids, paths, state_machine
```

`parents[3]` walks: `mcp/` → `<skill-name>/` → `skills/` → `src/` and
then we append `lib`. This is robust as long as the directory layout is
preserved.

## Consumer-install layout (post aibox-handover-v2)

aibox installs the lib alongside skills in the consumer project, at a
provider-neutral path under `context/skills/`:

```
context/skills/_lib/processkit/...
context/skills/event-log/mcp/server.py
context/skills/workitem-management/mcp/server.py
...
```

The relative path lookup in each server's `_find_lib()` walks up from
`context/skills/<name>/mcp/` and finds `context/skills/_lib/processkit/`
on the third hop. Servers may also accept `PROCESSKIT_LIB_PATH` env var
as an override for non-standard installations.

## No package install

The lib is intentionally **not** published as a PyPI package. PEP 723
inline deps in each server.py declare `mcp[cli]`, `pyyaml`, `jsonschema`,
and that's it. The shared lib comes from the same processkit checkout
that the server lives in. This avoids version drift between processkit
content and the lib that processes it.
