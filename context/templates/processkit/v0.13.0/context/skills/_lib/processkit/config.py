"""Read processkit runtime settings from per-skill config files.

Each processkit skill that has configurable behaviour stores its
settings in a ``config/settings.toml`` file inside its own installed
directory under ``context/skills/<name>/config/``. The agent edits
these files during first-time setup (guided by AGENTS.md); the MCP
servers read them on every call.

Skill config locations:

    context/skills/id-management/config/settings.toml
        format     = "word"    # word | uuid
        word_style = "pascal"  # pascal (BoldVale) | camel (boldVale) | kebab (bold-vale)
        slug       = true      # append content-derived slug from entity title

    context/skills/index-management/config/settings.toml
        [directories]
        WorkItem = "workitems"   # override default subdir under context/

        [sharding.LogEntry]
        scheme  = "date"
        pattern = "context/logs/{year}/{month}/"

Legacy fallback: if skill config files are absent, settings are read
from the [context] section of aibox.toml. Defaults apply if neither
source is present.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from . import paths


@dataclass
class Config:
    id_format: str = "word"
    id_slug: bool = False
    id_word_style: str = "kebab"
    id_datetime_prefix: bool = False
    directory_overrides: dict[str, str] = field(default_factory=dict)
    sharding: dict[str, dict[str, Any]] = field(default_factory=dict)
    index_path: str | None = None
    context_budget: dict[str, Any] = field(default_factory=dict)
    grooming_rules: dict[str, Any] = field(default_factory=dict)

    def directory_for(self, kind: str) -> str | None:
        return self.directory_overrides.get(kind)


def _load_toml(path: Path) -> dict | None:
    """Read and parse a TOML file. Returns None on any error."""
    if not path.is_file():
        return None
    try:
        import tomllib  # py 3.11+
    except ModuleNotFoundError:
        try:
            import tomli as tomllib  # type: ignore
        except ModuleNotFoundError:
            return None
    try:
        return tomllib.loads(path.read_text())
    except Exception:
        return None


def _skill_config_dir(root: Path, skill: str) -> Path:
    p = root / "context" / "skills" / "processkit" / skill / "config"
    if p.is_dir():
        return p
    return root / "context" / "skills" / skill / "config"


def load_config(root: Path | None = None) -> Config:
    """Load processkit settings from per-skill config files.

    Not cached — reads from disk on every call so that edits take
    effect immediately without restarting MCP servers.

    Search order per setting group:
    1. context/skills/<skill>/config/settings.toml  (preferred)
    2. aibox.toml [context] section                 (legacy fallback)
    3. Built-in defaults
    """
    root = root or paths.find_project_root()

    # ── ID settings (id-management skill) ────────────────────────────────
    id_cfg = _load_toml(_skill_config_dir(root, "id-management") / "settings.toml")
    if id_cfg is not None:
        id_format = id_cfg.get("format", "word")
        id_slug = bool(id_cfg.get("slug", False))
        id_word_style = id_cfg.get("word_style", "kebab")
        id_datetime_prefix = bool(id_cfg.get("datetime_prefix", False))
    else:
        id_format, id_slug = _legacy_id_settings(root)
        id_word_style = "kebab"
        id_datetime_prefix = False

    # ── Index/directory settings (index-management skill) ─────────────────
    idx_cfg = _load_toml(
        _skill_config_dir(root, "index-management") / "settings.toml"
    )
    if idx_cfg is not None:
        directory_overrides = idx_cfg.get("directories", {})
        sharding = idx_cfg.get("sharding", {})
        index_path = idx_cfg.get("index", {}).get("path")
    else:
        directory_overrides, sharding, index_path = _legacy_index_settings(root)

    return Config(
        id_format=id_format,
        id_slug=id_slug,
        id_word_style=id_word_style,
        id_datetime_prefix=id_datetime_prefix,
        directory_overrides=directory_overrides,
        sharding=sharding,
        index_path=index_path,
    )


# ── Legacy fallback readers (aibox.toml [context]) ────────────────────────────

def _legacy_id_settings(root: Path) -> tuple[str, bool]:
    data = _load_toml(root / "aibox.toml")
    if data is None:
        return "word", False
    ctx = data.get("context", {})
    return ctx.get("id_format", "word"), bool(ctx.get("id_slug", False))


def _legacy_index_settings(root: Path) -> tuple[dict, dict, str | None]:
    data = _load_toml(root / "aibox.toml")
    if data is None:
        return {}, {}, None
    ctx = data.get("context", {})
    return ctx.get("directories", {}), ctx.get("sharding", {}), None
