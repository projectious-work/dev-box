"""commands_consistency check.

Every processkit skill that declares `metadata.processkit.commands:` in
its SKILL.md must ship a matching `commands/<name>.md` file alongside
SKILL.md so the harness can register the slash command. v0.19.1 shipped
pk-doctor with the metadata stanza but without the commands/ directory;
this check prevents recurrence.

ERROR per missing commands/<name>.md. One INFO if all skills are
consistent. WARN if a skill has commands/*.md files that are not
declared in its metadata (stale file — harmless but worth surfacing).
"""

from __future__ import annotations

from pathlib import Path

import yaml

from .common import CheckResult


def _parse_frontmatter(path: Path) -> dict | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        data = yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


def _declared_commands(frontmatter: dict) -> list[str]:
    meta = frontmatter.get("metadata")
    if not isinstance(meta, dict):
        return []
    pk = meta.get("processkit")
    if not isinstance(pk, dict):
        return []
    commands = pk.get("commands")
    if not isinstance(commands, list):
        return []
    names: list[str] = []
    for entry in commands:
        if isinstance(entry, dict) and isinstance(entry.get("name"), str):
            names.append(entry["name"])
    return names


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]
    skills_root = repo_root / "context" / "skills" / "processkit"
    if not skills_root.is_dir():
        return [CheckResult(
            severity="INFO",
            category="commands_consistency",
            id="commands_consistency.no-skills-dir",
            message=f"{skills_root.relative_to(repo_root)} not found; skipping",
        )]

    findings: list[CheckResult] = []

    for skill_md in sorted(skills_root.glob("*/SKILL.md")):
        skill_dir = skill_md.parent
        skill_slug = skill_dir.name
        fm = _parse_frontmatter(skill_md)
        if fm is None:
            continue
        declared = _declared_commands(fm)
        commands_dir = skill_dir / "commands"
        existing = set()
        if commands_dir.is_dir():
            existing = {p.stem for p in commands_dir.glob("*.md")}

        for name in declared:
            expected = commands_dir / f"{name}.md"
            if not expected.is_file():
                findings.append(CheckResult(
                    severity="ERROR",
                    category="commands_consistency",
                    id="commands_consistency.missing-command-file",
                    message=(
                        f"skill '{skill_slug}' declares command '{name}' in SKILL.md "
                        f"metadata but {expected.relative_to(repo_root)} is missing"
                    ),
                    entity_ref=f"SKILL-{skill_slug}",
                ))

        declared_set = set(declared)
        for stem in sorted(existing - declared_set):
            findings.append(CheckResult(
                severity="WARN",
                category="commands_consistency",
                id="commands_consistency.undeclared-command-file",
                message=(
                    f"skill '{skill_slug}' ships commands/{stem}.md but '{stem}' is "
                    f"not declared in SKILL.md metadata.processkit.commands"
                ),
                entity_ref=f"SKILL-{skill_slug}",
            ))

    if not findings:
        return [CheckResult(
            severity="INFO",
            category="commands_consistency",
            id="commands_consistency.ok",
            message="all skill commands: metadata entries have matching commands/*.md files",
        )]
    return findings
