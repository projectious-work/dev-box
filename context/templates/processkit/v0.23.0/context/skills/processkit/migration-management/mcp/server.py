#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "mcp[cli]>=1.0",
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""processkit migration-management MCP server.

Tools:

    list_migrations(state?) -> [migrations]

    get_migration(id) -> migration | {error}

    start_migration(id)
        -> {ok, from_state, to_state}
        (pending → in-progress; stamps spec.started_at; moves the file)

    apply_migration(id, notes?)
        -> {ok, from_state, to_state}
        (in-progress → applied; stamps spec.applied_at; moves the file;
         if called on a pending migration, implicitly runs start first)

    reject_migration(id, reason)
        -> {ok, from_state, to_state}
        (pending OR in-progress → rejected; stamps spec.rejected_reason;
         moves the file to applied/ per processkit convention)

Migrations have a state machine: pending → in-progress → applied
(terminal). Either pending or in-progress can branch to rejected
(terminal side state). The directory the file lives in mirrors the
entity's spec.state, so every write-side tool also moves the file and
refreshes ``context/migrations/INDEX.md``. See mcp/SERVER.md for the
full contract.
"""
from __future__ import annotations

import datetime as _dt
import os
import sys
from pathlib import Path
from typing import Any


def _find_lib() -> Path:
    env = os.environ.get("PROCESSKIT_LIB_PATH")
    if env:
        return Path(env).resolve()
    here = Path(__file__).resolve().parent
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


sys.path.insert(0, str(_find_lib()))

from mcp.server.fastmcp import FastMCP  # noqa: E402
from mcp.types import ToolAnnotations  # noqa: E402

from processkit import entity, index, log, paths, state_machine  # noqa: E402

server = FastMCP("processkit-migration-management")

_STATES = ("pending", "in-progress", "applied", "rejected")


def _now_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat(timespec="seconds")


def _migrations_root(root: Path) -> Path:
    return root / "context" / "migrations"


def _state_dir(root: Path, state: str) -> Path:
    # pending → pending/, in-progress → in-progress/, applied → applied/.
    # Rejected migrations park under applied/ per the migration-management
    # convention: "applied" means "decision finalized", not "code applied".
    subdir = "applied" if state == "rejected" else state
    return _migrations_root(root) / subdir


def _walk_dir(d: Path) -> list[Path]:
    if not d.is_dir():
        return []
    return sorted(p for p in d.iterdir() if p.is_file() and p.suffix == ".md")


def _load_by_id(root: Path, id: str) -> entity.Entity | None:
    """Resolve a Migration by full ID, prefix, or bare word-pair.

    Walks the three state directories first (fast, no index needed).
    Falls back to the SQLite index so partial IDs and word-pair lookups
    still work when the caller's ID is only a fragment.
    """
    # Direct filename match under any of the state directories.
    for sub in ("pending", "in-progress", "applied"):
        direct = _migrations_root(root) / sub / f"{id}.md"
        if direct.is_file():
            return entity.load(direct)

    # Prefix search under the three state directories (handles
    # "MIG-20260419T080358" when the file has a slug suffix, or a
    # leading word-pair).
    for sub in ("pending", "in-progress", "applied"):
        for p in _walk_dir(_migrations_root(root) / sub):
            stem = p.stem
            if stem == id or stem.startswith(id) or stem.endswith(id):
                try:
                    return entity.load(p)
                except entity.EntityError:
                    continue

    # Fall back to the SQLite index — handles partial IDs and any
    # word-pair lookup the caller might throw at us.
    try:
        db = index.open_db()
        try:
            row, _candidates = index.resolve_entity(db, id, kind="Migration")
        finally:
            db.close()
        if row and row.get("path"):
            return entity.load(row["path"])
    except Exception:
        pass
    return None


def _summarize(ent: entity.Entity) -> dict[str, Any]:
    spec = ent.spec
    return {
        "id": ent.id,
        "source": spec.get("source"),
        "from_version": spec.get("from_version"),
        "to_version": spec.get("to_version"),
        "state": spec.get("state"),
        "summary": spec.get("summary"),
        "path": str(ent.path) if ent.path else None,
    }


def _move_file(ent: entity.Entity, new_state: str, root: Path) -> Path:
    """Write the entity (with updated spec) into the new state directory
    and remove the old file. Follows the discussion-management pattern:
    write-then-unlink (never rename-then-write) so a crash leaves at most
    a duplicate file, never a missing one."""
    dest_dir = _state_dir(root, new_state)
    dest_dir.mkdir(parents=True, exist_ok=True)
    old_path = ent.path
    dest = dest_dir / f"{ent.id}.md"
    ent.write(dest)
    if old_path and old_path.resolve() != dest.resolve():
        try:
            old_path.unlink()
        except FileNotFoundError:
            pass
    return dest


# ---------------------------------------------------------------------------
# INDEX.md regeneration
# ---------------------------------------------------------------------------

_INDEX_HEADER = "# Migrations Index"
_CLI_TAIL_HEADING = "## CLI Migrations"


def _load_existing_notes(index_path: Path) -> dict[str, str]:
    """Parse the Applied section of an existing INDEX.md and return
    ``{migration_id: notes_column}`` so human-written Notes survive a
    regeneration. Handles the three-column pipe format produced by
    :func:`_render_applied_row`."""
    notes: dict[str, str] = {}
    if not index_path.is_file():
        return notes
    text = index_path.read_text(encoding="utf-8")
    # Scan for a line matching "| <date> | MIG-... — ... | <notes> |".
    # Accept any section heading order — we key by MIG-id in the second
    # column so the section doesn't matter.
    for raw in text.splitlines():
        line = raw.rstrip()
        if not line.startswith("|"):
            continue
        parts = [c.strip() for c in line.strip().strip("|").split("|")]
        # Expect at least 3 columns (date, migration, notes).
        if len(parts) < 3:
            continue
        mig_col = parts[1]
        # First token in the migration column is the ID (e.g. "MIG-20260410T123638").
        tokens = mig_col.split()
        if not tokens:
            continue
        mig_id = tokens[0]
        if not mig_id.startswith("MIG-"):
            continue
        notes[mig_id] = parts[2]
    return notes


def _render_applied_row(ent: entity.Entity, carry_notes: str | None) -> str:
    spec = ent.spec
    applied_at = _iso(spec.get("applied_at") or spec.get("rejected_at"))
    date = applied_at[:10] if applied_at else ""
    source = spec.get("source") or ""
    from_v = spec.get("from_version") or ""
    to_v = spec.get("to_version") or ""
    header = f"{ent.id} — {source} {from_v} → {to_v}".strip()
    notes = carry_notes if carry_notes is not None else (spec.get("summary") or "")
    return f"| {date} | {header} | {notes} |"


def _truncate(text: str, limit: int = 160) -> str:
    s = (text or "").strip().replace("\n", " ")
    return s if len(s) <= limit else s[: limit - 1] + "…"


def _iso(v: Any) -> str:
    """Coerce a spec timestamp value to an ISO-8601 string.

    YAML parsing eagerly converts ``YYYY-MM-DDTHH:MM:SSZ`` strings into
    ``datetime`` objects. The rest of the server treats timestamps as
    strings (slicing, rendering into tables, etc.) so we normalize on
    read.
    """
    if v is None:
        return ""
    if isinstance(v, str):
        return v
    if isinstance(v, _dt.datetime):
        return v.isoformat(timespec="seconds").replace("+00:00", "Z")
    return str(v)


def _read_cli_tail(index_path: Path) -> str:
    """Return the `## CLI Migrations` section of an existing INDEX.md
    verbatim (including the heading) so regeneration leaves it alone."""
    if not index_path.is_file():
        return ""
    text = index_path.read_text(encoding="utf-8")
    idx = text.find(_CLI_TAIL_HEADING)
    if idx < 0:
        return ""
    return text[idx:].rstrip() + "\n"


def _regenerate_index(root: Path) -> Path:
    """Walk pending/, in-progress/, applied/ and rewrite INDEX.md.

    Preserves:
      - the Applied section's human-written Notes column (keyed by ID).
      - the trailing ``## CLI Migrations`` section verbatim — aibox owns
        that block and it must not be mangled by this regeneration.
    """
    migrations_root = _migrations_root(root)
    index_path = migrations_root / "INDEX.md"

    existing_notes = _load_existing_notes(index_path)
    cli_tail = _read_cli_tail(index_path)

    pending_ents: list[entity.Entity] = []
    in_progress_ents: list[entity.Entity] = []
    applied_ents: list[entity.Entity] = []
    rejected_ents: list[entity.Entity] = []

    for p in _walk_dir(migrations_root / "pending"):
        try:
            pending_ents.append(entity.load(p))
        except entity.EntityError:
            continue
    for p in _walk_dir(migrations_root / "in-progress"):
        try:
            in_progress_ents.append(entity.load(p))
        except entity.EntityError:
            continue
    # applied/ holds BOTH applied and rejected files. Split them here.
    for p in _walk_dir(migrations_root / "applied"):
        try:
            ent = entity.load(p)
        except entity.EntityError:
            continue
        if ent.spec.get("state") == "rejected":
            rejected_ents.append(ent)
        else:
            applied_ents.append(ent)

    pending_ents.sort(key=lambda e: _iso(e.spec.get("generated_at")) or e.id)
    in_progress_ents.sort(key=lambda e: _iso(e.spec.get("started_at")) or e.id)
    applied_ents.sort(
        key=lambda e: _iso(e.spec.get("applied_at"))[:10] or e.id
    )
    rejected_ents.sort(
        key=lambda e: _iso(e.spec.get("rejected_at") or e.spec.get("applied_at"))[:10] or e.id
    )

    lines: list[str] = [_INDEX_HEADER, ""]

    # ── Pending ──
    lines.append(f"## Pending ({len(pending_ents)})")
    lines.append("")
    if not pending_ents:
        lines.append("None.")
    else:
        for e in pending_ents:
            s = e.spec
            src = s.get("source") or ""
            fv = s.get("from_version") or ""
            tv = s.get("to_version") or ""
            gen = _iso(s.get("generated_at"))[:10]
            head = f"- **{e.id}** — {src} {fv} → {tv}".rstrip()
            if gen:
                head += f" (generated {gen})"
            lines.append(head)
            summary = s.get("summary")
            if summary:
                lines.append(f"  {summary}")
    lines.append("")

    # ── In Progress ──
    lines.append(f"## In Progress ({len(in_progress_ents)})")
    lines.append("")
    if not in_progress_ents:
        lines.append("None.")
    else:
        for e in in_progress_ents:
            s = e.spec
            src = s.get("source") or ""
            fv = s.get("from_version") or ""
            tv = s.get("to_version") or ""
            started = _iso(s.get("started_at"))[:10]
            head = f"- **{e.id}** — {src} {fv} → {tv}".rstrip()
            if started:
                head += f" (started {started})"
            lines.append(head)
            summary = s.get("summary")
            if summary:
                lines.append(f"  {summary}")
    lines.append("")

    # ── Applied (table; preserves hand-written Notes column) ──
    lines.append(f"## Applied ({len(applied_ents)})")
    lines.append("")
    if not applied_ents:
        lines.append("None.")
    else:
        lines.append("| Date       | Migration                                | Notes |")
        lines.append("|------------|------------------------------------------|-------|")
        for e in applied_ents:
            carry = existing_notes.get(e.id)
            lines.append(_render_applied_row(e, carry))
    lines.append("")

    # ── Rejected (table; DEC-20260420_1353 refinement: separate section) ──
    lines.append(f"## Rejected ({len(rejected_ents)})")
    lines.append("")
    if not rejected_ents:
        lines.append("None.")
    else:
        lines.append("| Date       | Migration                                | Reason |")
        lines.append("|------------|------------------------------------------|--------|")
        for e in rejected_ents:
            s = e.spec
            applied_at = _iso(s.get("rejected_at") or s.get("applied_at"))
            date = applied_at[:10] if applied_at else ""
            header = (
                f"{e.id} — {s.get('source') or ''} "
                f"{s.get('from_version') or ''} → {s.get('to_version') or ''}"
            ).strip()
            reason = _truncate(s.get("rejected_reason") or "", 120)
            lines.append(f"| {date} | {header} | {reason} |")
    lines.append("")

    # ── Preserved CLI tail (aibox-owned) ──
    if cli_tail:
        lines.append(cli_tail.rstrip())
        lines.append("")

    index_path.parent.mkdir(parents=True, exist_ok=True)
    index_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    return index_path


# ---------------------------------------------------------------------------
# Shared write helpers
# ---------------------------------------------------------------------------

def _validate_and_transition(
    ent: entity.Entity,
    from_state: str,
    to_state: str,
    root: Path,
) -> dict[str, Any] | None:
    """Validate spec.state == from_state and the from→to edge against the
    migration state machine. Returns ``None`` on success, or an
    ``{"error": ...}`` dict on failure so the caller can early-return."""
    current = ent.spec.get("state")
    if current != from_state:
        return {
            "error": (
                f"Migration {ent.id!r} is in state {current!r}, "
                f"not {from_state!r}; cannot perform requested transition."
            )
        }
    try:
        state_machine.validate_transition("migration", from_state, to_state)
    except state_machine.StateMachineError as e:
        return {"error": str(e)}
    return None


def _commit_transition(
    ent: entity.Entity,
    to_state: str,
    root: Path,
    event_type: str,
    summary: str,
) -> Path:
    """Update spec.state, move the file, refresh INDEX.md, re-index, and
    log the side-effect event."""
    ent.spec["state"] = to_state
    new_path = _move_file(ent, to_state, root)

    try:
        db = index.open_db()
        try:
            index.upsert_entity(db, ent)
        finally:
            db.close()
    except Exception:
        # Index is a cache — failure to refresh must not block the write.
        pass

    _regenerate_index(root)

    log.log_side_effect(
        "Migration", ent.id, event_type, summary, root=root,
        actor=ent.id,
    )
    return new_path


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------

@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def list_migrations(state: str | None = None) -> list[dict]:
    """List Migration entities grouped by state.

    Walks ``context/migrations/{pending,in-progress,applied}/`` directly
    rather than relying on the SQLite index — Migrations whose file is
    freshly moved but not yet re-indexed still appear correctly.

    Parameters
    ----------
    state:
        Optional filter: ``pending``, ``in-progress``, ``applied``, or
        ``rejected``. When omitted, returns all Migrations.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    if state is not None and state not in _STATES:
        return [{"error": f"invalid state {state!r}; must be one of {list(_STATES)}"}]

    root = paths.find_project_root()
    migrations_root = _migrations_root(root)

    out: list[dict] = []
    # pending/, in-progress/ map 1:1 to state.
    for sub_state in ("pending", "in-progress"):
        if state is not None and state != sub_state:
            continue
        for p in _walk_dir(migrations_root / sub_state):
            try:
                ent = entity.load(p)
            except entity.EntityError:
                continue
            out.append(_summarize(ent))

    # applied/ holds BOTH applied and rejected; filter by spec.state.
    if state is None or state in ("applied", "rejected"):
        for p in _walk_dir(migrations_root / "applied"):
            try:
                ent = entity.load(p)
            except entity.EntityError:
                continue
            ent_state = ent.spec.get("state")
            if state is not None and ent_state != state:
                continue
            out.append(_summarize(ent))

    return out


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=True,
    destructiveHint=False,
    idempotentHint=True,
    openWorldHint=False,
))
def get_migration(id: str) -> dict:
    """Return the full Migration spec + body by ID.

    Accepts a full ID (e.g. ``MIG-20260419T080358``), a prefix, or a
    bare word-pair — falls back to the SQLite index via
    ``resolve_entity`` when a direct filename lookup misses.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_by_id(root, id)
    if ent is None:
        return {"error": f"Migration {id!r} not found"}
    return {
        "id": ent.id,
        "state": ent.spec.get("state"),
        "source": ent.spec.get("source"),
        "source_url": ent.spec.get("source_url"),
        "from_version": ent.spec.get("from_version"),
        "to_version": ent.spec.get("to_version"),
        "summary": ent.spec.get("summary"),
        "generated_by": ent.spec.get("generated_by"),
        "generated_at": _iso(ent.spec.get("generated_at")) or None,
        "started_at": _iso(ent.spec.get("started_at")) or None,
        "applied_at": _iso(ent.spec.get("applied_at")) or None,
        "applied_by": ent.spec.get("applied_by"),
        "rejected_reason": ent.spec.get("rejected_reason"),
        "affected_groups": ent.spec.get("affected_groups", []),
        "affected_files": ent.spec.get("affected_files", []),
        "plan": ent.spec.get("plan"),
        "progress_notes": ent.spec.get("progress_notes", []),
        "body": ent.body,
        "path": str(ent.path) if ent.path else None,
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def start_migration(id: str) -> dict:
    """Transition a Migration from pending to in-progress.

    Stamps ``spec.started_at`` with the current UTC time, moves the
    file from ``pending/`` to ``in-progress/``, refreshes
    ``INDEX.md``, and writes a ``migration.transitioned`` event.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_by_id(root, id)
    if ent is None:
        return {"error": f"Migration {id!r} not found"}

    err = _validate_and_transition(ent, "pending", "in-progress", root)
    if err:
        return err

    ent.spec["started_at"] = _now_iso()
    try:
        _commit_transition(
            ent, "in-progress", root,
            event_type="migration.transitioned",
            summary=f"Migration {ent.id!r}: pending → in-progress",
        )
    except ValueError as e:
        return {"error": str(e)}
    return {
        "ok": True,
        "id": ent.id,
        "from_state": "pending",
        "to_state": "in-progress",
        "path": str(ent.path),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def apply_migration(id: str, notes: str | None = None) -> dict:
    """Transition a Migration to applied.

    From ``in-progress``: validate the edge, stamp
    ``spec.applied_at``, optionally append ``notes`` to
    ``spec.progress_notes`` as ``{timestamp, actor: "mcp", note}``,
    move the file to ``applied/``, refresh ``INDEX.md``, and write a
    ``migration.applied`` event.

    From ``pending`` (implicit start): silently walks pending →
    in-progress → applied, validating each edge individually and
    emitting two LogEntries (``migration.transitioned`` then
    ``migration.applied``). The state machine is never loosened.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    root = paths.find_project_root()
    ent = _load_by_id(root, id)
    if ent is None:
        return {"error": f"Migration {id!r} not found"}

    original = ent.spec.get("state")
    if original == "pending":
        # Implicit start — walks the pending → in-progress edge first.
        err = _validate_and_transition(ent, "pending", "in-progress", root)
        if err:
            return err
        ent.spec["started_at"] = _now_iso()
        try:
            _commit_transition(
                ent, "in-progress", root,
                event_type="migration.transitioned",
                summary=f"Migration {ent.id!r}: pending → in-progress (implicit)",
            )
        except ValueError as e:
            return {"error": str(e)}
        # Re-load the (now-moved) entity so the next transition sees the
        # fresh path.
        ent = _load_by_id(root, ent.id)
        if ent is None:
            return {"error": f"Migration {id!r} disappeared after implicit start"}

    err = _validate_and_transition(ent, "in-progress", "applied", root)
    if err:
        return err

    ent.spec["applied_at"] = _now_iso()
    if notes:
        progress = list(ent.spec.get("progress_notes") or [])
        progress.append({
            "timestamp": _now_iso(),
            "actor": "mcp",
            "note": notes,
        })
        ent.spec["progress_notes"] = progress

    try:
        _commit_transition(
            ent, "applied", root,
            event_type="migration.applied",
            summary=f"Migration {ent.id!r}: in-progress → applied",
        )
    except ValueError as e:
        return {"error": str(e)}
    return {
        "ok": True,
        "id": ent.id,
        "from_state": original,
        "to_state": "applied",
        "path": str(ent.path),
    }


@server.tool(annotations=ToolAnnotations(
    readOnlyHint=False,
    destructiveHint=False,
    idempotentHint=False,
    openWorldHint=False,
))
def reject_migration(id: str, reason: str) -> dict:
    """Transition a Migration to rejected.

    Valid from ``pending`` or ``in-progress``. Stamps
    ``spec.rejected_reason`` (schema spelling — the issue's original
    ``rejection_reason`` was renamed for schema consistency), moves the
    file to ``applied/`` (rejected migrations park under applied/ per
    processkit convention), refreshes ``INDEX.md``, and writes a
    ``migration.rejected`` event.

    Prerequisite: call find_skill(task_description) or confirm you are
    already operating within a named processkit skill before using this
    tool. 1% rule: call route_task first; commit in the same turn —
    deferred writes are dropped.
    """
    if not (reason or "").strip():
        return {"error": "reason must be a non-empty string"}

    root = paths.find_project_root()
    ent = _load_by_id(root, id)
    if ent is None:
        return {"error": f"Migration {id!r} not found"}

    original = ent.spec.get("state")
    if original not in ("pending", "in-progress"):
        return {
            "error": (
                f"Migration {ent.id!r} is in state {original!r}; "
                f"cannot reject from that state."
            )
        }
    try:
        state_machine.validate_transition("migration", original, "rejected")
    except state_machine.StateMachineError as e:
        return {"error": str(e)}

    ent.spec["rejected_reason"] = reason
    ent.spec["rejected_at"] = _now_iso()
    try:
        _commit_transition(
            ent, "rejected", root,
            event_type="migration.rejected",
            summary=f"Migration {ent.id!r}: {original} → rejected ({_truncate(reason, 80)})",
        )
    except ValueError as e:
        return {"error": str(e)}
    return {
        "ok": True,
        "id": ent.id,
        "from_state": original,
        "to_state": "rejected",
        "path": str(ent.path),
    }


if __name__ == "__main__":
    server.run(transport="stdio")
