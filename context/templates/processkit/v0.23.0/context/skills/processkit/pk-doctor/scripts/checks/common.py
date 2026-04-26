"""Shared dataclasses + helpers for pk-doctor checks.

Every check module exports:

    def run(ctx) -> list[CheckResult]: ...

and optionally:

    def run_fix(ctx, results) -> list[dict]: ...

`ctx` is a dict produced by doctor.py with at minimum:

    {
      "repo_root":  Path to the repo root,
      "since":      Optional[str] git ref, or None,
      "yes":        bool, auto-confirm non-data-loss prompts,
      "interactive": bool, sys.stdin.isatty(),
    }

CheckResult is the single vocabulary for findings.
"""

from __future__ import annotations

from dataclasses import dataclass, field, asdict
from typing import Literal, Optional


Severity = Literal["ERROR", "WARN", "INFO"]


@dataclass
class CheckResult:
    severity: Severity
    category: str
    id: str
    message: str
    entity_ref: Optional[str] = None
    fixable: bool = False
    suggested_fix: Optional[str] = None
    fix_mcp_tool: Optional[str] = None
    data_loss: bool = False
    extra: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        d = asdict(self)
        if not d["extra"]:
            d.pop("extra")
        return d


def tally(results: list[CheckResult]) -> dict[str, int]:
    """Return {'ERROR': n, 'WARN': n, 'INFO': n} for a list of results."""
    out = {"ERROR": 0, "WARN": 0, "INFO": 0}
    for r in results:
        out[r.severity] = out.get(r.severity, 0) + 1
    return out
