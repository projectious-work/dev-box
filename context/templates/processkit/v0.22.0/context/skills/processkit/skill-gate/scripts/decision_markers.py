"""
decision_markers.py — shared regex library for Rail-5 decision-language detection.

WorkItem: FEAT-20260415_1700-QuietLedger-rail5-auto-decision-capture-implementation

Two marker tiers:

    Tier A  — high-precision commit markers used by Lever 1 (PreToolUse gate).
              Only these trigger the gate.
    Tier B  — softer markers used by Lever 2 (SessionEnd sweeper) for higher-recall
              scanning. Never trigger the gate alone.

Usage
-----
    from decision_markers import scan, MatchResult

    hits = scan("ok let's go with the plan", tier="A")
    # Returns list of MatchResult objects with .pattern, .match, .start, .end
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import List


# ---------------------------------------------------------------------------
# Marker definitions
# ---------------------------------------------------------------------------

# Tier A — high-precision commit markers (Lever 1 gate trigger).
# Ordered from most specific to least specific to reduce ambiguity in output.
_TIER_A_PATTERNS: list[str] = [
    r"\bship\s+it\b",
    r"\blet'?s\s+go\s+with\b",
    r"\bi'?ll\s+go\s+with\b",
    r"\bwe'?ll\s+go\s+with\b",
    r"\bgo\s+with\s+the\b.{0,30}\b(?:approach|option|plan)\b",
    r"\bwe'?ll\b",
    r"\bi'?ll\b",
    r"\bapproved\b",
    r"\bdecided\b",
    r"\bdecision\b",
    r"\bship\s+it\b",
    r"\bproceeed\b",  # common typo variant
    r"\bproceed\b",
    r"\bconfirmed\b",
    r"\bloc(?:ked)?\s+in\b",
    r"\bfinal\s+answer\b",
    r"\bok(?:ay)?\b",
    r"\bgood\b",
    r"\byes\b",
]

# Tier B — softer markers (Lever 2 sweeper only, higher recall).
_TIER_B_PATTERNS: list[str] = [
    r"\bmakes\s+sense\b",
    r"\bright\s+call\b",
    r"\bsounds\s+(?:good|right)\b",
    r"\bi\s+think\b",
    r"\bprobably\b",
    r"\bseems\b",
    r"\bagreed\b",
    r"\bsure\b",
    r"\bwe\s+(?:picked|chose|settled\s+on)\b",
    r"\bwe\s+(?:went|going)\s+with\b",
    r"\blet'?s\b",
    r"\blet\s+('s|us)\b",
]

# Compile once at module load.
_TIER_A_RE: list[re.Pattern[str]] = [
    re.compile(p, re.IGNORECASE) for p in _TIER_A_PATTERNS
]
_TIER_B_RE: list[re.Pattern[str]] = [
    re.compile(p, re.IGNORECASE) for p in _TIER_B_PATTERNS
]

# Deduplicated combined list for "A+B" scans.
_ALL_RE: list[re.Pattern[str]] = _TIER_A_RE + _TIER_B_RE


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


@dataclass
class MatchResult:
    """A single regex hit in the scanned text."""

    pattern: str   # The regex pattern that matched.
    match: str     # The matched substring.
    start: int     # Start character index in the scanned text.
    end: int       # End character index in the scanned text.
    tier: str      # "A" or "B".


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def scan(text: str, tier: str = "A") -> List[MatchResult]:
    """Scan *text* for decision-language markers of the requested *tier*.

    Parameters
    ----------
    text:
        The text to scan (e.g. a user message or assistant turn).
    tier:
        One of:
          "A"    — Tier-A (high-precision) markers only.
          "B"    — Tier-B (soft) markers only.
          "A+B"  — Both tiers (used by the SessionEnd sweeper).

    Returns
    -------
    List of :class:`MatchResult`, one per non-overlapping regex hit.
    Empty list when no markers found.

    Raises
    ------
    ValueError
        If *tier* is not one of the documented values.
    """
    tier = tier.strip().upper()
    if tier == "A":
        patterns = _TIER_A_RE
        tier_label = "A"
    elif tier == "B":
        patterns = _TIER_B_RE
        tier_label = "B"
    elif tier in ("A+B", "AB", "ALL"):
        patterns = _ALL_RE
        tier_label = "A+B"
    else:
        raise ValueError(
            f"tier must be 'A', 'B', or 'A+B'; got {tier!r}"
        )

    results: list[MatchResult] = []
    seen_spans: set[tuple[int, int]] = set()

    for pat in patterns:
        t_label = "A" if pat in _TIER_A_RE else "B"
        for m in pat.finditer(text):
            span = (m.start(), m.end())
            if span in seen_spans:
                continue
            seen_spans.add(span)
            results.append(
                MatchResult(
                    pattern=pat.pattern,
                    match=m.group(0),
                    start=m.start(),
                    end=m.end(),
                    tier=t_label,
                )
            )

    # Sort by position in text.
    results.sort(key=lambda r: r.start)
    return results


def has_tier_a(text: str) -> bool:
    """Return True if *text* contains at least one Tier-A marker."""
    return bool(scan(text, tier="A"))
