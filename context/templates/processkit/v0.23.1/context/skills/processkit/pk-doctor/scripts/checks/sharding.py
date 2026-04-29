"""sharding check.

Two rules:

- Logs under `context/logs/` must live in a `YYYY/MM/` subdir
  (e.g. context/logs/2026/04/LOG-....md). A LogEntry directly under
  context/logs/ is a WARN.
- Migrations under `context/migrations/` must live in their state-bucket
  subdir (pending / in-progress / applied). A Migration directly under
  context/migrations/ or in the wrong bucket (relative to its spec.state)
  is a WARN.

Phase 1: WARN only, no auto-fix. Phase 2 can add a scripted move under `--fix`.
"""

from __future__ import annotations

import re
from pathlib import Path

import yaml

from .common import CheckResult


YYYY_RE = re.compile(r"^\d{4}$")
MM_RE = re.compile(r"^(0[1-9]|1[0-2])$")
VALID_MIG_BUCKETS = {"pending", "in-progress", "applied"}
# aibox-CLI migration docs (e.g. 20260410_1523_0.17.6-to-0.17.9.md) live in
# context/migrations/ but are NOT processkit Migration entities — they are
# CLI upgrade notes. Exempt them from sharding + schema checks.
CLI_MIGRATION_RE = re.compile(r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$")


def _read_state(path: Path) -> str | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None
    if not isinstance(data, dict):
        return None
    spec = data.get("spec")
    if isinstance(spec, dict):
        state = spec.get("state")
        if isinstance(state, str):
            return state
    return None


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []
    ctx_root = repo_root / "context"

    # ------------------ logs: YYYY/MM --------------------
    logs_dir = ctx_root / "logs"
    if logs_dir.is_dir():
        for p in logs_dir.rglob("*.md"):
            if since_files is not None and p not in since_files:
                continue
            if p.name.startswith("INDEX"):
                continue
            rel_parts = p.relative_to(logs_dir).parts
            # Expect [YYYY, MM, <LOG-...>.md]
            if len(rel_parts) < 3 or not YYYY_RE.match(rel_parts[0]) or not MM_RE.match(rel_parts[1]):
                results.append(CheckResult(
                    severity="WARN",
                    category="sharding",
                    id="sharding.log-wrong-bucket",
                    message=(
                        f"{p.relative_to(repo_root)}: LogEntry not under "
                        f"context/logs/YYYY/MM/"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))

    # ------------------ migrations: state bucket ---------
    mig_dir = ctx_root / "migrations"
    if mig_dir.is_dir():
        for p in mig_dir.rglob("*.md"):
            if since_files is not None and p not in since_files:
                continue
            if p.name.startswith("INDEX"):
                continue
            if CLI_MIGRATION_RE.match(p.name):
                # aibox-CLI upgrade doc, not a processkit Migration entity.
                continue
            rel_parts = p.relative_to(mig_dir).parts
            if len(rel_parts) < 2 or rel_parts[0] not in VALID_MIG_BUCKETS:
                results.append(CheckResult(
                    severity="WARN",
                    category="sharding",
                    id="sharding.migration-no-bucket",
                    message=(
                        f"{p.relative_to(repo_root)}: Migration not under "
                        f"pending/ | in-progress/ | applied/"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))
                continue
            bucket = rel_parts[0]
            state = _read_state(p)
            # Rejected migrations park under applied/ per existing convention.
            if state in ("pending", "in-progress", "applied") and state != bucket:
                results.append(CheckResult(
                    severity="WARN",
                    category="sharding",
                    id="sharding.migration-state-bucket-mismatch",
                    message=(
                        f"{p.relative_to(repo_root)}: spec.state='{state}' "
                        f"but file is in '{bucket}/'"
                    ),
                    entity_ref=str(p.relative_to(repo_root)),
                ))

    results.append(CheckResult(
        severity="INFO",
        category="sharding",
        id="sharding.checked",
        message="sharding check complete (logs YYYY/MM + migrations state-bucket)",
    ))
    return results
