"""processkit shared library for MCP servers.

This package is internal infrastructure for the MCP servers shipped under
`src/skills/<name>/mcp/server.py`. It is not published to PyPI and has no
external entry points.

The public modules are:

- `entity` — read/write entity files (apiVersion/kind/metadata/spec)
- `frontmatter` — split YAML frontmatter from Markdown body
- `ids` — generate IDs (word/uuid × with/without slug)
- `paths` — find project root, resolve per-kind directories
- `schema` — load schemas, validate spec against JSON Schema
- `state_machine` — load state machines, validate transitions
- `config` — read processkit.toml settings
- `index` — SQLite indexer (used by the index-management MCP server)

See `src/lib/README.md` for the import strategy from MCP server scripts.
"""

__version__ = "0.3.0"

API_VERSION = "processkit.projectious.work/v1"

# Prefix registry — primitive kind → ID prefix
KIND_PREFIXES = {
    "WorkItem": "BACK",
    "LogEntry": "LOG",
    "DecisionRecord": "DEC",
    "Actor": "ACTOR",
    "Role": "ROLE",
    "Binding": "BIND",
    "Scope": "SCOPE",
    "Category": "CAT",
    "Gate": "GATE",
    "Metric": "METRIC",
    "Schedule": "SCHED",
    "Constraint": "CONST",
    "Context": "CTX",
    "Discussion": "DISC",
    "Process": "PROC",
    "StateMachine": "SM",
    "Artifact": "ART",
    "Migration": "MIG",
    "Note": "NOTE",
    "TeamMember": "TEAMMEMBER",
}

# Default subdirectory under context/ for each primitive kind
DEFAULT_DIRS = {
    "WorkItem": "workitems",
    "LogEntry": "logs",
    "DecisionRecord": "decisions",
    "Actor": "actors",
    "Role": "roles",
    "Binding": "bindings",
    "Scope": "scopes",
    "Category": "categories",
    "Gate": "gates",
    "Metric": "metrics",
    "Schedule": "schedules",
    "Constraint": "constraints",
    "Discussion": "discussions",
    "Process": "processes",
    "StateMachine": "state-machines",
    "Artifact": "artifacts",
    # Migration's "default" is a logical root; the actual file lives under
    # context/migrations/{pending,in-progress,applied}/ — the substate
    # mirrors the entity's spec.state. The migration-management skill
    # places files in the correct subdirectory.
    "Migration": "migrations",
    "Note": "notes",
    "TeamMember": "team-members",
}
