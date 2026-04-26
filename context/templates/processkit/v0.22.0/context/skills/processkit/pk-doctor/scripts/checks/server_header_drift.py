"""server_header_drift check — flag stale PEP 723 dep headers.

Per DEC-20260424_0127-QuickPine (SharpBrook split). Walks every
``context/skills/processkit/*/mcp/server.py``, extracts each script's
PEP 723 inline metadata block (``# /// script`` ... ``# ///``), and
compares its sha256 to the per-server-header manifest entry baked at
release time by ``scripts/generate-mcp-manifest.py``.

Drift means the user edited a server.py header (typically to add a
dependency) without regenerating the manifest. The harness running the
MCP servers will not pick up the new deps until it is restarted, so the
fix is twofold: regenerate the manifest, then restart the harness.

Findings:

- manifest missing                 → WARN server_header_drift.manifest-missing
- manifest lacks per_server_header → WARN server_header_drift.field-missing
- live header hash differs from
  manifest entry                   → WARN server_header_drift.dep-changed
- server.py present but no manifest
  entry, or vice versa             → WARN server_header_drift.coverage-mismatch
- everything aligned               → INFO server_header_drift.in-sync

Detect-only — no auto-fix (a full harness restart is user-initiated).
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from .common import CheckResult


_MANIFEST_REL = Path("context/.processkit-mcp-manifest.json")
_HEADER_OPEN = "# /// script"
_HEADER_CLOSE = "# ///"


def _extract_header(path: Path) -> str | None:
    """Return the raw PEP 723 block (inclusive of fences) or None."""
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    lines = text.splitlines()
    in_block = False
    block: list[str] = []
    for line in lines:
        stripped = line.strip()
        if not in_block:
            if stripped == _HEADER_OPEN:
                in_block = True
                block.append(line)
            continue
        block.append(line)
        if stripped == _HEADER_CLOSE:
            return "\n".join(block) + "\n"
    return None


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _collect_current(repo_root: Path) -> dict[str, str]:
    """Return {posix_path: sha256} for every server.py with a PEP 723 block."""
    skills_root = repo_root / "context" / "skills"
    found: dict[str, str] = {}
    for server in sorted(skills_root.glob("*/*/mcp/server.py")):
        rel = server.relative_to(repo_root).as_posix()
        header = _extract_header(server)
        if header is None:
            continue
        found[rel] = _sha256(header)
    return found


def _slug_for(path_str: str) -> str:
    parts = Path(path_str).parts
    try:
        mcp_idx = parts.index("mcp")
    except ValueError:
        return path_str
    if mcp_idx >= 1:
        return parts[mcp_idx - 1]
    return path_str


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    manifest_path = repo_root / _MANIFEST_REL

    if not manifest_path.is_file():
        return [CheckResult(
            severity="WARN",
            category="server_header_drift",
            id="server_header_drift.manifest-missing",
            message=(
                f"{_MANIFEST_REL.as_posix()} not found; "
                "run `scripts/generate-mcp-manifest.py` before release."
            ),
            suggested_fix="uv run scripts/generate-mcp-manifest.py",
        )]

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        return [CheckResult(
            severity="ERROR",
            category="server_header_drift",
            id="server_header_drift.manifest-unreadable",
            message=f"could not parse {_MANIFEST_REL.as_posix()}: {e}",
        )]

    manifest_entries = manifest.get("per_server_header")
    if manifest_entries is None:
        return [CheckResult(
            severity="WARN",
            category="server_header_drift",
            id="server_header_drift.field-missing",
            message=(
                "manifest has no per_server_header field; regenerate with "
                "`scripts/generate-mcp-manifest.py` to record current headers."
            ),
            suggested_fix="uv run scripts/generate-mcp-manifest.py",
        )]

    manifest_map = {e["path"]: e["sha256"] for e in manifest_entries}
    current = _collect_current(repo_root)

    results: list[CheckResult] = []

    drifted = sorted(
        p for p, sha in current.items()
        if p in manifest_map and manifest_map[p] != sha
    )
    if drifted:
        slugs = sorted({_slug_for(p) for p in drifted})
        preview = ", ".join(slugs[:5])
        if len(slugs) > 5:
            preview += f" (+{len(slugs) - 5} more)"
        results.append(CheckResult(
            severity="WARN",
            category="server_header_drift",
            id="server_header_drift.dep-changed",
            message=(
                f"PEP 723 dep header changed for {len(drifted)} server(s) "
                f"({preview}); restart the harness to pick up dep changes "
                "and regenerate the manifest."
            ),
            suggested_fix="uv run scripts/generate-mcp-manifest.py",
        ))

    only_live = sorted(p for p in current if p not in manifest_map)
    only_manifest = sorted(p for p in manifest_map if p not in current)
    if only_live or only_manifest:
        bits: list[str] = []
        if only_live:
            slugs = sorted({_slug_for(p) for p in only_live})
            preview = ", ".join(slugs[:5])
            if len(slugs) > 5:
                preview += f" (+{len(slugs) - 5} more)"
            bits.append(f"new server(s) not in manifest: {preview}")
        if only_manifest:
            slugs = sorted({_slug_for(p) for p in only_manifest})
            preview = ", ".join(slugs[:5])
            if len(slugs) > 5:
                preview += f" (+{len(slugs) - 5} more)"
            bits.append(f"manifest entries with no live server: {preview}")
        results.append(CheckResult(
            severity="WARN",
            category="server_header_drift",
            id="server_header_drift.coverage-mismatch",
            message=(
                f"server.py / manifest coverage mismatch — {'; '.join(bits)}; "
                "regenerate the manifest."
            ),
            suggested_fix="uv run scripts/generate-mcp-manifest.py",
        ))

    if not results:
        results.append(CheckResult(
            severity="INFO",
            category="server_header_drift",
            id="server_header_drift.in-sync",
            message=(
                f"{len(current)} MCP server.py header(s) match manifest."
            ),
        ))

    return results
