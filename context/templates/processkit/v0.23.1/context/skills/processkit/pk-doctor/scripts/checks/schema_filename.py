"""schema_filename check.

For every `context/<kind>/**/*.md`:

- Load the matching schema from src/context/schemas/<kind>.yaml (LIVE source,
  NOT the template mirror under context/templates/).
- Parse the file's YAML frontmatter.
- Validate spec against schema.spec_schema.
- Confirm the filename stem equals metadata.id.
- Confirm any date encoded in the filename matches metadata.created.

Phase 1 is WARN-only for rename suggestions — no auto-fix. Auto-rename +
reindex is deferred to Phase 2.

Known parser quirks:
- YAML datetime fields vs JSON-Schema `string` format=date-time coercion
  mismatches surface as WARN schema.datetime-coercion (owner policy).
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterator

import yaml  # via uv inline deps in doctor.py

from .common import CheckResult


# Maps schema filename stem → directory under context/ where that kind lives.
# Only kinds with a `default_directory` under context/<dir>/ get walked.
KIND_TO_DIR = {
    "workitem": "workitems",
    "decisionrecord": "decisions",
    "artifact": "artifacts",
    "binding": "bindings",
    "actor": "actors",
    "role": "roles",
    "discussion": "discussions",
    "gate": "gates",
    "migration": "migrations",
    "note": "notes",
    "process": "processes",
    "scope": "scopes",
    "logentry": "logs",
}

FILENAME_DATE_RE = re.compile(r"(\d{8})(?:[_T](\d{4,6}))?")
# aibox-CLI migration docs (e.g. 20260410_1523_0.17.6-to-0.17.9.md) live in
# context/migrations/ but are NOT processkit Migration entities — they are
# CLI upgrade notes. Exempt them from schema validation.
CLI_MIGRATION_RE = re.compile(r"^\d{8}_\d{4}_\d+\.\d+\.\d+-to-\d+\.\d+\.\d+\.md$")

# Actor ID pattern enforcement (DEC-20260421_2036-SoundIvy-two-class-actor-ids)
_ACTOR_IDENTITY_RE = re.compile(
    r"^ACTOR-\d{8}_\d{4}-[A-Z][a-z]+[A-Z][a-z]+-[a-z0-9-]+$"
)
_ACTOR_ROLE_RE = re.compile(r"^ACTOR-[a-z][a-z0-9-]*$")


def _check_actor_id_pattern(
    entity_id: str, allowed_role_ids: list[str]
) -> str | None:
    """Return None if valid, or an error reason string if invalid."""
    if _ACTOR_IDENTITY_RE.match(entity_id):
        return None  # identity class — OK
    if _ACTOR_ROLE_RE.match(entity_id):
        slug = entity_id[len("ACTOR-"):]
        if slug in allowed_role_ids:
            return None  # role class in allowlist — OK
        return (
            f"role-actor ID '{slug}' not in x-allowed-role-ids allowlist"
        )
    return (
        "does not match identity class "
        "(ACTOR-<datetime>-<WordPair>-<slug>) or "
        "role class (ACTOR-<slug>)"
    )


def _resolve_schemas_dir(repo_root: Path) -> Path | None:
    """Return the schemas directory for ``repo_root`` or ``None``.

    Checks two layouts in order:
      1. ``<repo_root>/src/context/schemas/`` — processkit dogfood (the
         processkit repo itself; schemas live in src/ because they ship
         to consumers).
      2. ``<repo_root>/context/schemas/`` — derived projects (aibox-
         installed; schemas were copied directly into context/).

    A derived-project install never has a ``src/`` tree, so the first
    candidate misses and the function falls through to the second.
    Returning ``None`` is the cue for the caller to surface a single
    "no schemas dir found" WARN rather than silently walking 0 files.
    """
    for candidate in (
        repo_root / "src" / "context" / "schemas",
        repo_root / "context" / "schemas",
    ):
        if candidate.is_dir():
            return candidate
    return None


def _load_schema(schemas_dir: Path, kind: str) -> dict | None:
    path = schemas_dir / f"{kind}.yaml"
    if not path.is_file():
        return None
    try:
        with path.open("r", encoding="utf-8") as f:
            return yaml.safe_load(f)
    except yaml.YAMLError:
        return None


def _parse_frontmatter(path: Path) -> tuple[dict | None, str | None]:
    """Return (frontmatter_dict, parse_error_msg_or_None)."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as e:
        return None, f"unreadable: {e}"
    if not text.startswith("---"):
        return None, "no YAML frontmatter"
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None, "unterminated YAML frontmatter"
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError as e:
        return None, f"YAML parse error: {e}"
    if not isinstance(data, dict):
        return None, "frontmatter is not a mapping"
    return data, None


def _iter_entity_files(ctx_root: Path, kind_dir: str, since_files: set[Path] | None) -> Iterator[Path]:
    root = ctx_root / kind_dir
    if not root.is_dir():
        return
    for p in root.rglob("*.md"):
        if p.name.startswith("INDEX"):
            continue
        if CLI_MIGRATION_RE.match(p.name):
            # aibox-CLI upgrade doc, not a processkit entity — skip.
            continue
        if since_files is not None and p not in since_files:
            continue
        yield p


def _validate_against_schema(data: dict, schema: dict) -> list[str]:
    """Return list of JSON-schema error messages. Empty → valid.

    Uses jsonschema if available (via uv inline deps). If missing, returns
    a single synthetic error so the operator knows validation was skipped.
    """
    spec = data.get("spec")
    schema_spec = schema.get("spec", {}).get("spec_schema")
    if not schema_spec or not isinstance(spec, dict):
        return []  # nothing enforceable — surface other checks still fire
    try:
        import jsonschema
        from jsonschema import Draft202012Validator
    except ImportError:
        return ["jsonschema not installed (run via uv)"]

    # Coerce YAML datetime objects to ISO strings for JSON-Schema.
    # Record a marker so callers can downgrade the severity for coercion
    # failures per owner policy.
    def _coerce(o):
        if isinstance(o, dict):
            return {k: _coerce(v) for k, v in o.items()}
        if isinstance(o, list):
            return [_coerce(v) for v in o]
        # Handle datetime / date objects without importing datetime globally
        if hasattr(o, "isoformat"):
            return o.isoformat()
        return o

    coerced = _coerce(spec)
    validator = Draft202012Validator(schema_spec)
    errs = []
    for e in validator.iter_errors(coerced):
        path = ".".join(str(x) for x in e.absolute_path) or "<root>"
        errs.append(f"{path}: {e.message}")
    return errs


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    since_files: set[Path] | None = ctx.get("since_files")
    results: list[CheckResult] = []

    ctx_root = repo_root / "context"
    schemas_dir = _resolve_schemas_dir(repo_root)
    if schemas_dir is None:
        # No schemas anywhere — surface this loudly instead of silently
        # walking 0 files, which masked entity-hygiene problems in
        # derived projects (HappyReef).  Walk continues with no schema
        # validation but still catches structural issues (filename/id,
        # actor-id pattern) below.
        results.append(CheckResult(
            severity="WARN",
            category="schema_filename",
            id="schema.no-schemas-dir",
            message=(
                "no schemas directory found at "
                f"{repo_root / 'src' / 'context' / 'schemas'} or "
                f"{repo_root / 'context' / 'schemas'} — schema validation "
                "is disabled for this run; only filename/id and actor-id "
                "checks fire"
            ),
        ))

    walked_count = 0
    for kind, kind_dir in KIND_TO_DIR.items():
        schema = _load_schema(schemas_dir, kind) if schemas_dir else None
        # Even without a schema we still want to walk the entity files
        # so the filename/id, filename-date, and actor-id checks fire
        # (those don't need spec_schema). Skip only the schema-validation
        # block when schema is None.
        if schemas_dir is None and not (repo_root / "context" / kind_dir).is_dir():
            # No schemas AND no entity dir for this kind — skip silently.
            continue
        # Cache allowed role IDs per kind (actor only, cheap to compute).
        allowed_role_ids: list[str] = (
            schema.get("x-allowed-role-ids", []) if schema else []
        )
        for path in _iter_entity_files(ctx_root, kind_dir, since_files):
            walked_count += 1
            fm, parse_err = _parse_frontmatter(path)
            rel = path.relative_to(repo_root)
            if parse_err:
                results.append(CheckResult(
                    severity="ERROR",
                    category="schema_filename",
                    id="schema.parse-error",
                    message=f"{rel}: {parse_err}",
                    entity_ref=str(rel),
                ))
                continue

            md = fm.get("metadata", {}) if isinstance(fm, dict) else {}
            meta_id = md.get("id") if isinstance(md, dict) else None

            # --- filename stem vs metadata.id -----------------------------
            stem = path.stem
            if meta_id and stem != meta_id:
                results.append(CheckResult(
                    severity="WARN",
                    category="schema_filename",
                    id="schema.filename-id-mismatch",
                    message=(
                        f"{rel}: filename stem '{stem}' != metadata.id "
                        f"'{meta_id}' (Phase 1: rename deferred)"
                    ),
                    entity_ref=str(rel),
                    fixable=False,
                    suggested_fix=f"rename to {meta_id}.md (Phase 2)",
                ))

            # --- Actor: two-class ID pattern check -------------------------
            if kind == "actor" and meta_id:
                reason = _check_actor_id_pattern(meta_id, allowed_role_ids)
                if reason is not None:
                    results.append(CheckResult(
                        severity="ERROR",
                        category="schema_filename",
                        id="schema.filename.invalid_actor_id_pattern",
                        message=(
                            f"{rel}: metadata.id '{meta_id}': {reason}"
                        ),
                        entity_ref=str(rel),
                    ))

            # --- filename date vs metadata.created ------------------------
            m = FILENAME_DATE_RE.search(stem)
            created = md.get("created") if isinstance(md, dict) else None
            if m and created:
                fn_date = m.group(1)  # YYYYMMDD
                created_str = created.isoformat() if hasattr(created, "isoformat") else str(created)
                if fn_date not in created_str.replace("-", ""):
                    results.append(CheckResult(
                        severity="WARN",
                        category="schema_filename",
                        id="schema.filename-date-mismatch",
                        message=(
                            f"{rel}: filename date '{fn_date}' not in "
                            f"metadata.created '{created_str}'"
                        ),
                        entity_ref=str(rel),
                    ))

            # --- schema validation ----------------------------------------
            errs = _validate_against_schema(fm, schema) if schema else []
            for err in errs:
                # Owner policy: datetime-coercion errors are WARN (parser quirk),
                # all other schema errors are ERROR.
                if "date-time" in err or "datetime" in err.lower() or "is not of type 'string'" in err and "created" in err:
                    results.append(CheckResult(
                        severity="WARN",
                        category="schema_filename",
                        id="schema.datetime-coercion",
                        message=f"{rel}: {err}",
                        entity_ref=str(rel),
                    ))
                else:
                    results.append(CheckResult(
                        severity="ERROR",
                        category="schema_filename",
                        id="schema.invalid",
                        message=f"{rel}: {err}",
                        entity_ref=str(rel),
                    ))

    results.append(CheckResult(
        severity="INFO",
        category="schema_filename",
        id="schema.walked",
        message=f"walked {walked_count} entity file(s) across {len(KIND_TO_DIR)} kinds",
    ))
    return results


# --- narrow autofix (DEC-BrightHawk / SnappyBird replacement) --------------
#
# Scope: exactly ONE failure pattern — a LogEntry missing the required
# `actor` field (CalmAnt-class, pre-TeamMember schema-drift window).
# The fix inserts `actor: system` into spec. Any other schema.invalid
# pattern is left untouched — extend case-by-case when a real need
# arises, per DEC-20260424_0128-BrightHawk.

_MISSING_ACTOR_MARKER = "'actor' is a required property"


def run_fix(ctx, results: list[CheckResult]) -> list[dict]:
    """Apply narrow, known-safe patches for specific schema ERRORs.

    Currently handles only: LogEntry missing required field ``actor``
    → insert ``actor: system`` into spec, validate post-patch, write
    back on success.

    Returns a list of fix records with ``status`` in
    ``applied | skipped | rolled-back``.  Called by doctor.py only
    when ``--fix=schema_filename`` (or ``--fix-all``) is passed.
    """
    repo_root: Path = ctx["repo_root"]
    schemas_dir = repo_root / "src" / "context" / "schemas"
    log_schema = _load_schema(schemas_dir, "logentry")
    fixes: list[dict] = []

    for r in results:
        if r.id != "schema.invalid":
            continue
        if _MISSING_ACTOR_MARKER not in (r.message or ""):
            continue
        if not r.entity_ref:
            continue
        rel = Path(r.entity_ref)
        if rel.parts[:2] != ("context", "logs"):
            continue  # only LogEntries
        abs_path = repo_root / rel
        if not abs_path.is_file():
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "file not found",
            })
            continue
        if log_schema is None:
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "no logentry.yaml schema available for validation",
            })
            continue

        fm, perr = _parse_frontmatter(abs_path)
        if fm is None:
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": f"frontmatter parse failed: {perr}",
            })
            continue

        spec = fm.get("spec") if isinstance(fm, dict) else None
        if not isinstance(spec, dict):
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "no spec block in frontmatter",
            })
            continue
        if "actor" in spec:
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "actor already present",
            })
            continue

        # Patch in memory and re-validate against the schema.
        spec["actor"] = "system"
        errs_after = _validate_against_schema(fm, log_schema)
        if errs_after:
            # Post-patch STILL invalid → don't write; report.
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "rolled-back",
                "reason": f"post-patch still invalid: {errs_after[0]}",
            })
            continue

        # Rewrite frontmatter, preserving the original body verbatim.
        original_text = abs_path.read_text(encoding="utf-8")
        parts = original_text.split("---", 2)
        if len(parts) < 3:
            fixes.append({
                "category": "schema_filename",
                "entity": str(rel),
                "status": "skipped",
                "reason": "unterminated frontmatter",
            })
            continue
        body = parts[2]
        new_fm_text = yaml.safe_dump(fm, sort_keys=False, allow_unicode=True)
        new_text = f"---\n{new_fm_text}---{body}"
        abs_path.write_text(new_text, encoding="utf-8")

        fixes.append({
            "category": "schema_filename",
            "entity": str(rel),
            "status": "applied",
            "patch": "spec.actor = 'system'",
            "rationale": (
                "missing required 'actor' field (CalmAnt-class, "
                "DEC-BrightHawk)"
            ),
        })

    return fixes
