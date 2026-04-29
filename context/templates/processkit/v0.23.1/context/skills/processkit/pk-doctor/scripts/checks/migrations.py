"""migrations check.

Reads the pending migration entities directly from the filesystem (the
same filesystem walk that `migration-management.list_migrations(state=
'pending')` performs server-side — kept in sync with that convention).

Two layouts are supported (DeepMoss):

  1. processkit dogfood — pending migrations live under
     ``context/migrations/pending/``; applied ones move to
     ``context/migrations/applied/``.
  2. derived projects (e.g. aibox) — pending migrations live at the
     **top level** of ``context/migrations/`` and only applied ones
     move into ``context/migrations/applied/``.  ``aibox-CLI``
     upgrade-doc filenames (``YYYYMMDD_HHMM_<from>-to-<to>.md``) are
     filtered out via the same regex used by ``schema_filename`` so
     they are never miscounted as processkit Migration entities.

- INFO: count of pending migrations.
- WARN: each pending migration older than STALE_DAYS days.

Under `--fix=migrations` (or `--fix-all`), walks each pending migration
interactively and records the user's decision. Phase 1 does NOT call the
MCP `apply_migration` tool from this CLI — doctor.py is a subprocess
without an MCP client. Instead, the check surfaces a FIX-INTENT line
per confirmed migration that the user (or the calling agent) can
execute via the migration-management MCP server. This keeps the
non-negotiable "no hand-edits; all writes via MCP" invariant.

Phase 2 will teach doctor to invoke the MCP tool via a short-lived
stdio client.
"""

from __future__ import annotations

import re
import sys
from datetime import datetime, timezone
from pathlib import Path

import yaml

from .common import CheckResult


STALE_DAYS = 14

# aibox-CLI upgrade-doc filenames: ``YYYYMMDD_HHMM_<from>-to-<to>.md``.
# These live alongside processkit Migration entities under
# ``context/migrations/`` in a derived project, but are NOT processkit
# Migration entities and must not be counted as pending. Mirrors the
# constant of the same name in ``schema_filename.py``.
_CLI_MIGRATION_RE = re.compile(
    r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$"
)


def _load_frontmatter(path: Path) -> dict | None:
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
    return data if isinstance(data, dict) else None


def _candidate_pending_paths(repo_root: Path) -> list[Path]:
    """Return paths that *could* be pending Migration entities.

    Layout 1 (dogfood): ``context/migrations/pending/*.md``.
    Layout 2 (derived): ``context/migrations/*.md`` minus
        - the ``applied/`` subtree (terminal state),
        - ``INDEX.md``,
        - aibox-CLI upgrade-doc filenames.
    Layout 1 takes precedence when both exist, since dogfood projects
    intentionally use it.
    """
    mig_root = repo_root / "context" / "migrations"
    pending_dir = mig_root / "pending"
    if pending_dir.is_dir():
        return sorted(pending_dir.glob("*.md"))
    if not mig_root.is_dir():
        return []
    out: list[Path] = []
    for p in sorted(mig_root.glob("*.md")):
        if p.name == "INDEX.md":
            continue
        if _CLI_MIGRATION_RE.match(p.name):
            continue
        out.append(p)
    return out


def _list_pending(repo_root: Path) -> list[tuple[str, Path, datetime | None]]:
    out: list[tuple[str, Path, datetime | None]] = []
    for p in _candidate_pending_paths(repo_root):
        fm = _load_frontmatter(p)
        if not fm:
            continue
        # Filter to ONLY processkit Migration entities — derived projects
        # may keep non-Migration markdown next to migrations, and we don't
        # want to count those.
        if fm.get("kind") not in (None, "Migration"):
            continue
        md = fm.get("metadata", {}) if isinstance(fm.get("metadata"), dict) else {}
        mid = md.get("id") or p.stem
        created = md.get("created")
        ts: datetime | None = None
        if hasattr(created, "isoformat"):
            # YAML datetime
            ts = created if created.tzinfo else created.replace(tzinfo=timezone.utc)
        elif isinstance(created, str):
            try:
                ts = datetime.fromisoformat(created.replace("Z", "+00:00"))
            except ValueError:
                ts = None
        out.append((str(mid), p, ts))
    return out


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []

    pending = _list_pending(repo_root)
    results.append(CheckResult(
        severity="WARN" if pending else "INFO",
        category="migrations",
        id="migration.pending-count",
        message=(
            f"{len(pending)} pending migration(s) — review and apply via "
            f"migration-management MCP (or `/pk-doctor --fix=migrations`)"
            if pending
            else "0 pending migration(s)"
        ),
    ))

    now = datetime.now(timezone.utc)
    for mid, path, ts in pending:
        if ts is None:
            continue
        age_days = (now - ts).days
        if age_days >= STALE_DAYS:
            results.append(CheckResult(
                severity="WARN",
                category="migrations",
                id="migration.stale-pending",
                message=(
                    f"{mid}: pending for {age_days} day(s) "
                    f"(threshold: {STALE_DAYS})"
                ),
                entity_ref=mid,
                fixable=True,
                suggested_fix="review and apply or reject via migration-management MCP",
                fix_mcp_tool="mcp__processkit-migration-management__apply_migration",
            ))
    return results


def run_fix(ctx, results: list[CheckResult]) -> list[dict]:
    """Interactive walk over pending migrations.

    Emits a list of fix records. Phase 1 records intent only — the
    actual MCP call is performed by the invoking agent based on these
    records (see check's module docstring for rationale).
    """
    interactive = ctx.get("interactive", False)
    auto_yes = ctx.get("yes", False)
    repo_root: Path = ctx["repo_root"]
    fixes: list[dict] = []

    if not interactive and not auto_yes:
        fixes.append({
            "category": "migrations",
            "status": "skipped",
            "reason": "fix requires interactive prompt; re-run from terminal or pass --yes",
        })
        return fixes

    pending = _list_pending(repo_root)
    for mid, path, _ts in pending:
        if auto_yes:
            choice = "y"
        else:
            try:
                raw = input(f"  Apply {mid}? [y/N/s(kip)] ").strip().lower()
            except (EOFError, KeyboardInterrupt):
                print(file=sys.stderr)
                fixes.append({"category": "migrations", "entity": mid, "status": "aborted"})
                break
            choice = raw or "n"
        if choice == "y":
            fixes.append({
                "category": "migrations",
                "entity": mid,
                "status": "intent-recorded",
                "mcp_tool": "mcp__processkit-migration-management__apply_migration",
                "mcp_args": {"id": mid},
                "note": (
                    "doctor.py cannot call MCP tools from a plain subprocess "
                    "(Phase 1 limitation). Invoke apply_migration via the agent "
                    "or run `uv run migration-management/mcp/server.py` "
                    "in MCP mode."
                ),
            })
        elif choice == "s":
            fixes.append({"category": "migrations", "entity": mid, "status": "skipped"})
        else:
            fixes.append({"category": "migrations", "entity": mid, "status": "declined"})
    return fixes
