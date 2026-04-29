#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""pk-doctor — aggregator health check for a processkit repo (Phase 1).

Runs a fixed suite of detect-only checks and emits:

    - a human-readable report on stdout
    - a single `doctor.report` LogEntry via the event-log MCP tool
      (written by the invoking agent, see write_logentry_stub section)

Detect-only by default. --fix / --fix-all opt in to scoped repairs that
route through existing MCP write tools only. Doctor does not hand-edit
files under context/.

Usage:

    doctor.py [--category=LIST] [--fix=LIST | --fix-all]
              [--since=REF] [--yes]

Exit 0 if no ERRORs; 1 otherwise.

Provenance: generated as part of WorkItem BACK-20260420_1631-ProudGlade,
shape per DecisionRecord DEC-20260420_1631-WiseGarnet.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional


DOCTOR_VERSION = "1.0.0"
LOGENTRY_STUB_ENV = "PK_DOCTOR_LOGENTRY_STUB"  # tests set this

# ---------------------------------------------------------------------------
# Path bootstrap — allow imports relative to this file regardless of cwd.
# ---------------------------------------------------------------------------
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from checks import REGISTRY, get as get_check, names as check_names  # noqa: E402
from checks.common import CheckResult, tally  # noqa: E402


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="pk-doctor",
        description="Aggregator health check for a processkit repo.",
    )
    p.add_argument("--category", default=None,
                   help="Comma-list of categories to run. Default: all.")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--fix", default=None,
                   help="Comma-list of categories to enable fixes for.")
    g.add_argument("--fix-all", action="store_true",
                   help="Enable fixes for every supporting category.")
    p.add_argument("--since", default=None,
                   help="Git ref; restricts file-walk checks to changed files.")
    p.add_argument("--yes", action="store_true",
                   help="Auto-confirm non-data-loss fix prompts.")
    p.add_argument("--repo-root", default=None,
                   help="(test helper) explicit repo root. Defaults to git rev-parse.")
    p.add_argument("--no-log", action="store_true",
                   help="(test helper) skip event-log emission.")
    return p.parse_args(argv)


def _resolve_repo_root(explicit: Optional[str]) -> Path:
    if explicit:
        return Path(explicit).resolve()
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, check=True,
        )
        return Path(out.stdout.strip()).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return Path.cwd()


def _resolve_since_files(repo_root: Path, since: Optional[str]) -> Optional[set[Path]]:
    if not since:
        return None
    try:
        out = subprocess.run(
            ["git", "-C", str(repo_root), "diff", "--name-only", f"{since}...HEAD"],
            capture_output=True, text=True, check=True,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return set()  # conservative: nothing matches → skip file-walks
    return {(repo_root / line).resolve() for line in out.stdout.splitlines() if line}


def _selected_categories(arg: Optional[str]) -> list[str]:
    if not arg:
        return check_names()
    requested = [x.strip() for x in arg.split(",") if x.strip()]
    unknown = [x for x in requested if x not in check_names()]
    if unknown:
        raise SystemExit(f"error: unknown categories: {unknown}")
    return requested


def _fix_categories(args: argparse.Namespace) -> set[str]:
    if args.fix_all:
        return set(check_names())
    if args.fix:
        return {x.strip() for x in args.fix.split(",") if x.strip()}
    return set()


# ---------------------------------------------------------------------------
# Report formatting
# ---------------------------------------------------------------------------

SEV_GLYPH = {"ERROR": "E", "WARN": "W", "INFO": "i"}


def _format_report(
    per_cat: dict[str, list[CheckResult]],
    fixes_applied: list[dict],
    duration_ms: int,
    invocation: str,
) -> str:
    lines: list[str] = []
    lines.append(f"# pk-doctor v{DOCTOR_VERSION}")
    lines.append(f"invocation: {invocation}")
    lines.append(f"duration:   {duration_ms} ms")
    lines.append("")
    grand = {"ERROR": 0, "WARN": 0, "INFO": 0}
    for cat, res in per_cat.items():
        t = tally(res)
        for k, v in t.items():
            grand[k] += v
        lines.append(f"## {cat} — {t['ERROR']} ERROR / {t['WARN']} WARN / {t['INFO']} INFO")
        for r in res:
            if r.severity == "INFO":
                lines.append(f"  [i] {r.message}")
            else:
                g = SEV_GLYPH[r.severity]
                tail = f"  (fix: {r.suggested_fix})" if r.suggested_fix else ""
                lines.append(f"  [{g}] {r.id} — {r.message}{tail}")
        lines.append("")
    lines.append(f"## totals — {grand['ERROR']} ERROR / {grand['WARN']} WARN / {grand['INFO']} INFO")
    if fixes_applied:
        lines.append("")
        lines.append(f"## fixes ({len(fixes_applied)})")
        for fx in fixes_applied:
            lines.append(f"  - {fx}")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# LogEntry emission
# ---------------------------------------------------------------------------

def _emit_logentry(
    repo_root: Path,
    invocation: str,
    per_cat: dict[str, list[CheckResult]],
    fixes_applied: list[dict],
    duration_ms: int,
) -> dict:
    """Write the doctor.report payload.

    Doctor.py is a subprocess — it cannot call MCP tools directly. The
    contract is: write the payload to a well-known path (or stdout
    under PK_DOCTOR_LOGENTRY_STUB) so the calling agent can relay it
    via mcp__processkit-event-log__log_event. Returns the payload.
    """
    grand = {"ERROR": 0, "WARN": 0, "INFO": 0}
    top_findings: list[dict] = []
    per_cat_tally: dict[str, dict[str, int]] = {}
    for cat, res in per_cat.items():
        t = tally(res)
        per_cat_tally[cat] = t
        for k, v in t.items():
            grand[k] += v
        for r in res:
            if r.severity != "INFO" and len(top_findings) < 20:
                top_findings.append(r.to_dict())

    payload = {
        "event_type": "doctor.report",
        "summary": (
            f"{invocation} — "
            f"{grand['ERROR']} ERROR / {grand['WARN']} WARN / {grand['INFO']} INFO"
        ),
        "details": {
            "doctor_version": DOCTOR_VERSION,
            "invocation": invocation,
            "categories": per_cat_tally,
            "top_findings": top_findings,
            "fixes_applied": fixes_applied,
            "duration_ms": duration_ms,
        },
    }
    import os
    stub = os.environ.get(LOGENTRY_STUB_ENV)
    if stub:
        Path(stub).write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    return payload


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main(argv: list[str]) -> int:
    args = _parse_args(argv)
    repo_root = _resolve_repo_root(args.repo_root)
    selected = _selected_categories(args.category)
    fix_cats = _fix_categories(args)

    # Reconstruct the invocation string for the report + logentry.
    invocation_bits = ["/pk-doctor"]
    if args.category:
        invocation_bits.append(f"--category={args.category}")
    if args.fix_all:
        invocation_bits.append("--fix-all")
    elif args.fix:
        invocation_bits.append(f"--fix={args.fix}")
    if args.since:
        invocation_bits.append(f"--since={args.since}")
    if args.yes:
        invocation_bits.append("--yes")
    invocation = " ".join(invocation_bits)

    ctx = {
        "repo_root": repo_root,
        "since": args.since,
        "since_files": _resolve_since_files(repo_root, args.since),
        "yes": args.yes,
        "interactive": sys.stdin.isatty(),
    }

    t0 = time.monotonic()
    per_cat: dict[str, list[CheckResult]] = {}
    fixes_applied: list[dict] = []
    for cat in selected:
        mod = get_check(cat)
        try:
            res = mod.run(ctx)
        except Exception as e:  # pragma: no cover — defensive
            res = [CheckResult(
                severity="ERROR",
                category=cat,
                id=f"{cat}.exception",
                message=f"check raised: {type(e).__name__}: {e}",
            )]
        per_cat[cat] = res

        if cat in fix_cats and hasattr(mod, "run_fix"):
            if not ctx["interactive"] and not ctx["yes"]:
                per_cat[cat].append(CheckResult(
                    severity="WARN",
                    category=cat,
                    id="fix.non-interactive",
                    message="fix requires interactive prompt, re-run from terminal",
                ))
            else:
                try:
                    fixes_applied.extend(mod.run_fix(ctx, res))
                except Exception as e:  # pragma: no cover
                    per_cat[cat].append(CheckResult(
                        severity="ERROR",
                        category=cat,
                        id=f"{cat}.fix-exception",
                        message=f"fix raised: {type(e).__name__}: {e}",
                    ))

    duration_ms = int((time.monotonic() - t0) * 1000)

    # stdout report
    report = _format_report(per_cat, fixes_applied, duration_ms, invocation)
    sys.stdout.write(report)

    # logentry
    if not args.no_log:
        _emit_logentry(repo_root, invocation, per_cat, fixes_applied, duration_ms)

    # exit code
    grand_err = sum(tally(r)["ERROR"] for r in per_cat.values())
    return 1 if grand_err else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
