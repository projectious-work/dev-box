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


class _BlockStyleDumper(yaml.SafeDumper):
    """SafeDumper that emits multi-line strings as `|` literal block scalars.

    Markdown bodies in entity descriptions routinely contain pipe-tables,
    fenced code blocks, and backslashes. Default PyYAML serialization picks
    double-quoted style for any string with newlines, which then escapes
    those characters in ways PyYAML itself can refuse to round-trip
    (BACK-20260425_1755-CalmArch). Literal block scalars are unescaped and
    round-trip safely.
    """


def _str_representer(dumper: yaml.SafeDumper, data: str) -> yaml.ScalarNode:
    if "\n" not in data:
        return dumper.represent_scalar("tag:yaml.org,2002:str", data)
    # Normalize trailing whitespace so the YAML clip (`|`) chomping indicator
    # — which preserves exactly one trailing newline on round-trip — gives
    # exact equality back. PyYAML does not let us pass `|+` / `|-` directly
    # via `style=`, so we shape the data instead. A single trailing newline
    # is canonical for a multi-line text block.
    canonical = data.rstrip("\n") + "\n"
    return dumper.represent_scalar("tag:yaml.org,2002:str", canonical, style="|")


_BlockStyleDumper.add_representer(str, _str_representer)


def parse(text: str) -> tuple[dict[str, Any], str]:
    """Split text into (frontmatter dict, body string).

    Raises FrontmatterError if the text does not start with a YAML
    frontmatter block delimited by ``---`` lines.
    """
    match = _FRONTMATTER_RE.match(text)
    if not match:
        raise FrontmatterError("file does not start with a YAML frontmatter block")
    # Re-attach the trailing newline that the regex's `\n---` boundary
    # consumes — without it, a `|` block scalar at the end of frontmatter
    # loses its final newline on the round-trip (CalmArch).
    yaml_text = match.group("yaml") + "\n"
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
    yaml_text = yaml.dump(
        data,
        Dumper=_BlockStyleDumper,
        sort_keys=False,
        default_flow_style=False,
        allow_unicode=True,
    ).rstrip("\n")
    body = body.lstrip("\n")
    if body:
        return f"---\n{yaml_text}\n---\n\n{body.rstrip()}\n"
    return f"---\n{yaml_text}\n---\n"
