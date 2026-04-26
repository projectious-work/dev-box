#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
# ]
# ///
"""migrate_models.py — extract models from model_scores.json to first-class artifacts.

Reads the monolithic `model_scores.json` registry and emits one
`MODEL-<provider>-<family>.md` entity artifact per (provider, family),
with versions nested under `spec.versions[]`. Writes both to
`context/models/` and the mirrored `src/context/models/` tree.

The script is idempotent: running it again produces identical output
(modulo metadata.created timestamps, which are preserved if an
existing file already contains a created timestamp).

Usage:

    python3 migrate_models.py [--scores PATH] [--out PATH] [--src-out PATH]
                              [--dry-run] [--verbose]

Exit 0 on success. Exit 1 on error.

Provenance: generated as part of v0.19.0 Phase 3 — DEC-20260422_0234-LoyalComet.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import yaml

# ── constants ──────────────────────────────────────────────────────────────

DEFAULT_CREATED = "2026-04-22T00:00:00Z"

PROVIDER_MAP = {
    "Anthropic": "anthropic",
    "OpenAI": "openai",
    "Google": "google",
    "Google / Open": "google",
    "xAI": "xai",
    "DeepSeek": "deepseek",
    "Meta": "meta",
    "Mistral": "mistral",
    "MiniMax": "minimax",
    "Alibaba / Open": "alibaba",
    "Microsoft": "microsoft",
    "Cohere": "cohere",
}

PROVIDER_STATUS_PAGES = {
    "anthropic": "https://status.anthropic.com/",
    "openai": "https://status.openai.com/",
    "google": "https://status.cloud.google.com/",
    "mistral": "https://status.mistral.ai/",
    "xai": "https://status.x.ai/",
    "deepseek": "https://status.deepseek.com/",
    "meta": "https://ai.meta.com/",
    "minimax": "https://www.minimax.io/",
    "alibaba": "https://status.aliyun.com/",
    "microsoft": "https://azure.status.microsoft/",
    "cohere": "https://status.cohere.com/",
}

# Families where the small/mini/fast/haiku variant should cap effort at medium.
SMALL_FAMILIES = {
    "claude-haiku",
    "gemini-flash",
    "gemini-2-5-flash",
    "gemini-3-flash",
    "gpt-5-mini",
    "o4-mini",
    "grok-mini",
    "gemma",
    "mistral-small",
    "phi",
    "llama",
    "gemma-3",
    "command-r",
    "codestral",
    "qwen2-5-coder",
    "qwen2-5",
}

# Families that are flagship / reasoning-heavy — full effort ladder.
FLAGSHIP_FAMILIES = {
    "claude-opus",
    "claude-sonnet",
    "gpt-5",
    "gpt-5-pro",
    "o3",
    "grok-3",
    "grok-3-5",
    "grok-4",
    "grok-4-1",
    "grok-4-heavy",
    "gemini-2-5-pro",
    "gemini-3-1-pro",
    "deepseek-v",
    "deepseek-r",
    "qwen3",
    "mistral-large",
    "mistral-medium",
    "mistral-deep-think",
    "minimax-text",
    "minimax-m",
}

# ── id-parsing ────────────────────────────────────────────────────────────

# Explicit id → (family, version_id) overrides.
# Keys are original JSON ids; values are derived slugs.
ID_OVERRIDES: dict[str, tuple[str, str]] = {
    "claude-opus-4.6":        ("claude-opus",       "4.6"),
    "claude-sonnet-4.6":      ("claude-sonnet",     "4.6"),
    "claude-haiku-4.5":       ("claude-haiku",      "4.5"),
    "gpt-4o":                 ("gpt-4o",            "1"),
    "o3":                     ("o3",                "1"),
    "o4-mini":                ("o4-mini",           "1"),
    "gpt-5":                  ("gpt-5",             "5"),
    "gpt-5-pro":              ("gpt-5-pro",         "5.4"),
    "gemini-2.5-pro":         ("gemini-2-5-pro",    "2.5"),
    "gemini-2.5-flash":       ("gemini-2-5-flash",  "2.5"),
    "gemini-flash-2.0":       ("gemini-flash",      "2.0"),
    "gemini-3-flash":         ("gemini-3-flash",    "3"),
    "gemini-3.1-pro":         ("gemini-3-1-pro",    "3.1"),
    "llama-3.3-70b":          ("llama-3-70b",       "3.3"),
    "mistral-large-3":        ("mistral-large",     "3"),
    "mistral-medium-3":       ("mistral-medium",    "3"),
    "mistral-small-3":        ("mistral-small",     "3"),
    "mistral-deep-think":     ("mistral-deep-think","1"),
    "codestral":              ("codestral",         "1"),
    "deepseek-v3":            ("deepseek-v",        "3"),
    "deepseek-r1":            ("deepseek-r",        "1"),
    "minimax-text-01":        ("minimax-text",      "01"),
    "minimax-m2.5":           ("minimax-m",         "2.5"),
    "qwen2.5-72b":            ("qwen2-5-72b",       "2.5"),
    "qwen2.5-coder-32b":      ("qwen2-5-coder-32b", "2.5"),
    "qwen3-235b":             ("qwen3-235b",        "3"),
    "phi-4":                  ("phi",               "4"),
    "gemma-3-27b":            ("gemma-3-27b",       "3"),
    "grok-3":                 ("grok-3",            "3"),
    "grok-3.5":               ("grok-3-5",          "3.5"),
    "grok-4":                 ("grok-4",            "4"),
    "grok-4.1":               ("grok-4-1",          "4.1"),
    "grok-4-heavy":           ("grok-4-heavy",      "4"),
    "command-r-plus":         ("command-r-plus",    "1"),
}


def parse_id(model_id: str) -> tuple[str, str]:
    """Return (family, version_id) for a raw JSON id.

    Falls back to best-effort parsing for unknown ids: family = model_id
    with any trailing "-X.Y" / "-X" version suffix stripped; version
    = the stripped suffix (or "1" if absent).
    """
    if model_id in ID_OVERRIDES:
        return ID_OVERRIDES[model_id]
    # Fallback: strip a trailing -<digits.digits?> if present.
    m = re.match(r"^(.*?)[-](\d+(?:\.\d+)?)$", model_id)
    if m:
        return m.group(1), m.group(2)
    return model_id, "1"


# ── mapping helpers ────────────────────────────────────────────────────────

def equivalent_tier(reasoning: int, engineering: int) -> str:
    avg = (reasoning + engineering) / 2.0
    if avg >= 4.5:
        return "xxl"
    if avg >= 4.0:
        return "xl"
    if avg >= 3.5:
        return "l"
    if avg >= 3.0:
        return "m"
    if avg >= 2.5:
        return "s"
    return "xs"


def efforts_for_family(family: str) -> list[str]:
    if family in SMALL_FAMILIES:
        return ["none", "low", "medium"]
    if family in FLAGSHIP_FAMILIES:
        return ["none", "low", "medium", "high", "extra-high", "max"]
    return ["none", "low", "medium", "high", "extra-high"]


def modalities_for(entry: dict) -> list[str]:
    """Infer modalities from sub-dimension breadth scores."""
    sub_b = entry["dimensions"]["B"].get("sub", {})
    mods = ["text"]
    if sub_b.get("vision", 1) >= 3:
        mods.append("vision")
    if sub_b.get("audio", 1) >= 3:
        mods.append("audio")
    # Tool-use proxy via E.sub.tool_use
    sub_e = entry["dimensions"]["E"].get("sub", {})
    if sub_e.get("tool_use", 1) >= 3:
        mods.append("tools")
    # computer-use: Claude Opus/Sonnet flagship only, not in JSON sub-dims; add by family later
    return mods


def access_tier_for(entry: dict) -> str:
    """Derive access_tier from governance + hosting note."""
    hosting = entry.get("pricing", {}).get("hosting", "api") or "api"
    if "self-hosted" in hosting or "azure-or-self-hosted" in hosting:
        # Open weights / self-hostable are "public" (anyone can obtain).
        return "public"
    return "public"


def context_window_tokens(context_k: int | None) -> int | None:
    if context_k is None:
        return None
    return int(context_k) * 1000


def pricing_dict(entry: dict) -> dict | None:
    p = entry.get("pricing", {})
    inp = p.get("input_per_1m")
    out = p.get("output_per_1m")
    if inp is None and out is None:
        return None
    res: dict[str, float] = {}
    if inp is not None:
        res["input"] = inp
    if out is not None:
        res["output"] = out
    return res


def status_for(entry: dict) -> str:
    """Map JSON entry to a version status. Estimated → preview, else ga."""
    if entry.get("_estimated"):
        return "preview"
    return "ga"


# ── artifact construction ─────────────────────────────────────────────────

def build_version(entry: dict, version_id: str) -> dict:
    v: dict[str, Any] = {
        "version_id": str(version_id),
        "status": status_for(entry),
    }
    cw = context_window_tokens(entry.get("context_k"))
    if cw is not None:
        v["context_window"] = cw
    pricing = pricing_dict(entry)
    if pricing is not None:
        v["pricing_usd_per_1m"] = pricing
    note = entry.get("pricing", {}).get("note")
    if note:
        v["pricing_note"] = note
    gov_warn = entry.get("governance_warning")
    if gov_warn:
        v["governance_warning"] = gov_warn
    return v


def build_entity(provider: str, family: str, entries: list[dict]) -> dict:
    """Build the Model entity dict for (provider, family) from its JSON entries."""
    # Use the first entry for capability dimensions (they should be ~same
    # within a family; if they drift, the newest entry wins — entries come
    # sorted newest-first after grouping).
    primary = entries[0]
    dims = primary["dimensions"]
    dimensions = {
        "reasoning":   dims["R"]["score"],
        "engineering": dims["E"]["score"],
        "speed":       dims["S"]["score"],
        "breadth":     dims["B"]["score"],
        "reliability": dims["L"]["score"],
        "governance":  dims["G"]["score"],
    }

    versions: list[dict] = []
    for entry in entries:
        _fam, vid = parse_id(entry["id"])
        versions.append(build_version(entry, vid))

    modalities = modalities_for(primary)
    # Add computer-use for Anthropic flagships.
    if provider == "anthropic" and family in ("claude-opus", "claude-sonnet"):
        if "computer-use" not in modalities:
            modalities.append("computer-use")

    entity: dict[str, Any] = {
        "apiVersion": "processkit.projectious.work/v1",
        "kind": "Model",
        "metadata": {
            "id": f"MODEL-{provider}-{family}",
            "created": DEFAULT_CREATED,
        },
        "spec": {
            "provider": provider,
            "family": family,
            "versions": versions,
            "efforts_supported": efforts_for_family(family),
            "dimensions": dimensions,
            "modalities": modalities,
            "access_tier": access_tier_for(primary),
            "equivalent_tier": equivalent_tier(
                dimensions["reasoning"], dimensions["engineering"]
            ),
            "status_page_url": PROVIDER_STATUS_PAGES.get(provider, ""),
            "rationale": primary.get("tier", ""),
        },
    }
    return entity


def render_markdown(entity: dict) -> str:
    """Render entity dict → entity markdown (YAML frontmatter + empty body)."""
    yaml_block = yaml.safe_dump(
        entity,
        sort_keys=False,
        default_flow_style=False,
        allow_unicode=True,
    )
    return f"---\n{yaml_block}---\n"


# ── main migration ────────────────────────────────────────────────────────

def group_entries(entries: list[dict]) -> dict[tuple[str, str], list[dict]]:
    """Group JSON entries by (provider_slug, family)."""
    grouped: dict[tuple[str, str], list[dict]] = {}
    unmapped: list[str] = []
    for entry in entries:
        prov_raw = entry.get("provider", "")
        provider = PROVIDER_MAP.get(prov_raw)
        if not provider:
            unmapped.append(f"{entry['id']} (provider={prov_raw!r})")
            continue
        family, _vid = parse_id(entry["id"])
        grouped.setdefault((provider, family), []).append(entry)
    # Sort versions within each family — newest first by version_id lex.
    for key in grouped:
        grouped[key].sort(key=lambda e: parse_id(e["id"])[1], reverse=True)
    if unmapped:
        print(f"[warn] unmapped providers: {unmapped}", file=sys.stderr)
    return grouped


def migrate(
    scores_path: Path,
    out_path: Path,
    src_out_path: Path,
    dry_run: bool = False,
    verbose: bool = False,
) -> dict:
    """Run the migration. Returns summary dict."""
    data = json.loads(scores_path.read_text())
    entries = data["models"]
    grouped = group_entries(entries)

    if not dry_run:
        out_path.mkdir(parents=True, exist_ok=True)
        src_out_path.mkdir(parents=True, exist_ok=True)

    files_written: list[str] = []
    tier_counts: dict[str, int] = {}

    for (provider, family), fam_entries in sorted(grouped.items()):
        entity = build_entity(provider, family, fam_entries)
        md = render_markdown(entity)
        rel = f"MODEL-{provider}-{family}.md"

        tier = entity["spec"]["equivalent_tier"]
        tier_counts[tier] = tier_counts.get(tier, 0) + 1

        primary_target = out_path / rel
        mirror_target = src_out_path / rel

        if dry_run:
            if verbose:
                print(f"[dry-run] would write {primary_target}")
                print(f"[dry-run] would write {mirror_target}")
        else:
            # Preserve existing created timestamp (verbatim, quoting and all)
            # for byte-exact idempotency.
            for tgt in (primary_target, mirror_target):
                md_for_this = md
                if tgt.exists():
                    existing = tgt.read_text()
                    mcreated = re.search(
                        r"^(\s*created:\s*)(\S.*)$", existing, re.MULTILINE
                    )
                    if mcreated:
                        # Keep the exact trailing token (including quotes).
                        replacement_line = mcreated.group(1) + mcreated.group(2)
                        md_for_this = re.sub(
                            r"^\s*created:\s*\S.*$",
                            replacement_line,
                            md,
                            count=1,
                            flags=re.MULTILINE,
                        )
                tgt.write_text(md_for_this)
            files_written.append(rel)
            if verbose:
                print(f"[ok] {primary_target}")

    return {
        "json_models": len(entries),
        "families_written": len(grouped),
        "files_written": files_written,
        "tier_counts": tier_counts,
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    repo_root = Path(__file__).resolve().parents[5]
    default_scores = (
        repo_root
        / "context/skills/processkit/model-recommender/mcp/model_scores.json"
    )
    ap.add_argument("--scores", type=Path, default=default_scores)
    ap.add_argument("--out", type=Path, default=repo_root / "context/models")
    ap.add_argument(
        "--src-out", type=Path, default=repo_root / "src/context/models"
    )
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args(argv)

    if not args.scores.exists():
        print(f"[err] scores file not found: {args.scores}", file=sys.stderr)
        return 1

    summary = migrate(
        args.scores, args.out, args.src_out,
        dry_run=args.dry_run, verbose=args.verbose,
    )
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
