#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
#   "mcp[cli]>=1.0",
#   "jsonschema>=4.0",
# ]
# ///
"""pk-retro — post-release blameless retrospective generator.

Pulls signals from the MCP index and emits:

  1. An Artifact (markdown) saved via create_artifact MCP tool.
  2. A LogEntry (event_type=retro.completed) via log_event MCP tool.

The Artifact is emitted first; the LogEntry references its ID.
If create_artifact fails, log_event is NOT called (atomicity).

Usage:

    python3 pk_retro.py --release v0.18.2
        [--since <iso-or-ref>] [--until <iso-or-ref>]
        [--auto-workitems] [--verbose] [--dry-run]
        [--notes-file <path>] [--repo-root <path>]

Exit 0 on success. Exit 1 on error.

Provenance: generated as part of WorkItem
BACK-20260420_1340-LoyalFrog-add-pk-retro-skill.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable


RETRO_VERSION = "1.0.0"

# MCP server modules to import in-process, relative to the processkit
# skills directory (.../context/skills/processkit/).
#
# File layout:
#   context/skills/processkit/retrospective/scripts/pk_retro.py
#                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
#   parent(0) = scripts/
#   parent(1) = retrospective/
#   parent(2) = processkit/          ← _PK_SKILLS_ROOT
_PK_SKILLS_ROOT = Path(__file__).resolve().parent.parent.parent

_MCP_SERVERS: dict[str, tuple[str, str]] = {
    # tool-name → (relative-path-to-server.py, function-name)
    "create_artifact": (
        "artifact-management/mcp/server.py",
        "create_artifact",
    ),
    "log_event": (
        "event-log/mcp/server.py",
        "log_event",
    ),
    "create_workitem": (
        "workitem-management/mcp/server.py",
        "create_workitem",
    ),
}


def _load_production_mcp() -> dict[str, Callable]:
    """Load real MCP callables by importing each server module directly.

    Returns a dict mapping tool-name → callable, matching the shape the
    existing _emit_* functions already expect (mcp.get("create_artifact"),
    etc.).  Falls back gracefully: any individual import failure is
    re-raised so main() can surface it as a fatal error rather than
    silently producing no output.
    """
    result: dict[str, Callable] = {}

    for tool_name, (rel_path, fn_name) in _MCP_SERVERS.items():
        server_path = _PK_SKILLS_ROOT / rel_path
        spec = importlib.util.spec_from_file_location(
            f"_pk_mcp_{tool_name}", server_path
        )
        if spec is None or spec.loader is None:
            raise ImportError(
                f"Cannot locate MCP server module: {server_path}"
            )
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)  # type: ignore[union-attr]
        fn = getattr(mod, fn_name, None)
        if fn is None or not callable(fn):
            raise AttributeError(
                f"Function {fn_name!r} not found or not callable "
                f"in {server_path}"
            )
        result[tool_name] = fn

    return result

# Per-section line budgets (enforced when --verbose is off)
_LINE_BUDGETS: dict[str, int] = {
    "release_summary": 8,
    "timeline": 10,
    "signals": 8,
    "what_held": 7,
    "what_slipped": 8,
    "action_items": 8,
    "learned": 6,
}
_TOTAL_BUDGET = 80

# ---------------------------------------------------------------------------
# Path bootstrap
# ---------------------------------------------------------------------------
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from signals import release_summary as _rs  # noqa: E402
from signals import timeline as _tl  # noqa: E402
from signals import workitems as _wi  # noqa: E402
from signals import drift as _dr  # noqa: E402


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="pk-retro",
        description="Generate a post-release blameless retrospective.",
    )
    p.add_argument(
        "--release", required=True,
        help="Release version tag, e.g. v0.18.2",
    )
    p.add_argument(
        "--since", default=None,
        help="Window start (ISO datetime or git ref). Default: prior release.",
    )
    p.add_argument(
        "--until", default=None,
        help="Window end (ISO datetime or git ref). Default: --release marker.",
    )
    p.add_argument(
        "--auto-workitems", action="store_true",
        help="Create Action Items as WorkItems via MCP.",
    )
    p.add_argument(
        "--verbose", action="store_true",
        help="Disable section line caps; include full entity bodies.",
    )
    p.add_argument(
        "--dry-run", action="store_true",
        help="Skip all MCP writes; print rendered Artifact to stdout only.",
    )
    p.add_argument(
        "--notes-file", default=None,
        help="Path to owner notes file for the optional Learned section.",
    )
    p.add_argument(
        "--repo-root", default=None,
        help="(test helper) explicit repo root.",
    )
    p.add_argument(
        "--no-mcp", action="store_true",
        help="(test helper) skip all MCP calls.",
    )
    return p.parse_args(argv)


def _resolve_repo_root(explicit: str | None) -> Path:
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


# ---------------------------------------------------------------------------
# Signal collection
# ---------------------------------------------------------------------------

def _collect_signals(ctx: dict) -> dict[str, dict]:
    """Run all four signal modules and return their outputs."""
    return {
        "release_summary": _rs.collect(ctx),
        "timeline": _tl.collect(ctx),
        "workitems": _wi.collect(ctx),
        "drift": _dr.collect(ctx),
    }


# ---------------------------------------------------------------------------
# Artifact rendering
# ---------------------------------------------------------------------------

def _render_artifact(
    release: str,
    signals: dict[str, dict],
    verbose: bool,
    auto_workitems: bool,
    notes: str | None,
) -> str:
    """Render the full retrospective Artifact markdown."""
    rs = signals["release_summary"]
    tl = signals["timeline"]
    wi = signals["workitems"]
    dr = signals["drift"]

    lines: list[str] = []
    lines.append(f"# Retrospective — {release}")
    lines.append("")

    # ── Section 1: Release Summary ────────────────────────────────────────
    lines.append("## Release Summary")
    lines.append("")
    lines.append(f"- **Version:** {rs.get('version', release)}")
    lines.append(f"- **Release date:** {rs.get('release_date', '') or 'unknown'}")
    lines.append(f"- **Commit count:** {rs.get('commit_count', 0)}")
    contribs = rs.get("contributors", [])
    if contribs:
        lines.append(f"- **Contributors:** {', '.join(contribs)}")
    prior = rs.get("prior_release")
    if prior:
        lines.append(
            f"- **Prior release:** {prior} "
            f"({rs.get('prior_release_date', '') or 'unknown'})"
        )
    lines.append("")
    deliverables = rs.get("top_deliverables", [])
    if deliverables:
        lines.append("**Top deliverables:**")
        for d in deliverables[:3]:
            lines.append(f"- {d}")
        lines.append("")
    sec1_end = len(lines)

    # ── Section 2: Timeline ───────────────────────────────────────────────
    lines.append("## Timeline")
    lines.append("")
    sessions = tl.get("sessions", [])
    milestones = tl.get("milestones", [])

    timeline_items: list[str] = []
    for m in milestones:
        ts = m.get("timestamp", "")[:10]
        desc = m.get("description", "")
        entity = m.get("entity", "")
        ref = f" ({entity})" if entity else ""
        timeline_items.append(f"- [{ts}] {desc}{ref}")

    session_count = tl.get("session_count", 0)
    if session_count:
        timeline_items.append(
            f"- {session_count} session(s) in release window"
        )
    for sess in sessions[:5]:
        start = (sess.get("start") or "")[:10]
        end = (sess.get("end") or "")[:10]
        summ = sess.get("summary", "")
        if start or end:
            range_str = f"{start}–{end}" if (start and end) else (start or end)
            item = f"- Session: {range_str}"
            if summ:
                item += f" — {summ[:60]}"
            timeline_items.append(item)

    if not timeline_items:
        timeline_items = ["- No session boundary data found in window."]

    cap = _LINE_BUDGETS["timeline"] if not verbose else 999
    for item in timeline_items[:cap]:
        lines.append(item)
    if not verbose and len(timeline_items) > cap:
        lines.append("- … (truncated — run with --verbose)")
    lines.append("")

    # ── Section 3: Signals (DORA-like) ────────────────────────────────────
    lines.append("## Signals")
    lines.append("")
    prior_date = rs.get("prior_release_date", "")
    rel_date = rs.get("release_date", "")
    lead_time = _compute_lead_time(prior_date, rel_date)
    total_closed = wi.get("total_closed", 0)
    bug_closed = wi.get("bug_closed", 0)
    cfr = (
        f"{bug_closed}/{total_closed} ({100*bug_closed//total_closed}%)"
        if total_closed > 0
        else "0/0 (n/a)"
    )

    lines.append(f"- **Lead time proxy:** {lead_time}")
    lines.append(f"- **Change failure rate proxy:** {cfr} "
                 "(bug WorkItems closed / total closed)")
    lines.append(
        f"- **WorkItems closed this cycle:** {total_closed} "
        f"({wi.get('held_count', 0)} held, "
        f"{wi.get('slipped_count', 0)} slipped)"
    )
    if verbose:
        e_delta = dr.get("doctor_error_delta", 0)
        w_delta = dr.get("doctor_warn_delta", 0)
        lines.append(
            f"- **Doctor delta:** ERROR {_delta_str(e_delta)}, "
            f"WARN {_delta_str(w_delta)}"
        )
        lines.append(
            f"- **Skip markers in window:** "
            f"{dr.get('skip_marker_count', 0)}"
        )
        if prior_date and rel_date:
            lines.append(
                f"  - Lead time computed as: "
                f"release_date({rel_date}) − prior_release_date({prior_date})"
            )
        lines.append(
            f"  - CFR = bug WorkItems with completed_at in window "
            f"/ all WorkItems with completed_at in window"
        )
    lines.append("")

    # ── Section 4: What Held ─────────────────────────────────────────────
    lines.append("## What Held")
    lines.append("")
    held = wi.get("held", [])
    if held:
        cap = _LINE_BUDGETS["what_held"] if not verbose else 999
        held_body: list[str] = []
        for item in held:
            wi_id = item.get("id", "")
            title = item.get("title", "unknown")
            ref = f" ({wi_id})" if wi_id else ""
            if verbose:
                state = item.get("state", "")
                completed = item.get("completed_at", "")[:10]
                held_body.append(
                    f"- {title}{ref} — {state}; completed {completed}"
                )
            else:
                held_body.append(f"- {title}{ref}")
        for line in held_body[:cap]:
            lines.append(line)
        if not verbose and len(held_body) > cap:
            lines.append("- … (truncated — run with --verbose)")
    else:
        lines.append("- No held WorkItems found in window.")
    lines.append("")

    # ── Section 5: What Slipped ───────────────────────────────────────────
    lines.append("## What Slipped")
    lines.append("")
    slipped = wi.get("slipped", [])
    drift_signals = dr.get("slipped_signals", [])
    slip_lines: list[str] = []

    # WorkItem slips
    for item in slipped:
        wi_id = item.get("id", "")
        title = item.get("title", "unknown")
        state = item.get("state", "")
        reason = item.get("reason", "")

        if wi_id:
            ref = f" ({wi_id})"
            detail = f" — {state}"
            if reason:
                detail += f"; {reason[:80]}"
            if verbose:
                deferred = item.get("deferred_at", "")[:10]
                detail += f"; deferred {deferred}"
            slip_lines.append(f"- {title}{ref}{detail}")
        else:
            slip_lines.append(
                f"- [uncertain: WorkItem title='{title}' has no ID]"
            )

    # Drift signals
    for sig in drift_signals:
        desc = sig.get("description", "")
        entity = sig.get("entity")
        certain = sig.get("certain", False)
        if certain and entity:
            slip_lines.append(f"- {desc} ({entity})")
        else:
            slip_lines.append(f"- [uncertain: {desc}]")

    if not slip_lines:
        slip_lines = ["- No slipped WorkItems or drift signals found in window."]

    cap = _LINE_BUDGETS["what_slipped"] if not verbose else 999
    for line in slip_lines[:cap]:
        lines.append(line)
    if not verbose and len(slip_lines) > cap:
        lines.append("- … (truncated — run with --verbose)")
    lines.append("")

    # ── Section 6: Action Items (proposed) ───────────────────────────────
    lines.append("## Action Items (proposed)")
    lines.append("")
    action_items = _derive_action_items(slipped, drift_signals)
    if action_items:
        cap = _LINE_BUDGETS["action_items"] if not verbose else 999
        for item in action_items[:cap]:
            lines.append(f"- {item}")
        if not verbose and len(action_items) > cap:
            lines.append("- … (truncated — run with --verbose)")
    else:
        lines.append("- No action items derived from slipped signals.")
    lines.append("")
    if not auto_workitems:
        lines.append(
            "> Pass --auto-workitems to create these as proposed WorkItems."
        )
        lines.append("")

    # ── Section 7: Learned (optional) ─────────────────────────────────────
    if notes:
        lines.append("## Learned")
        lines.append("")
        note_lines = [ln.rstrip() for ln in notes.splitlines() if ln.strip()]
        cap = _LINE_BUDGETS["learned"] if not verbose else 999
        for ln in note_lines[:cap]:
            if not ln.startswith("-"):
                ln = f"- {ln}"
            lines.append(ln)
        if not verbose and len(note_lines) > cap:
            lines.append("- … (truncated — run with --verbose)")
        lines.append("")

    # ── Verbose appendix ──────────────────────────────────────────────────
    if verbose:
        lines.append("## Appendix A — Raw Signal Dumps")
        lines.append("")
        lines.append("```json")
        import json as _json
        raw_dumps = {
            k: v.get("raw", {}) for k, v in signals.items()
        }
        lines.append(_json.dumps(raw_dumps, indent=2, default=str))
        lines.append("```")
        lines.append("")

    body = "\n".join(lines)

    # Global line cap (only for non-verbose)
    if not verbose:
        body_lines = body.splitlines()
        if len(body_lines) > _TOTAL_BUDGET:
            body_lines = body_lines[:_TOTAL_BUDGET]
            body_lines.append(
                "\n*(truncated — run with --verbose for full output)*"
            )
            body = "\n".join(body_lines)

    return body


def _compute_lead_time(prior_date: str, release_date: str) -> str:
    """Return human-readable lead time between two ISO dates."""
    if not prior_date or not release_date:
        return "unknown [uncertain: no prior release marker found]"
    try:
        from datetime import date
        d1 = date.fromisoformat(prior_date[:10])
        d2 = date.fromisoformat(release_date[:10])
        delta = (d2 - d1).days
        return f"{delta} day(s) ({prior_date[:10]} → {release_date[:10]})"
    except Exception:
        return f"{prior_date[:10]} → {release_date[:10]}"


def _delta_str(n: int) -> str:
    if n > 0:
        return f"+{n}"
    if n < 0:
        return str(n)
    return "0 (no change)"


def _derive_action_items(
    slipped: list[dict],
    drift_signals: list[dict],
) -> list[str]:
    """Derive proposed action items from slipped WorkItems and drift signals."""
    items: list[str] = []
    for wi in slipped:
        title = wi.get("title", "")
        wi_id = wi.get("id", "")
        if title and wi_id:
            items.append(
                f"Revisit '{title}' ({wi_id}) — "
                f"deferred this cycle, reassess for next release"
            )
    for sig in drift_signals:
        desc = sig.get("description", "")
        if desc and sig.get("certain"):
            items.append(
                f"Address: {desc}"
            )
    return items


# ---------------------------------------------------------------------------
# MCP emission
# ---------------------------------------------------------------------------

def _strip_markdown(text: str) -> str:
    """Strip markdown syntax for WorkItem titles."""
    text = re.sub(r"[*_`\[\]#]", "", text)
    return text.strip()[:80]


def _emit_artifact(
    mcp: dict,
    release: str,
    body: str,
    dry_run: bool,
) -> dict | None:
    """Create the retrospective Artifact. Returns the created entity or None."""
    if dry_run:
        print(body)
        return None

    create_fn = mcp.get("create_artifact")
    if not create_fn:
        print(
            "[pk-retro] WARNING: create_artifact not available; "
            "use --dry-run to preview.",
            file=sys.stderr,
        )
        return None

    try:
        result = create_fn(
            name=f"Retrospective — {release}",
            kind="document",
            format="markdown",
            tags=["retrospective", "release"],
            body=body,
        )
        return result
    except Exception as e:
        print(f"[pk-retro] ERROR: create_artifact failed: {e}", file=sys.stderr)
        return None


def _emit_logentry(
    mcp: dict,
    artifact: dict,
    release: str,
    dry_run: bool,
) -> bool:
    """Emit the retro.completed LogEntry. Returns True on success."""
    if dry_run:
        return True

    log_fn = mcp.get("log_event")
    if not log_fn:
        print(
            "[pk-retro] WARNING: log_event not available.",
            file=sys.stderr,
        )
        return False

    artifact_id = (
        artifact.get("metadata", {}).get("id")
        or artifact.get("id")
        or ""
    )

    try:
        log_fn(
            event_type="retro.completed",
            summary=f"Retrospective completed — {release}",
            subject=artifact_id,
            subject_kind="Artifact",
            details={
                "release": release,
                "artifact_id": artifact_id,
                "retro_version": RETRO_VERSION,
            },
        )
        return True
    except Exception as e:
        print(
            f"[pk-retro] ERROR: log_event failed: {e}",
            file=sys.stderr,
        )
        return False


def _emit_action_item_workitems(
    mcp: dict,
    action_items: list[str],
    artifact_id: str,
    dry_run: bool,
) -> list[dict]:
    """Create WorkItems for each action item. Returns list of created items."""
    if dry_run or not action_items:
        return []

    create_fn = mcp.get("create_workitem")
    log_fn = mcp.get("log_event")
    if not create_fn:
        print(
            "[pk-retro] WARNING: create_workitem not available.",
            file=sys.stderr,
        )
        return []

    created: list[dict] = []
    for item_text in action_items:
        # Strip retro-type: prefix if present
        wi_type = "chore"
        text = item_text
        if text.startswith("retro-type:"):
            parts = text.split(None, 1)
            type_part = parts[0].split(":", 1)[1].strip()
            if type_part:
                wi_type = type_part
            text = parts[1].strip() if len(parts) > 1 else text

        title = _strip_markdown(text)
        description = (
            f"{text}\n\nSource retro: {artifact_id}"
        )
        try:
            wi = create_fn(
                title=title,
                description=description,
                type=wi_type,
                priority="medium",
                state="backlog",
            )
            created.append(wi)
            # Emit linking LogEntry
            wi_id = (
                wi.get("metadata", {}).get("id")
                or wi.get("id")
                or ""
            )
            if log_fn and wi_id:
                try:
                    log_fn(
                        event_type="retro.action_item_created",
                        summary=(
                            f"Retro action item created: {title[:60]}"
                        ),
                        subject=wi_id,
                        subject_kind="WorkItem",
                        details={
                            "workitem_id": wi_id,
                            "artifact_id": artifact_id,
                            "action_text": text[:200],
                        },
                    )
                except Exception:
                    pass  # LogEntry failure doesn't block WI creation
        except Exception as e:
            print(
                f"[pk-retro] WARNING: create_workitem failed for "
                f"'{title[:40]}': {e}",
                file=sys.stderr,
            )

    return created


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main(argv: list[str], mcp_overrides: dict | None = None) -> int:
    """Run pk-retro. mcp_overrides injects mock callables for testing."""
    args = _parse_args(argv)
    repo_root = _resolve_repo_root(args.repo_root)

    # MCP callable stubs: tests inject mcp_overrides; production uses the
    # in-process loader; --dry-run and --no-mcp both skip the loader.
    if mcp_overrides is not None:
        mcp: dict = dict(mcp_overrides)
    elif args.dry_run or args.no_mcp:
        mcp = {}
    else:
        try:
            mcp = _load_production_mcp()
        except (ImportError, AttributeError, Exception) as exc:
            print(
                f"[pk-retro] WARNING: in-process MCP loader failed: {exc}. "
                "Use --dry-run to preview or --no-mcp to skip.",
                file=sys.stderr,
            )
            return 1

    # Read notes file if provided
    notes: str | None = None
    if args.notes_file:
        try:
            notes = Path(args.notes_file).read_text(encoding="utf-8")
        except Exception as e:
            print(
                f"[pk-retro] WARNING: could not read notes-file: {e}",
                file=sys.stderr,
            )

    # Build signal context
    ctx: dict[str, Any] = {
        "repo_root": repo_root,
        "release": args.release,
        "since": args.since,
        "until": args.until,
        "verbose": args.verbose,
        "mcp": mcp,
    }

    # Collect signals
    signals = _collect_signals(ctx)

    # Render the Artifact body
    body = _render_artifact(
        release=args.release,
        signals=signals,
        verbose=args.verbose,
        auto_workitems=args.auto_workitems,
        notes=notes,
    )

    # ── Dry run: print + exit ─────────────────────────────────────────────
    if args.dry_run:
        print(body)
        return 0

    # ── Emit Artifact (Phase 1) ───────────────────────────────────────────
    artifact = _emit_artifact(mcp, args.release, body, dry_run=False)
    if artifact is None:
        print(
            "[pk-retro] ERROR: Artifact creation failed; "
            "LogEntry NOT emitted.",
            file=sys.stderr,
        )
        return 1

    artifact_id = (
        artifact.get("metadata", {}).get("id")
        or artifact.get("id")
        or ""
    )

    # ── Emit LogEntry (atomicity: only after Artifact succeeds) ───────────
    ok = _emit_logentry(mcp, artifact, args.release, dry_run=False)
    if not ok:
        print(
            "[pk-retro] WARNING: Artifact was created but LogEntry failed.",
            file=sys.stderr,
        )

    # ── Auto-workitems (Phase 2) ──────────────────────────────────────────
    if args.auto_workitems and artifact_id:
        wi_data = signals["workitems"]
        drift_data = signals["drift"]
        action_items = _derive_action_items(
            wi_data.get("slipped", []),
            drift_data.get("slipped_signals", []),
        )
        created = _emit_action_item_workitems(
            mcp, action_items, artifact_id, dry_run=False
        )
        if created:
            print(
                f"[pk-retro] Created {len(created)} action-item WorkItem(s)."
            )

    print(f"[pk-retro] Retrospective filed: {artifact_id or '(unknown id)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
