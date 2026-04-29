"""Tests for the shared frontmatter renderer.

Run with:

    uv run --with pyyaml --with pytest \
        pytest context/skills/_lib/processkit/test_frontmatter.py -v

Covers the CalmArch fix (BACK-20260425_1755): multi-line strings must
serialize as YAML literal block scalars (`|`) so that markdown bodies
containing pipe-tables, fenced code blocks, and backslashes round-trip
safely.
"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest
import yaml

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from processkit.frontmatter import parse, render


def _canonical(s: str) -> str:
    """Multi-line strings round-trip with a single trailing newline (YAML
    clip chomping). Tests compare against this canonical shape."""
    return s.rstrip("\n") + "\n"


def _roundtrip(data: dict, body: str = "") -> tuple[dict, str]:
    text = render(data, body)
    return parse(text)


def test_render_short_string_is_unquoted_or_single_quoted():
    """Single-line strings should not be forced into block style."""
    text = render({"title": "a short title"})
    assert "title: a short title\n" in text or "title: 'a short title'\n" in text
    assert "|" not in text


def test_render_multi_line_string_uses_block_scalar():
    """Multi-line strings must use the `|` literal block scalar style."""
    data = {"description": "first line\nsecond line\n"}
    text = render(data)
    assert "description: |" in text
    assert "  first line" in text
    assert "  second line" in text


def test_pipe_table_roundtrip():
    """The CalmArch repro: a description with a markdown pipe-table must
    survive serialize → parse with no YAMLError and identical content
    (modulo trailing newline normalization)."""
    description = (
        "## Behavior\n"
        "\n"
        "| col-a | col-b | col-c |\n"
        "|---|---|---|\n"
        "| `pipe \\| in code` | path\\to\\thing | normal |\n"
        "\n"
        "End of description."
    )
    out, _ = _roundtrip({"id": "BACK-x", "description": description})
    assert out["description"] == _canonical(description)


def test_fenced_code_block_roundtrip():
    """Fenced code blocks (with backticks and newlines) round-trip cleanly."""
    description = "Example:\n\n```python\ndef f():\n    return 1\n```\n"
    out, _ = _roundtrip({"description": description})
    assert out["description"] == _canonical(description)


def test_backslash_heavy_string_roundtrip():
    """Backslashes in multi-line strings (Windows paths, regex, escapes)."""
    description = (
        "regex: \\d+\\s+\\w+\n"
        "path: C:\\Users\\me\\file.txt\n"
        "escaped: \\n is a newline literal in regex\n"
    )
    out, _ = _roundtrip({"description": description})
    assert out["description"] == _canonical(description)


def test_nested_dict_with_multi_line_string():
    """Block style must work for multi-line strings nested in a mapping."""
    data = {
        "metadata": {"id": "X-1"},
        "spec": {
            "title": "single line",
            "description": "line 1\nline 2\nline 3\n",
        },
    }
    out, _ = _roundtrip(data)
    assert out["spec"]["description"] == _canonical("line 1\nline 2\nline 3")
    assert out["spec"]["title"] == "single line"


def test_input_without_trailing_newline_is_normalized():
    """Inputs without a trailing newline gain one on round-trip — this is
    YAML clip-chomp behavior and the canonical shape we emit."""
    out, _ = _roundtrip({"description": "no\ntrailing\nnewline"})
    assert out["description"] == "no\ntrailing\nnewline\n"


def test_quickbison_shape_no_parse_error():
    """Reproduce the original QuickBison shape: a description that, before
    the fix, failed YAML parsing because the double-quoted style escaped
    things in a way safe_load could not recover. The round-trip must
    succeed."""
    description = (
        "## Why\n"
        "\n"
        "Investigate a provider-neutral SubAgent primitive (modelled on\n"
        "Claude Code `/agents`, targets `.claude/agents/`, `.codex/`, …).\n"
        "\n"
        "## Constraints\n"
        "\n"
        "- agents = `{name, description, prompt, allowed_tools[]}`\n"
        "- pipe-table:\n"
        "\n"
        "| harness | dir | format |\n"
        "|---|---|---|\n"
        "| Claude Code | `.claude/agents/` | YAML frontmatter |\n"
        "| Codex CLI | `.codex/agents/` | TOML |\n"
    )
    out, _ = _roundtrip({"description": description})
    assert out["description"] == _canonical(description)
