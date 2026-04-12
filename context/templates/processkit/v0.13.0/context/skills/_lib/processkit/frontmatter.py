"""YAML frontmatter parsing for entity files.

A processkit entity file has the shape:

    ---
    <YAML>
    ---

    <Markdown body, optional>

This module splits and joins those two halves. It does NOT validate the
YAML against a schema — that is `processkit.schema`'s job.
"""
from __future__ import annotations

import re
from typing import Any

import yaml

_FRONTMATTER_RE = re.compile(
    r"\A---\s*\n(?P<yaml>.*?)\n---\s*(?:\n(?P<body>.*))?\Z",
    re.DOTALL,
)


class FrontmatterError(ValueError):
    """Raised when an entity file does not have parseable frontmatter."""


def parse(text: str) -> tuple[dict[str, Any], str]:
    """Split text into (frontmatter dict, body string).

    Raises FrontmatterError if the text does not start with a YAML
    frontmatter block delimited by ``---`` lines.
    """
    match = _FRONTMATTER_RE.match(text)
    if not match:
        raise FrontmatterError("file does not start with a YAML frontmatter block")
    yaml_text = match.group("yaml")
    body = match.group("body") or ""
    try:
        data = yaml.safe_load(yaml_text)
    except yaml.YAMLError as e:
        raise FrontmatterError(f"invalid YAML in frontmatter: {e}") from e
    if not isinstance(data, dict):
        raise FrontmatterError(
            f"frontmatter must be a YAML mapping at the top level, got {type(data).__name__}"
        )
    return data, body


def render(data: dict[str, Any], body: str = "") -> str:
    """Render a frontmatter dict + body into a complete entity file string.

    The YAML is dumped with stable key ordering and block style. The body
    is appended verbatim with a single blank line separator.
    """
    yaml_text = yaml.safe_dump(
        data,
        sort_keys=False,
        default_flow_style=False,
        allow_unicode=True,
    ).rstrip("\n")
    body = body.lstrip("\n")
    if body:
        return f"---\n{yaml_text}\n---\n\n{body.rstrip()}\n"
    return f"---\n{yaml_text}\n---\n"
