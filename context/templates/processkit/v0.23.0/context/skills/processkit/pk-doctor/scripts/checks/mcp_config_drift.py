"""mcp_config_drift check — detect stale MCP-config manifests.

Reads `context/.processkit-mcp-manifest.json` and compares against the
on-disk per-skill `mcp-config.json` files. Three findings matter:

- Manifest missing              → WARN mcp_config_drift.manifest-missing
- Manifest present, aggregate
  does not match current files  → WARN mcp_config_drift.manifest-stale
- Manifest fresh, but a derived
  project's `.mcp.json` is out
  of sync (skill servers missing
  from `mcpServers`)            → ERROR mcp_config_drift.harness-stale

The harness-stale check only runs when the repo looks like a derived
project (`aibox.lock` AND `.mcp.json` both present at the repo root). In
the processkit repo itself only the first two checks apply. See
DEC-20260423_2049-VastLake and projectious-work/aibox#54.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from .common import CheckResult


def _canonical_json(data: object) -> str:
    return json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def _sha256_of_file(path: Path) -> str:
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    return hashlib.sha256(_canonical_json(data).encode("utf-8")).hexdigest()


def _aggregate(entries: list[dict]) -> str:
    joined = "\n".join(e["sha256"] for e in entries)
    return hashlib.sha256(joined.encode("utf-8")).hexdigest()


def _collect_current(repo_root: Path) -> list[dict]:
    skills_root = repo_root / "context" / "skills"
    entries: list[dict] = []
    seen: set[str] = set()
    for cfg in sorted(skills_root.glob("*/*/mcp/mcp-config.json")):
        rel = cfg.relative_to(repo_root).as_posix()
        if rel in seen:
            continue
        seen.add(rel)
        entries.append({"path": rel, "sha256": _sha256_of_file(cfg)})
    for cfg in sorted(skills_root.glob("*/mcp/mcp-config.json")):
        rel = cfg.relative_to(repo_root).as_posix()
        if rel in seen:
            continue
        seen.add(rel)
        entries.append({"path": rel, "sha256": _sha256_of_file(cfg)})
    entries.sort(key=lambda e: e["path"])
    return entries


def _manifest_server_names(manifest: dict) -> list[str]:
    """Derive expected mcpServers keys from a manifest's per_skill paths.

    Convention used by aibox-merged .mcp.json: server name is
    `processkit-<slug>` where <slug> is the skill directory name under
    `context/skills/<category>/<slug>/mcp/mcp-config.json`.
    """
    names: list[str] = []
    for entry in manifest.get("per_skill") or []:
        path = entry.get("path", "")
        parts = Path(path).parts
        # .../skills/<category>/<slug>/mcp/mcp-config.json
        try:
            mcp_idx = parts.index("mcp")
        except ValueError:
            continue
        if mcp_idx >= 1:
            slug = parts[mcp_idx - 1]
            names.append(f"processkit-{slug}")
    return names


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    manifest_path = repo_root / "context" / ".processkit-mcp-manifest.json"

    if not manifest_path.is_file():
        return [CheckResult(
            severity="WARN",
            category="mcp_config_drift",
            id="mcp_config_drift.manifest-missing",
            message=(
                "context/.processkit-mcp-manifest.json not found; "
                "run `scripts/generate-mcp-manifest.py` before release."
            ),
            suggested_fix="uv run scripts/generate-mcp-manifest.py",
        )]

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        return [CheckResult(
            severity="ERROR",
            category="mcp_config_drift",
            id="mcp_config_drift.manifest-unreadable",
            message=f"could not parse {manifest_path.relative_to(repo_root)}: {e}",
        )]

    try:
        current = _collect_current(repo_root)
    except Exception as e:
        return [CheckResult(
            severity="ERROR",
            category="mcp_config_drift",
            id="mcp_config_drift.collect-failed",
            message=f"could not walk mcp-config.json files: {e}",
        )]

    current_aggregate = _aggregate(current)
    manifest_aggregate = manifest.get("aggregate_sha256")

    if current_aggregate != manifest_aggregate:
        manifest_paths = {e["path"]: e["sha256"] for e in manifest.get("per_skill") or []}
        current_paths = {e["path"]: e["sha256"] for e in current}
        changed = 0
        for p, sha in current_paths.items():
            if manifest_paths.get(p) != sha:
                changed += 1
        for p in manifest_paths:
            if p not in current_paths:
                changed += 1
        return [CheckResult(
            severity="WARN",
            category="mcp_config_drift",
            id="mcp_config_drift.manifest-stale",
            message=(
                f"{changed} skill config(s) changed since manifest was "
                f"generated; run `scripts/generate-mcp-manifest.py`"
            ),
            suggested_fix="uv run scripts/generate-mcp-manifest.py",
        )]

    # Derived-project harness-stale check.
    aibox_lock = repo_root / "aibox.lock"
    mcp_json = repo_root / ".mcp.json"
    if aibox_lock.is_file() and mcp_json.is_file():
        try:
            merged = json.loads(mcp_json.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as e:
            return [CheckResult(
                severity="ERROR",
                category="mcp_config_drift",
                id="mcp_config_drift.mcp-json-unreadable",
                message=f"could not parse {mcp_json.relative_to(repo_root)}: {e}",
            )]
        servers = (merged or {}).get("mcpServers") or {}
        expected = _manifest_server_names(manifest)
        missing = [name for name in expected if name not in servers]
        if missing:
            preview = ", ".join(missing[:5])
            if len(missing) > 5:
                preview += f" (+{len(missing) - 5} more)"
            return [CheckResult(
                severity="ERROR",
                category="mcp_config_drift",
                id="mcp_config_drift.harness-stale",
                message=(
                    f"{len(missing)} processkit server(s) missing from "
                    f".mcp.json ({preview}); run `aibox sync` (or "
                    f"`aibox sync --force`) to re-merge .mcp.json."
                ),
                suggested_fix="aibox sync",
            )]

    return [CheckResult(
        severity="INFO",
        category="mcp_config_drift",
        id="mcp_config_drift.in-sync",
        message="MCP config manifest in sync.",
    )]
