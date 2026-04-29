"""team_consistency check — wraps team-manager.check_all().

Runs all 10 team-member consistency checks defined by the team-manager
skill (DEC-20260422_0233-SpryTulip) and surfaces findings in the pk-doctor
report vocabulary.

If the team-manager skill is not installed, emits one INFO finding and
returns without erroring — lets pk-doctor stay usable in projects that
don't ship team-manager.
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

from .common import CheckResult


def _add_processkit_lib_to_path(repo_root: Path) -> None:
    """Mirror the team-manager server's _find_lib helper so consistency.py's
    `from processkit import entity, schema` imports resolve.
    """
    candidates = [
        repo_root / "context" / "skills" / "_lib",
        repo_root / "src" / "lib",
        repo_root / "_lib",
    ]
    for c in candidates:
        if (c / "processkit" / "__init__.py").is_file() and str(c) not in sys.path:
            sys.path.insert(0, str(c))
            return


def _load_consistency_module(repo_root: Path):
    path = repo_root / "context" / "skills" / "processkit" / "team-manager" / "scripts" / "consistency.py"
    if not path.is_file():
        return None
    _add_processkit_lib_to_path(repo_root)
    spec = importlib.util.spec_from_file_location("_pk_doctor_tm_consistency", path)
    mod = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(mod)
    except Exception:
        return None
    return mod


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    mod = _load_consistency_module(repo_root)
    if mod is None:
        return [CheckResult(
            severity="INFO",
            category="team_consistency",
            id="team_consistency.skill-not-installed",
            message="team-manager skill not found; team consistency checks skipped",
        )]

    tm_root = repo_root / "context" / "team-members"
    pool_path = repo_root / "context" / "skills" / "processkit" / "team-manager" / "data" / "name-pool.yaml"
    assets_dir = repo_root / "context" / "skills" / "processkit" / "team-manager" / "assets"

    try:
        report = mod.check_all(repo_root, tm_root, pool_path, assets_dir)
    except Exception as e:
        return [CheckResult(
            severity="ERROR",
            category="team_consistency",
            id="team_consistency.internal-error",
            message=f"check_all raised: {type(e).__name__}: {e}",
        )]

    members = report.get("members", {}) or {}
    summary = report.get("summary", {}) or {}

    if not members:
        return [CheckResult(
            severity="INFO",
            category="team_consistency",
            id="team_consistency.no-members",
            message="no team-members present under context/team-members/",
        )]

    results: list[CheckResult] = []
    sev_map = {"error": "ERROR", "warning": "WARN", "info": "INFO"}
    for slug, findings in members.items():
        for f in findings:
            sev = sev_map.get((f.get("severity") or "warning").lower(), "WARN")
            extra: dict = {}
            if f.get("path"):
                extra["path"] = f["path"]
            results.append(CheckResult(
                severity=sev,
                category="team_consistency",
                id=f.get("code") or "team_consistency.unknown",
                message=f.get("message") or "(no message)",
                entity_ref=f"TEAMMEMBER-{slug}",
                extra=extra,
            ))

    if not results:
        results.append(CheckResult(
            severity="INFO",
            category="team_consistency",
            id="team_consistency.clean",
            message=(
                f"checked {summary.get('count', len(members))} team-member(s); "
                f"{summary.get('errors', 0)} error(s), {summary.get('warnings', 0)} warning(s)"
            ),
        ))
    return results
