---
name: python-best-practices
description: Python conventions and patterns — typing, testing, project layout, tooling. Use when writing or reviewing Python code.
---

# Python Best Practices

## When to Use

When the user is working with Python code and asks about conventions, project structure, typing, testing, or says "how should I structure this Python project?".

## Instructions

1. **Project layout:**
   - Use `src/` layout: `src/package_name/` with `__init__.py`
   - Configuration in `pyproject.toml` (not setup.py or setup.cfg)
   - Use `uv` for dependency management when available
2. **Type hints:**
   - Add type hints to all public function signatures
   - Use `from __future__ import annotations` for modern syntax
   - Use `TypeAlias`, `Protocol`, and `dataclass` over raw dicts
3. **Testing with pytest:**
   - Test files in `tests/` mirroring `src/` structure
   - Use fixtures for shared setup, parametrize for variant testing
   - Name tests: `test_<function>_<scenario>_<expected>`
4. **Code style:**
   - Format with `ruff format`, lint with `ruff check`
   - Prefer dataclasses or Pydantic models over plain dicts
   - Use pathlib over os.path
   - Use f-strings over .format() or % formatting
5. **Error handling:**
   - Raise specific exceptions, not generic `Exception`
   - Use custom exception classes for domain errors
   - Never bare `except:` — always catch specific types

## Examples

**User:** "Set up a new Python project"
**Agent:** Creates `pyproject.toml` with project metadata and dependencies, `src/` layout, `tests/` directory, `ruff` config, and basic `__init__.py`.
