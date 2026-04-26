"""drift check — wraps scripts/check-src-context-drift.sh.

Exit 0  → one INFO "trees in sync".
Exit 1  → one WARN per non-empty line of the script's stderr report.
Exit 2+ → one ERROR (script usage error or missing).
"""

from __future__ import annotations

import subprocess
from pathlib import Path

from .common import CheckResult


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    script = repo_root / "scripts" / "check-src-context-drift.sh"
    if not script.is_file():
        return [CheckResult(
            severity="ERROR",
            category="drift",
            id="drift.script-missing",
            message=f"{script.relative_to(repo_root)} not found",
        )]

    proc = subprocess.run(
        ["bash", str(script)],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
    )
    rc = proc.returncode
    if rc == 0:
        return [CheckResult(
            severity="INFO",
            category="drift",
            id="drift.in-sync",
            message="context/ and src/context/ are in sync",
        )]
    if rc == 1:
        results: list[CheckResult] = []
        # The script prints the "Offending paths:" block on stderr
        # between the two "===" markers. Extract lines that look like
        # diff output (`Only in ...`, `Files ... differ`).
        for line in proc.stderr.splitlines():
            line = line.strip()
            if line.startswith("Only in ") or "differ" in line:
                results.append(CheckResult(
                    severity="WARN",
                    category="drift",
                    id="drift.detected",
                    message=line,
                ))
        if not results:
            results.append(CheckResult(
                severity="WARN",
                category="drift",
                id="drift.detected",
                message="check-src-context-drift.sh exit 1 (see stderr)",
            ))
        return results
    return [CheckResult(
        severity="ERROR",
        category="drift",
        id="drift.script-error",
        message=f"check-src-context-drift.sh exited {rc}: {proc.stderr.strip()[:200]}",
    )]
