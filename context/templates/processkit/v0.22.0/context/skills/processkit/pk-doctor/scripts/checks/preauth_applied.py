"""preauth_applied check — flag missing aibox-merged preauth in settings.

processkit ships a preauth spec at
`context/skills/processkit/skill-gate/assets/preauth.json` listing the
`permissions.allow[]` patterns and `enabledMcpjsonServers[]` entries
that aibox sync should merge into a derived project's
`.claude/settings.json`. Until aibox#55 ships and a sync run picks the
new spec up, derived projects are re-prompted for every processkit MCP
tool after each container rebuild (see
`BACK-20260425_1316-WildGrove`).

This check compares the live `.claude/settings.json` against the spec
and reports the gap. It is detect-only (no fix) — the actual merge
lives in aibox.

Findings:

- preauth.json missing
  → ERROR preauth_applied.spec-missing
- `.claude/settings.json` missing or unreadable
  → SKIP (INFO with `not-applicable`) — project does not use Claude Code
- spec entries missing from `permissions.allow[]` or
  `enabledMcpjsonServers[]`
  → WARN preauth_applied.permissions-missing /
        preauth_applied.servers-missing
- spec drift vs the MCP manifest
  (`context/.processkit-mcp-manifest.json`)
  → WARN preauth_applied.spec-drift
- everything aligned
  → INFO preauth_applied.in-sync
"""

from __future__ import annotations

import json
from pathlib import Path

from .common import CheckResult


_SPEC_REL = Path("context/skills/processkit/skill-gate/assets/preauth.json")
_SETTINGS_REL = Path(".claude/settings.json")
_MANIFEST_REL = Path("context/.processkit-mcp-manifest.json")


def _load_json(path: Path) -> dict | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def _expected_servers_from_manifest(manifest: dict) -> set[str]:
    names: set[str] = set()
    for entry in manifest.get("per_skill") or []:
        path = entry.get("path", "")
        parts = Path(path).parts
        try:
            mcp_idx = parts.index("mcp")
        except ValueError:
            continue
        if mcp_idx >= 1:
            names.add(f"processkit-{parts[mcp_idx - 1]}")
    return names


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]

    spec_path = repo_root / _SPEC_REL
    if not spec_path.is_file():
        return [CheckResult(
            severity="ERROR",
            category="preauth_applied",
            id="preauth_applied.spec-missing",
            message=(
                f"{_SPEC_REL.as_posix()} not found; processkit ships this "
                "spec for aibox to merge into .claude/settings.json"
            ),
        )]

    spec = _load_json(spec_path)
    if spec is None:
        return [CheckResult(
            severity="ERROR",
            category="preauth_applied",
            id="preauth_applied.spec-unreadable",
            message=f"could not parse {_SPEC_REL.as_posix()}",
        )]

    spec_perms = set((spec.get("permissions") or {}).get("allow") or [])
    spec_servers = set(spec.get("enabledMcpjsonServers") or [])

    results: list[CheckResult] = []

    # Drift: spec server list vs manifest-derived list. Warns processkit
    # maintainers when the manifest moves but preauth.json hasn't been
    # regenerated.
    manifest_path = repo_root / _MANIFEST_REL
    if manifest_path.is_file():
        manifest = _load_json(manifest_path)
        if manifest is not None:
            expected = _expected_servers_from_manifest(manifest)
            if expected and expected != spec_servers:
                missing = sorted(expected - spec_servers)
                extra = sorted(spec_servers - expected)
                bits: list[str] = []
                if missing:
                    bits.append(f"missing from spec: {', '.join(missing[:5])}"
                                + (f" (+{len(missing) - 5} more)"
                                   if len(missing) > 5 else ""))
                if extra:
                    bits.append(f"extra in spec: {', '.join(extra[:5])}"
                                + (f" (+{len(extra) - 5} more)"
                                   if len(extra) > 5 else ""))
                results.append(CheckResult(
                    severity="WARN",
                    category="preauth_applied",
                    id="preauth_applied.spec-drift",
                    message=(
                        "preauth.json enabledMcpjsonServers does not match "
                        f"manifest-derived names; {'; '.join(bits)}"
                    ),
                    suggested_fix=(
                        "regenerate preauth.json from "
                        ".processkit-mcp-manifest.json"
                    ),
                ))

    settings_path = repo_root / _SETTINGS_REL
    if not settings_path.is_file():
        results.append(CheckResult(
            severity="INFO",
            category="preauth_applied",
            id="preauth_applied.not-applicable",
            message=(
                ".claude/settings.json not present — Claude Code preauth "
                "check skipped (project may use a different harness)."
            ),
        ))
        return results

    settings = _load_json(settings_path)
    if settings is None:
        results.append(CheckResult(
            severity="ERROR",
            category="preauth_applied",
            id="preauth_applied.settings-unreadable",
            message=f"could not parse {_SETTINGS_REL.as_posix()}",
        ))
        return results

    live_perms = set(((settings.get("permissions") or {}).get("allow")) or [])
    live_servers = set(settings.get("enabledMcpjsonServers") or [])

    missing_perms = sorted(spec_perms - live_perms)
    if missing_perms:
        preview = ", ".join(missing_perms[:5])
        if len(missing_perms) > 5:
            preview += f" (+{len(missing_perms) - 5} more)"
        results.append(CheckResult(
            severity="WARN",
            category="preauth_applied",
            id="preauth_applied.permissions-missing",
            message=(
                f"{len(missing_perms)} preauth permission pattern(s) not in "
                f".claude/settings.json permissions.allow ({preview}); "
                "merge gated by aibox#55"
            ),
            suggested_fix="aibox sync (once aibox#55 ships)",
        ))

    missing_servers = sorted(spec_servers - live_servers)
    if missing_servers:
        preview = ", ".join(missing_servers[:5])
        if len(missing_servers) > 5:
            preview += f" (+{len(missing_servers) - 5} more)"
        results.append(CheckResult(
            severity="WARN",
            category="preauth_applied",
            id="preauth_applied.servers-missing",
            message=(
                f"{len(missing_servers)} preauth server(s) not in "
                f".claude/settings.json enabledMcpjsonServers ({preview}); "
                "merge gated by aibox#55"
            ),
            suggested_fix="aibox sync (once aibox#55 ships)",
        ))

    if not results:
        results.append(CheckResult(
            severity="INFO",
            category="preauth_applied",
            id="preauth_applied.in-sync",
            message=(
                "preauth.json fully merged into .claude/settings.json "
                f"({len(spec_perms)} permission patterns, "
                f"{len(spec_servers)} servers)."
            ),
        ))

    return results
