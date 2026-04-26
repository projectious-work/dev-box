"""migration_integrity check — flag malformed pending migrations.

Catches the class of defective output described in
``BACK-20260425_1711-CleverRiver``: aibox sync emitting "same-version"
migrations whose baseline appears empty so every on-disk file is
labelled ``new-upstream``, with ``affected_files`` empty despite the
body listing hundreds of rows. Rejected manually as
``MIG-20260425T164303``; this check makes the next instance detectable
before it is applied.

Two frontmatter-only invariants (no body parsing needed):

1. Same-version migrations should be a no-op. If
   ``from_version == to_version`` **and** ``affected_groups`` or
   ``affected_files`` is non-empty, the document is almost certainly
   the "empty baseline" defect.

2. ``affected_files`` length must match the body rows. The frontmatter
   ships a structured ``affected_files`` array; if ``affected_groups``
   is populated but ``affected_files`` is empty, the post-processor
   that derives the file list saw nothing — a clear inconsistency.

Both fire as WARN per pending migration; the suggested fix is to
reject via the migration-management MCP and refile the upstream
diff-generator bug.
"""

from __future__ import annotations

from pathlib import Path

from .common import CheckResult
from .migrations import _candidate_pending_paths, _load_frontmatter


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    results: list[CheckResult] = []
    pending_seen = 0

    for path in _candidate_pending_paths(repo_root):
        fm = _load_frontmatter(path)
        if not fm or fm.get("kind") not in (None, "Migration"):
            continue
        pending_seen += 1
        spec = fm.get("spec") or {}
        md = fm.get("metadata") or {}
        mid = md.get("id") or path.stem
        from_v = spec.get("from_version")
        to_v = spec.get("to_version")
        affected_groups = spec.get("affected_groups") or []
        affected_files = spec.get("affected_files") or []

        if from_v and to_v and from_v == to_v and (
            affected_groups or affected_files
        ):
            results.append(CheckResult(
                severity="WARN",
                category="migration_integrity",
                id="migration_integrity.same-version-with-content",
                message=(
                    f"{mid}: from_version == to_version ({from_v}) but "
                    f"affected_groups={len(affected_groups)}, "
                    f"affected_files={len(affected_files)}; same-version "
                    "migrations should be no-op (likely empty-baseline "
                    "defect — see BACK-20260425_1711-CleverRiver)."
                ),
                entity_ref=mid,
                suggested_fix=(
                    "reject via migration-management MCP and refile "
                    "the upstream diff-generator bug"
                ),
                fix_mcp_tool="mcp__processkit-migration-management__reject_migration",
            ))

        if affected_groups and not affected_files:
            results.append(CheckResult(
                severity="WARN",
                category="migration_integrity",
                id="migration_integrity.affected-files-empty",
                message=(
                    f"{mid}: affected_groups has {len(affected_groups)} "
                    "entries but affected_files is empty — body rows and "
                    "file list are inconsistent (likely post-processor "
                    "defect — see BACK-20260425_1711-CleverRiver)."
                ),
                entity_ref=mid,
                suggested_fix=(
                    "reject via migration-management MCP and refile "
                    "the upstream diff-generator bug"
                ),
                fix_mcp_tool="mcp__processkit-migration-management__reject_migration",
            ))

    if not results:
        results.append(CheckResult(
            severity="INFO",
            category="migration_integrity",
            id="migration_integrity.in-sync",
            message=(
                f"{pending_seen} pending migration(s) — all frontmatter "
                "invariants pass."
            ),
        ))

    return results
