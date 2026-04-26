"""Tests for the catalog() tool in the skill-finder MCP server.

Run with:
    uv run --with pyyaml --with pytest --with mcp pytest \
        context/skills/processkit/skill-finder/scripts/test_catalog.py -v
"""
from __future__ import annotations

import importlib.util
import json
import os
import sys
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# Bootstrap: load server module from the canonical path, resolving _find_lib
# ---------------------------------------------------------------------------

SERVER_PATH = (
    Path(__file__).resolve().parents[1] / "mcp" / "server.py"
)

# Locate project root (where AGENTS.md lives) so we can set cwd
PROJECT_ROOT = Path(__file__).resolve()
while PROJECT_ROOT.parent != PROJECT_ROOT:
    if (PROJECT_ROOT / "AGENTS.md").exists():
        break
    PROJECT_ROOT = PROJECT_ROOT.parent

os.chdir(PROJECT_ROOT)


def _load_server():
    """Import server.py as a module without executing __main__."""
    spec = importlib.util.spec_from_file_location("skill_finder_server", SERVER_PATH)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


_server = _load_server()
catalog = _server.catalog


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


class TestCatalogDefault:
    """Default call — all skills, default columns, markdown output."""

    def test_returns_string(self):
        result = catalog()
        assert isinstance(result, str)

    def test_at_least_30_skills(self):
        result = catalog()
        # Each data row starts with "| " — count rows excluding header/sep
        rows = [
            line for line in result.splitlines()
            if line.startswith("|") and "---" not in line
            and "name" not in line.lower().split("|")[1].strip().lower()
            or (
                line.startswith("|")
                and "---" not in line
                and line.count("|") >= 3
            )
        ]
        # More robust: count non-header, non-separator table rows
        data_rows = [
            line for line in result.splitlines()
            if line.startswith("|")
            and "---" not in line
            and not line.lower().startswith("| name")
        ]
        assert len(data_rows) >= 30, (
            f"Expected >=30 skills, got {len(data_rows)}"
        )

    def test_default_columns_present(self):
        result = catalog()
        header_line = result.splitlines()[0]
        assert "name" in header_line
        assert "category" in header_line
        assert "description" in header_line


class TestCatalogCategoryFilter:
    """category= filter narrows results."""

    def test_processkit_only(self):
        result = catalog(category="processkit", output="json")
        assert isinstance(result, list)
        assert len(result) > 0
        for row in result:
            assert row["category"] == "processkit", (
                f"Unexpected category: {row}"
            )

    def test_processkit_excludes_engineering(self):
        result = catalog(category="processkit", output="json")
        for row in result:
            assert row["category"] != "engineering"

    def test_unknown_category_returns_empty(self):
        result = catalog(category="nonexistent-category", output="json")
        assert isinstance(result, list)
        assert result == []


class TestCatalogKeyword:
    """keyword= filter searches name and description."""

    def test_model_returns_results(self):
        result = catalog(keyword="model", output="json")
        assert isinstance(result, list)
        assert len(result) >= 1, "Expected >=1 skill matching 'model'"

    def test_model_hits_contain_keyword(self):
        result = catalog(keyword="model", output="json")
        for row in result:
            name_hit = "model" in row.get("name", "").lower()
            desc_hit = "model" in row.get("description", "").lower()
            assert name_hit or desc_hit, (
                f"Row doesn't contain 'model' in name or description: {row}"
            )

    def test_keyword_case_insensitive(self):
        lower = catalog(keyword="model", output="json")
        upper = catalog(keyword="MODEL", output="json")
        assert len(lower) == len(upper)


class TestCatalogOutputJson:
    """output="json" returns valid, parseable list[dict]."""

    def test_returns_list(self):
        result = catalog(output="json")
        assert isinstance(result, list)

    def test_serialisable_by_json_loads(self):
        result = catalog(output="json")
        serialised = json.dumps(result)
        parsed = json.loads(serialised)
        assert isinstance(parsed, list)
        assert len(parsed) == len(result)

    def test_default_keys_present(self):
        result = catalog(output="json")
        for row in result:
            assert "name" in row
            assert "category" in row
            assert "description" in row


class TestCatalogColumns:
    """columns= restricts returned keys."""

    def test_name_and_layer_only(self):
        result = catalog(columns=["name", "layer"], output="json")
        assert isinstance(result, list)
        assert len(result) > 0
        for row in result:
            assert set(row.keys()) == {"name", "layer"}, (
                f"Unexpected keys: {set(row.keys())}"
            )

    def test_single_column(self):
        result = catalog(columns=["name"], output="json")
        for row in result:
            assert list(row.keys()) == ["name"]

    def test_all_columns_valid(self):
        all_cols = [
            "name", "category", "description", "tags",
            "version", "layer", "allowed_tools", "has_mcp", "skill_md_path",
        ]
        result = catalog(columns=all_cols, output="json")
        assert isinstance(result, list)
        assert len(result) > 0
        for row in result:
            assert set(row.keys()) == set(all_cols)

    def test_unknown_column_returns_error_string(self):
        result = catalog(columns=["name", "bogus_col"])
        assert isinstance(result, str)
        assert "bogus_col" in result.lower() or "unknown" in result.lower()


class TestCatalogOutputYaml:
    """output="yaml" returns a YAML fenced block."""

    def test_returns_fenced_yaml(self):
        result = catalog(output="yaml")
        assert isinstance(result, str)
        assert result.startswith("```yaml")
        assert result.endswith("```")

    def test_yaml_parseable(self):
        import yaml
        result = catalog(output="yaml")
        # Strip fences
        inner = result[len("```yaml"):].rstrip("`").strip()
        parsed = yaml.safe_load(inner)
        assert isinstance(parsed, list)
        assert len(parsed) >= 30


class TestCatalogSortBy:
    """sort_by= controls ordering."""

    def test_sort_by_category(self):
        result = catalog(sort_by="category", output="json")
        categories = [r["category"] for r in result]
        assert categories == sorted(categories, key=str.lower)

    def test_default_sort_by_name(self):
        result = catalog(output="json")
        names = [r["name"] for r in result]
        assert names == sorted(names, key=str.lower)


class TestCatalogMarkdownShape:
    """Markdown output has correct table structure."""

    def test_has_header_and_separator(self):
        result = catalog()
        lines = result.splitlines()
        assert lines[0].startswith("|")
        assert "---" in lines[1]

    def test_category_filter_markdown(self):
        result = catalog(category="processkit")
        assert isinstance(result, str)
        # All data rows should mention processkit
        data_rows = [
            line for line in result.splitlines()
            if line.startswith("|")
            and "---" not in line
            and not line.lower().startswith("| name")
        ]
        for row in data_rows:
            assert "processkit" in row, (
                f"Row doesn't mention processkit: {row}"
            )
