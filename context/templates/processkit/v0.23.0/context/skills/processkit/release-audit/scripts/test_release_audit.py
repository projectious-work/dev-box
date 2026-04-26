#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
#   "pytest>=7.0",
# ]
# ///
"""test_release_audit.py — pytest tests for release_audit.py.

Tests:
- A clean fixture tree passes all checks with 0 ERRORs.
- A fixture with a missing 'kind' field surfaces an entity_files ERROR.
- A fixture with a missing '## Gotchas' section in a SKILL.md surfaces a
  skill_structure ERROR.

Run:
    uv run --with pyyaml --with pytest pytest context/skills/processkit/release-audit/scripts/test_release_audit.py -v
"""

from __future__ import annotations

import sys
import textwrap
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# Make release_audit importable without installing it.
# ---------------------------------------------------------------------------
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

from release_audit import (  # noqa: E402
    run_entity_files,
    run_skill_structure,
    run_mcp_annotations,
    run_cross_references,
    tally,
    EXPECTED_API_VERSION,
)


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------

def _make_entity(
    tmp_path: Path,
    dir_name: str,
    stem: str,
    kind: str,
    override_api_version: str = "",
    override_kind: str = "",
    override_id: str = "",
) -> Path:
    """Write a minimal valid entity file. Use override_* to replace specific fields."""
    entity_dir = tmp_path / "context" / dir_name
    entity_dir.mkdir(parents=True, exist_ok=True)
    api_version = override_api_version or EXPECTED_API_VERSION
    fm_kind = override_kind or kind
    fm_id = override_id or stem
    lines = [
        "---",
        f"apiVersion: {api_version}",
        f"kind: {fm_kind}",
        "metadata:",
        f"  id: {fm_id}",
        "---",
        "",
        "Body text.",
    ]
    content = "\n".join(lines) + "\n"
    path = entity_dir / f"{stem}.md"
    path.write_text(content, encoding="utf-8")
    return path


def _make_skill(tmp_path: Path, category: str, name: str, *, missing_section: str = "") -> Path:
    """Write a minimal valid SKILL.md. Pass missing_section to omit one required section."""
    skill_dir = tmp_path / "context" / "skills" / category / name
    skill_dir.mkdir(parents=True, exist_ok=True)
    sections = ["## Intro", "## Overview", "## Gotchas", "## Full reference"]
    body_parts = []
    for s in sections:
        if s != missing_section:
            body_parts.append(f"{s}\n\nContent for {s}.")
    body_sections = "\n\n".join(body_parts)
    # Build content line-by-line to avoid textwrap.dedent issues with
    # f-string interpolation consuming indentation.
    lines = [
        "---",
        f"name: {name}",
        "description: |",
        f"  Test skill {name}.",
        "metadata:",
        "  processkit:",
        f"    apiVersion: {EXPECTED_API_VERSION}",
        f"    id: SKILL-{name}",
        '    version: "1.0.0"',
        f"    category: {category}",
        "    layer: 3",
        "---",
        "",
        f"# {name}",
        "",
        body_sections,
    ]
    content = "\n".join(lines) + "\n"
    path = skill_dir / "SKILL.md"
    path.write_text(content, encoding="utf-8")
    return path


# ---------------------------------------------------------------------------
# Test: clean tree passes
# ---------------------------------------------------------------------------

class TestCleanTree:
    """A minimal fully-valid fixture tree produces 0 ERRORs on all checks."""

    def setup_method(self, tmp_path_factory):
        pass

    def test_entity_files_clean(self, tmp_path):
        _make_entity(tmp_path, "workitems", "BACK-20260101_1200-TestItem-sample", "WorkItem")
        _make_entity(tmp_path, "decisions", "DEC-20260101_1200-TestDec-sample", "DecisionRecord")
        findings = run_entity_files(tmp_path)
        t = tally(findings)
        assert t["ERROR"] == 0, f"Expected 0 ERRORs, got: {[f for f in findings if f.severity == 'ERROR']}"

    def test_skill_structure_clean(self, tmp_path):
        _make_skill(tmp_path, "processkit", "my-skill")
        findings = run_skill_structure(tmp_path)
        t = tally(findings)
        assert t["ERROR"] == 0, f"Expected 0 ERRORs, got: {[f for f in findings if f.severity == 'ERROR']}"

    def test_mcp_annotations_no_servers(self, tmp_path):
        """No server.py files → INFO-only, 0 ERRORs."""
        (tmp_path / "context" / "skills").mkdir(parents=True, exist_ok=True)
        findings = run_mcp_annotations(tmp_path)
        t = tally(findings)
        assert t["ERROR"] == 0

    def test_cross_references_no_uses(self, tmp_path):
        """Skill with no uses: block → 0 ERRORs."""
        _make_skill(tmp_path, "processkit", "standalone")
        findings = run_cross_references(tmp_path)
        t = tally(findings)
        assert t["ERROR"] == 0

    def test_cross_references_resolved(self, tmp_path):
        """Skill that uses another existing skill → 0 ERRORs."""
        _make_skill(tmp_path, "processkit", "dep-skill")

        # Make a skill that uses dep-skill
        skill_dir = tmp_path / "context" / "skills" / "processkit" / "main-skill"
        skill_dir.mkdir(parents=True, exist_ok=True)
        content = textwrap.dedent(f"""\
            ---
            name: main-skill
            description: |
              Test main skill.
            metadata:
              processkit:
                apiVersion: {EXPECTED_API_VERSION}
                id: SKILL-main-skill
                version: "1.0.0"
                category: processkit
                layer: 3
                uses:
                  - skill: dep-skill
                    purpose: Testing cross-reference resolution.
            ---

            # main-skill

            ## Intro

            Content.

            ## Overview

            Content.

            ## Gotchas

            Content.

            ## Full reference

            Content.
        """)
        (skill_dir / "SKILL.md").write_text(content, encoding="utf-8")

        findings = run_cross_references(tmp_path)
        t = tally(findings)
        assert t["ERROR"] == 0, f"Expected 0 ERRORs, got: {[f for f in findings if f.severity == 'ERROR']}"
        info_msgs = [f.message for f in findings if f.severity == "INFO"]
        assert any("main-skill → dep-skill" in m for m in info_msgs)


# ---------------------------------------------------------------------------
# Test: missing 'kind' field surfaces ERROR
# ---------------------------------------------------------------------------

class TestMissingKind:
    """Entity file with no 'kind' field surfaces an entity_files ERROR."""

    def test_missing_kind_is_error(self, tmp_path):
        entity_dir = tmp_path / "context" / "workitems"
        entity_dir.mkdir(parents=True, exist_ok=True)
        content = textwrap.dedent(f"""\
            ---
            apiVersion: {EXPECTED_API_VERSION}
            metadata:
              id: BACK-20260101_1200-NoKind-sample
            ---

            Body.
        """)
        (entity_dir / "BACK-20260101_1200-NoKind-sample.md").write_text(content, encoding="utf-8")

        findings = run_entity_files(tmp_path)
        error_ids = [f.id for f in findings if f.severity == "ERROR"]
        assert "entity.missing-kind" in error_ids, (
            f"Expected entity.missing-kind ERROR. Got: {error_ids}"
        )

    def test_wrong_api_version_is_error(self, tmp_path):
        _make_entity(
            tmp_path,
            "workitems",
            "BACK-20260101_1200-BadVer-sample",
            "WorkItem",
            override_api_version="processkit.projectious.work/v0",
        )
        findings = run_entity_files(tmp_path)
        error_ids = [f.id for f in findings if f.severity == "ERROR"]
        assert "entity.wrong-api-version" in error_ids

    def test_unknown_kind_is_error(self, tmp_path):
        _make_entity(
            tmp_path,
            "workitems",
            "BACK-20260101_1200-BadKind-sample",
            "WorkItem",
            override_kind="UnknownKind",
        )
        findings = run_entity_files(tmp_path)
        error_ids = [f.id for f in findings if f.severity == "ERROR"]
        assert "entity.unknown-kind" in error_ids

    def test_id_mismatch_is_error(self, tmp_path):
        _make_entity(
            tmp_path,
            "workitems",
            "BACK-20260101_1200-WrongId-sample",
            "WorkItem",
            override_id="BACK-20260101_1200-DifferentId-sample",
        )
        findings = run_entity_files(tmp_path)
        error_ids = [f.id for f in findings if f.severity == "ERROR"]
        assert "entity.id-filename-mismatch" in error_ids


# ---------------------------------------------------------------------------
# Test: missing ## Gotchas section in SKILL.md surfaces ERROR
# ---------------------------------------------------------------------------

class TestMissingGotchasSection:
    """SKILL.md missing the ## Gotchas section surfaces a skill_structure ERROR."""

    def test_missing_gotchas_is_error(self, tmp_path):
        _make_skill(tmp_path, "processkit", "bad-skill", missing_section="## Gotchas")
        findings = run_skill_structure(tmp_path)
        errors = [f for f in findings if f.severity == "ERROR"]
        assert any(
            f.id == "skill.missing-section" and "## Gotchas" in f.message
            for f in errors
        ), f"Expected skill.missing-section ERROR for ## Gotchas. Got: {errors}"

    def test_missing_intro_is_error(self, tmp_path):
        _make_skill(tmp_path, "processkit", "bad-skill2", missing_section="## Intro")
        findings = run_skill_structure(tmp_path)
        errors = [f for f in findings if f.severity == "ERROR"]
        assert any(
            f.id == "skill.missing-section" and "## Intro" in f.message
            for f in errors
        )

    def test_missing_frontmatter_field_is_error(self, tmp_path):
        """SKILL.md missing metadata.processkit.layer surfaces ERROR."""
        skill_dir = tmp_path / "context" / "skills" / "processkit" / "no-layer"
        skill_dir.mkdir(parents=True, exist_ok=True)
        content = textwrap.dedent(f"""\
            ---
            name: no-layer
            description: |
              Missing layer field.
            metadata:
              processkit:
                apiVersion: {EXPECTED_API_VERSION}
                id: SKILL-no-layer
                version: "1.0.0"
                category: processkit
            ---

            # no-layer

            ## Intro

            Content.

            ## Overview

            Content.

            ## Gotchas

            Content.

            ## Full reference

            Content.
        """)
        (skill_dir / "SKILL.md").write_text(content, encoding="utf-8")

        findings = run_skill_structure(tmp_path)
        errors = [f for f in findings if f.severity == "ERROR"]
        assert any(
            f.id == "skill.missing-field" and "layer" in f.message
            for f in errors
        ), f"Expected skill.missing-field ERROR for layer. Got: {errors}"


# ---------------------------------------------------------------------------
# Test: MCP annotation checks
# ---------------------------------------------------------------------------

class TestMcpAnnotations:
    def _write_server(self, tmp_path: Path, skill_name: str, tool_source: str) -> Path:
        mcp_dir = tmp_path / "context" / "skills" / "processkit" / skill_name / "mcp"
        mcp_dir.mkdir(parents=True, exist_ok=True)
        server_path = mcp_dir / "server.py"
        server_path.write_text(tool_source, encoding="utf-8")
        return server_path

    def test_annotated_tool_passes(self, tmp_path):
        source = textwrap.dedent("""\
            from mcp.server.fastmcp import FastMCP
            from mcp.types import ToolAnnotations
            server = FastMCP("test")

            @server.tool(
                name="my_tool",
                annotations=ToolAnnotations(readOnlyHint=True, destructiveHint=False, idempotentHint=True, openWorldHint=False),
            )
            def my_tool():
                pass
        """)
        self._write_server(tmp_path, "annotated-skill", source)
        findings = run_mcp_annotations(tmp_path)
        t = tally(findings)
        assert t["ERROR"] == 0, f"Expected 0 ERRORs, got: {[f for f in findings if f.severity == 'ERROR']}"

    def test_missing_annotations_is_error(self, tmp_path):
        source = textwrap.dedent("""\
            from mcp.server.fastmcp import FastMCP
            server = FastMCP("test")

            @server.tool(name="bare_tool")
            def bare_tool():
                pass
        """)
        self._write_server(tmp_path, "bare-skill", source)
        findings = run_mcp_annotations(tmp_path)
        errors = [f for f in findings if f.severity == "ERROR"]
        assert any(f.id == "mcp.missing-annotations" for f in errors), (
            f"Expected mcp.missing-annotations ERROR. Got: {errors}"
        )


# ---------------------------------------------------------------------------
# Test: cross-reference errors
# ---------------------------------------------------------------------------

class TestCrossReferences:
    def test_unresolved_ref_is_error(self, tmp_path):
        skill_dir = tmp_path / "context" / "skills" / "processkit" / "orphan-skill"
        skill_dir.mkdir(parents=True, exist_ok=True)
        content = textwrap.dedent(f"""\
            ---
            name: orphan-skill
            description: |
              Uses a non-existent skill.
            metadata:
              processkit:
                apiVersion: {EXPECTED_API_VERSION}
                id: SKILL-orphan-skill
                version: "1.0.0"
                category: processkit
                layer: 3
                uses:
                  - skill: does-not-exist
                    purpose: Testing unresolved reference.
            ---

            # orphan-skill

            ## Intro

            Content.

            ## Overview

            Content.

            ## Gotchas

            Content.

            ## Full reference

            Content.
        """)
        (skill_dir / "SKILL.md").write_text(content, encoding="utf-8")

        findings = run_cross_references(tmp_path)
        errors = [f for f in findings if f.severity == "ERROR"]
        assert any(f.id == "xref.unresolved" for f in errors), (
            f"Expected xref.unresolved ERROR. Got: {errors}"
        )
