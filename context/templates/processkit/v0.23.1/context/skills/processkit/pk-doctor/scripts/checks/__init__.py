"""Check registry for pk-doctor.

Adding a Phase 2 check: drop a new module next to these, then append it
to REGISTRY. Order matters only for report ordering on stdout; the
aggregator doesn't otherwise sequence checks.
"""

from __future__ import annotations

from . import (
    schema_filename,
    sharding,
    migrations,
    migration_integrity,
    drift,
    team_consistency,
    release_integrity,
    commands_consistency,
    mcp_config_drift,
    server_header_drift,
    preauth_applied,
    skill_dag,
)

# (name, module). Name is the --category= token and the key in the
# per-category tally block of the doctor.report LogEntry.
REGISTRY = [
    ("schema_filename", schema_filename),
    ("sharding", sharding),
    ("migrations", migrations),
    ("migration_integrity", migration_integrity),
    ("drift", drift),
    ("team_consistency", team_consistency),
    ("release_integrity", release_integrity),
    ("commands_consistency", commands_consistency),
    ("mcp_config_drift", mcp_config_drift),
    ("server_header_drift", server_header_drift),
    ("preauth_applied", preauth_applied),
    ("skill_dag", skill_dag),
]


def get(name: str):
    for n, mod in REGISTRY:
        if n == name:
            return mod
    raise KeyError(f"unknown check: {name}")


def names() -> list[str]:
    return [n for n, _ in REGISTRY]
