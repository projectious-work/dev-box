#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
# ]
# ///
"""release_audit.py — detect-only pre-release validation for a processkit repo.

Walks the live context/ tree and validates:

1. entity_files   — frontmatter, apiVersion, kind, metadata.id vs filename
2. skill_structure — SKILL.md frontmatter fields + four required sections
3. mcp_annotations — every @server.tool() has ToolAnnotations with all hint keys
4. cross_references — uses: entries resolve to existing SKILL.md files

Outputs a single human-readable Markdown report on stdout.
Exit 0 if no ERRORs; exit 1 otherwise.

Usage:
    release_audit.py [--repo-root=PATH]

Provenance: WorkItem BACK-20260409_1830-TidyGrove.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal, Optional

import yaml

AUDIT_VERSION = "1.0.0"

# ---------------------------------------------------------------------------
# Registered entity kinds and their context/ directories.
# ---------------------------------------------------------------------------

ENTITY_DIRS: dict[str, str] = {
    "workitems": "WorkItem",
    "decisions": "DecisionRecord",
    "logs": "LogEntry",
    "artifacts": "Artifact",
    "actors": "Actor",
    "bindings": "Binding",
    "scopes": "Scope",
    "gates": "Gate",
    "roles": "Role",
    "migrations": "Migration",
    "team-members": "TeamMember",
    "notes": "Note",
    "discussions": "Discussion",
}

REGISTERED_KINDS: frozenset[str] = frozenset(ENTITY_DIRS.values())

EXPECTED_API_VERSION = "processkit.projectious.work/v1"

# ---------------------------------------------------------------------------
# Required SKILL.md frontmatter fields (dot-path notation).
# ---------------------------------------------------------------------------

SKILL_REQUIRED_FIELDS: list[str] = [
    "name",
    "description",
    "metadata.processkit.id",
    "metadata.processkit.version",
    "metadata.processkit.category",
    "metadata.processkit.layer",
]

SKILL_REQUIRED_SECTIONS: list[str] = [
    "## Intro",
    "## Overview",
    "## Gotchas",
    "## Full reference",
]

# Required ToolAnnotations hint keys that must appear inside the
# annotations=ToolAnnotations(...) call of each @server.tool() decoration.
MCP_ANNOTATION_KEYS: list[str] = [
    "readOnlyHint",
    "destructiveHint",
    "idempotentHint",
    "openWorldHint",
]

# ---------------------------------------------------------------------------
# Finding dataclass
# ---------------------------------------------------------------------------

Severity = Literal["ERROR", "WARN", "INFO"]


@dataclass
class Finding:
    severity: Severity
    category: str
    id: str
    message: str
    entity_ref: Optional[str] = None
    extra: dict = field(default_factory=dict)


def tally(findings: list[Finding]) -> dict[str, int]:
    out = {"ERROR": 0, "WARN": 0, "INFO": 0}
    for f in findings:
        out[f.severity] = out.get(f.severity, 0) + 1
    return out


# ---------------------------------------------------------------------------
# YAML frontmatter helpers
# ---------------------------------------------------------------------------

def _split_frontmatter(text: str) -> tuple[Optional[str], str]:
    """Return (frontmatter_yaml, body) or (None, full_text)."""
    if not text.startswith("---"):
        return None, text
    # Find the closing ---
    rest = text[3:]
    # Skip optional newline after opening ---
    if rest.startswith("\n"):
        rest = rest[1:]
    idx = rest.find("\n---")
    if idx == -1:
        return None, text
    fm = rest[:idx]
    body = rest[idx + 4:]  # skip "\n---"
    return fm, body


def _get_nested(data: dict, dotpath: str):
    """Traverse dotpath like 'metadata.processkit.id' into nested dicts."""
    parts = dotpath.split(".")
    cur = data
    for part in parts:
        if not isinstance(cur, dict) or part not in cur:
            return None
        cur = cur[part]
    return cur


# ---------------------------------------------------------------------------
# Check 1: entity_files
# ---------------------------------------------------------------------------

def run_entity_files(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    context = repo_root / "context"

    for dir_name, expected_kind in ENTITY_DIRS.items():
        entity_dir = context / dir_name
        if not entity_dir.is_dir():
            findings.append(Finding(
                severity="WARN",
                category="entity_files",
                id="entity.missing-dir",
                message=f"Expected entity directory does not exist: context/{dir_name}/",
            ))
            continue

        md_files = list(entity_dir.rglob("*.md"))
        if not md_files:
            findings.append(Finding(
                severity="INFO",
                category="entity_files",
                id="entity.empty-dir",
                message=f"context/{dir_name}/ contains no .md files",
            ))
            continue

        for md_path in sorted(md_files):
            rel = md_path.relative_to(repo_root)
            stem = md_path.stem
            ref = str(rel)

            try:
                text = md_path.read_text(encoding="utf-8")
            except OSError as exc:
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.unreadable",
                    message=f"Cannot read file: {exc}",
                    entity_ref=ref,
                ))
                continue

            fm_yaml, _body = _split_frontmatter(text)
            if fm_yaml is None:
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.no-frontmatter",
                    message="No YAML frontmatter found (missing --- delimiters)",
                    entity_ref=ref,
                ))
                continue

            try:
                data = yaml.safe_load(fm_yaml)
            except yaml.YAMLError as exc:
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.bad-frontmatter",
                    message=f"Frontmatter YAML parse error: {exc}",
                    entity_ref=ref,
                ))
                continue

            if not isinstance(data, dict):
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.bad-frontmatter",
                    message="Frontmatter parsed to non-dict",
                    entity_ref=ref,
                ))
                continue

            # apiVersion check
            api_ver = data.get("apiVersion")
            if api_ver != EXPECTED_API_VERSION:
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.wrong-api-version",
                    message=f"apiVersion={api_ver!r} expected {EXPECTED_API_VERSION!r}",
                    entity_ref=ref,
                ))

            # kind check
            kind = data.get("kind")
            if not kind:
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.missing-kind",
                    message="'kind' field is missing",
                    entity_ref=ref,
                ))
            elif kind not in REGISTERED_KINDS:
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.unknown-kind",
                    message=f"kind={kind!r} is not a registered kind",
                    entity_ref=ref,
                ))

            # metadata.id check
            metadata = data.get("metadata")
            if not isinstance(metadata, dict):
                findings.append(Finding(
                    severity="ERROR",
                    category="entity_files",
                    id="entity.missing-metadata-id",
                    message="metadata block missing or not a dict",
                    entity_ref=ref,
                ))
            else:
                entity_id = metadata.get("id")
                if not entity_id:
                    findings.append(Finding(
                        severity="ERROR",
                        category="entity_files",
                        id="entity.missing-metadata-id",
                        message="metadata.id is missing",
                        entity_ref=ref,
                    ))
                elif str(entity_id) != stem:
                    findings.append(Finding(
                        severity="ERROR",
                        category="entity_files",
                        id="entity.id-filename-mismatch",
                        message=f"metadata.id={entity_id!r} does not match filename stem {stem!r}",
                        entity_ref=ref,
                    ))
                else:
                    findings.append(Finding(
                        severity="INFO",
                        category="entity_files",
                        id="entity.ok",
                        message=f"{stem} ({dir_name}) — OK",
                        entity_ref=ref,
                    ))

    return findings


# ---------------------------------------------------------------------------
# Check 2: skill_structure
# ---------------------------------------------------------------------------

def run_skill_structure(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    skills_dir = repo_root / "context" / "skills"

    skill_files = sorted(skills_dir.rglob("SKILL.md"))
    if not skill_files:
        findings.append(Finding(
            severity="WARN",
            category="skill_structure",
            id="skill.no-skills-found",
            message="No SKILL.md files found under context/skills/",
        ))
        return findings

    for skill_path in skill_files:
        rel = skill_path.relative_to(repo_root)
        ref = str(rel)

        try:
            text = skill_path.read_text(encoding="utf-8")
        except OSError as exc:
            findings.append(Finding(
                severity="ERROR",
                category="skill_structure",
                id="skill.unreadable",
                message=f"Cannot read file: {exc}",
                entity_ref=ref,
            ))
            continue

        fm_yaml, body = _split_frontmatter(text)
        if fm_yaml is None:
            findings.append(Finding(
                severity="ERROR",
                category="skill_structure",
                id="skill.no-frontmatter",
                message="No YAML frontmatter found",
                entity_ref=ref,
            ))
            continue

        try:
            data = yaml.safe_load(fm_yaml)
        except yaml.YAMLError as exc:
            findings.append(Finding(
                severity="ERROR",
                category="skill_structure",
                id="skill.bad-frontmatter",
                message=f"Frontmatter YAML parse error: {exc}",
                entity_ref=ref,
            ))
            continue

        if not isinstance(data, dict):
            findings.append(Finding(
                severity="ERROR",
                category="skill_structure",
                id="skill.bad-frontmatter",
                message="Frontmatter parsed to non-dict",
                entity_ref=ref,
            ))
            continue

        # Check required frontmatter fields
        field_errors = False
        for dotpath in SKILL_REQUIRED_FIELDS:
            val = _get_nested(data, dotpath)
            if val is None:
                findings.append(Finding(
                    severity="ERROR",
                    category="skill_structure",
                    id="skill.missing-field",
                    message=f"Required frontmatter field missing: {dotpath}",
                    entity_ref=ref,
                ))
                field_errors = True

        # Check required sections in body
        section_errors = False
        for section in SKILL_REQUIRED_SECTIONS:
            # Match the section header at start of a line
            pattern = re.compile(r"^" + re.escape(section) + r"(\s|$)", re.MULTILINE)
            if not pattern.search(body):
                findings.append(Finding(
                    severity="ERROR",
                    category="skill_structure",
                    id="skill.missing-section",
                    message=f"Required section missing: {section}",
                    entity_ref=ref,
                ))
                section_errors = True

        if not field_errors and not section_errors:
            skill_name = data.get("name", rel.parts[-2] if len(rel.parts) >= 2 else ref)
            findings.append(Finding(
                severity="INFO",
                category="skill_structure",
                id="skill.ok",
                message=f"{skill_name} — OK",
                entity_ref=ref,
            ))

    return findings


# ---------------------------------------------------------------------------
# Check 3: mcp_annotations
# ---------------------------------------------------------------------------

# Matches @server.tool(...) decorator lines (with possible multiline args).
# We look for lines that contain @server.tool and then scan the decorator
# block for annotations=ToolAnnotations(...).
_TOOL_DECO_RE = re.compile(r"@\w+\.tool\(")
_ANNOTATIONS_RE = re.compile(r"annotations\s*=\s*ToolAnnotations\s*\(")


def _extract_tool_blocks(source: str) -> list[tuple[int, str]]:
    """Return list of (lineno, decorator_block) for each @server.tool() found.

    decorator_block includes from the @-line up to (but not including) the
    def line. Lineno is 1-based.
    """
    lines = source.splitlines()
    blocks: list[tuple[int, str]] = []
    i = 0
    while i < len(lines):
        if _TOOL_DECO_RE.search(lines[i]):
            start = i
            block_lines = [lines[i]]
            # Collect continuation lines until we hit a 'def ' line or
            # another decorator or a blank line followed by def.
            j = i + 1
            while j < len(lines):
                stripped = lines[j].strip()
                if stripped.startswith("def ") or stripped.startswith("async def "):
                    break
                if stripped.startswith("@") and not stripped.startswith("@server.tool"):
                    break
                block_lines.append(lines[j])
                j += 1
            blocks.append((start + 1, "\n".join(block_lines)))
            i = j
        else:
            i += 1
    return blocks


def run_mcp_annotations(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    skills_dir = repo_root / "context" / "skills"

    server_files = sorted(skills_dir.rglob("mcp/server.py"))
    if not server_files:
        findings.append(Finding(
            severity="INFO",
            category="mcp_annotations",
            id="mcp.no-servers",
            message="No mcp/server.py files found under context/skills/",
        ))
        return findings

    for server_path in server_files:
        rel = server_path.relative_to(repo_root)
        ref = str(rel)
        skill_slug = rel.parts[-3] if len(rel.parts) >= 3 else ref

        try:
            source = server_path.read_text(encoding="utf-8")
        except OSError as exc:
            findings.append(Finding(
                severity="ERROR",
                category="mcp_annotations",
                id="mcp.unreadable",
                message=f"Cannot read file: {exc}",
                entity_ref=ref,
            ))
            continue

        tool_blocks = _extract_tool_blocks(source)
        if not tool_blocks:
            findings.append(Finding(
                severity="INFO",
                category="mcp_annotations",
                id="mcp.no-tools",
                message=f"{skill_slug} — no @server.tool() decorators found",
                entity_ref=ref,
            ))
            continue

        # Find tool names from def lines following each decorator block
        lines = source.splitlines()
        tool_errors = False
        for lineno, block in tool_blocks:
            # Tool name: look at the def line right after the block end
            block_line_count = block.count("\n") + 1
            def_lineno = lineno + block_line_count - 1  # 0-based index
            tool_name = "unknown"
            if def_lineno < len(lines):
                def_match = re.search(r"(?:async\s+)?def\s+(\w+)", lines[def_lineno])
                if def_match:
                    tool_name = def_match.group(1)

            # Check annotations= present in decorator block
            if not _ANNOTATIONS_RE.search(block):
                findings.append(Finding(
                    severity="ERROR",
                    category="mcp_annotations",
                    id="mcp.missing-annotations",
                    message=f"{skill_slug}:{tool_name} (line {lineno}) — @server.tool() missing annotations=ToolAnnotations(...)",
                    entity_ref=ref,
                ))
                tool_errors = True
                continue

            # Check all four hint keys present in the decorator block area.
            # Extend search a bit past the block to catch multi-line ToolAnnotations().
            # Find the annotations=ToolAnnotations( opening and scan until closing )
            anno_match = _ANNOTATIONS_RE.search(block)
            if anno_match:
                anno_start = block.find("annotations=")
                # Grab from annotations= to end of block (may be truncated if
                # ToolAnnotations spans into the def body — that's a heuristic;
                # in practice all four keys fit on one line or within the block)
                anno_fragment = block[anno_start:]
                # Also grab a few more lines after the block if needed
                extra_lines_start = lineno + block_line_count - 1
                extra_lines_end = min(extra_lines_start + 10, len(lines))
                extra = "\n".join(lines[extra_lines_start:extra_lines_end])
                search_text = anno_fragment + "\n" + extra

                missing_keys = [k for k in MCP_ANNOTATION_KEYS if k not in search_text]
                if missing_keys:
                    findings.append(Finding(
                        severity="ERROR",
                        category="mcp_annotations",
                        id="mcp.incomplete-annotations",
                        message=(
                            f"{skill_slug}:{tool_name} (line {lineno}) — "
                            f"ToolAnnotations missing keys: {', '.join(missing_keys)}"
                        ),
                        entity_ref=ref,
                    ))
                    tool_errors = True
                else:
                    findings.append(Finding(
                        severity="INFO",
                        category="mcp_annotations",
                        id="mcp.ok",
                        message=f"{skill_slug}:{tool_name} — annotations OK",
                        entity_ref=ref,
                    ))

        if not tool_errors and tool_blocks:
            pass  # individual INFO findings already added above

    return findings


# ---------------------------------------------------------------------------
# Check 4: cross_references
# ---------------------------------------------------------------------------

def _build_skill_index(repo_root: Path) -> dict[str, Path]:
    """Build {skill_name: SKILL.md path} index from all context/skills/**."""
    index: dict[str, Path] = {}
    skills_dir = repo_root / "context" / "skills"
    for skill_md in skills_dir.rglob("SKILL.md"):
        # skill name = the directory containing SKILL.md
        skill_name = skill_md.parent.name
        index[skill_name] = skill_md
    return index


def run_cross_references(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    skills_dir = repo_root / "context" / "skills"
    skill_index = _build_skill_index(repo_root)

    skill_files = sorted(skills_dir.rglob("SKILL.md"))
    for skill_path in skill_files:
        rel = skill_path.relative_to(repo_root)
        ref = str(rel)
        skill_name = skill_path.parent.name

        try:
            text = skill_path.read_text(encoding="utf-8")
        except OSError:
            continue  # already caught in skill_structure

        fm_yaml, _body = _split_frontmatter(text)
        if fm_yaml is None:
            continue

        try:
            data = yaml.safe_load(fm_yaml)
        except yaml.YAMLError:
            continue

        if not isinstance(data, dict):
            continue

        uses = _get_nested(data, "metadata.processkit.uses")
        if not uses:
            continue
        if not isinstance(uses, list):
            continue

        for entry in uses:
            if not isinstance(entry, dict):
                continue
            dep_name = entry.get("skill")
            if not dep_name:
                continue

            if dep_name in skill_index:
                findings.append(Finding(
                    severity="INFO",
                    category="cross_references",
                    id="xref.ok",
                    message=f"{skill_name} → {dep_name} — resolved",
                    entity_ref=ref,
                ))
            else:
                findings.append(Finding(
                    severity="ERROR",
                    category="cross_references",
                    id="xref.unresolved",
                    message=(
                        f"{skill_name} → {dep_name} — no SKILL.md found "
                        f"at context/skills/<category>/{dep_name}/SKILL.md"
                    ),
                    entity_ref=ref,
                ))

    return findings


# ---------------------------------------------------------------------------
# Report formatting
# ---------------------------------------------------------------------------

SEV_GLYPH = {"ERROR": "E", "WARN": "W", "INFO": "i"}


def _format_report(
    per_check: dict[str, list[Finding]],
    repo_root: Path,
) -> str:
    lines: list[str] = []
    lines.append(f"# pk-release-audit v{AUDIT_VERSION}")
    lines.append(f"repo_root: {repo_root}")
    lines.append("")
    grand = {"ERROR": 0, "WARN": 0, "INFO": 0}

    for check_name, findings in per_check.items():
        t = tally(findings)
        for k, v in t.items():
            grand[k] += v
        lines.append(
            f"## {check_name} — {t['ERROR']} ERROR / {t['WARN']} WARN / {t['INFO']} INFO"
        )
        for f in findings:
            if f.severity == "INFO":
                lines.append(f"  [i] {f.message}")
            else:
                g = SEV_GLYPH[f.severity]
                ref_suffix = f"  ({f.entity_ref})" if f.entity_ref else ""
                lines.append(f"  [{g}] {f.id} — {f.message}{ref_suffix}")
        lines.append("")

    lines.append(
        f"## totals — {grand['ERROR']} ERROR / {grand['WARN']} WARN / {grand['INFO']} INFO"
    )
    lines.append("")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

CHECKS = [
    ("entity_files", run_entity_files),
    ("skill_structure", run_skill_structure),
    ("mcp_annotations", run_mcp_annotations),
    ("cross_references", run_cross_references),
]


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


def main(argv: Optional[list[str]] = None) -> int:
    p = argparse.ArgumentParser(
        prog="pk-release-audit",
        description="Detect-only pre-release validation sweep for a processkit repo.",
    )
    p.add_argument("--repo-root", default=None,
                   help="Explicit repo root path. Defaults to git rev-parse --show-toplevel.")
    args = p.parse_args(argv)

    repo_root = _resolve_repo_root(args.repo_root)

    per_check: dict[str, list[Finding]] = {}
    for check_name, check_fn in CHECKS:
        per_check[check_name] = check_fn(repo_root)

    report = _format_report(per_check, repo_root)
    print(report)

    grand_errors = sum(tally(findings)["ERROR"] for findings in per_check.values())
    return 1 if grand_errors > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
