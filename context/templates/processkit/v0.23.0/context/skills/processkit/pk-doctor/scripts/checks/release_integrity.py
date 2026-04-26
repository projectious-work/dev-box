"""release_integrity check — detect git tags missing GitHub Releases.

For every local git tag matching ``v*``, probes GitHub via the `gh`
CLI for a matching Release. Emits WARN for any tag without one.
INFO when `gh` is not installed or not authenticated — the check
remains useful in projects that want it but degrades gracefully
when the tool is absent.

Motivation: a ``git push --tags`` is a ref operation, not a release
artifact. Downstream consumers (package managers, release feeds,
aibox sync, browsers looking at the Releases page) read the Release
not the tag. See DEC-20260422_1348-SnowyWolf.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Optional

from .common import CheckResult


def _gh_available() -> tuple[bool, Optional[str]]:
    """Return (available, reason_if_not)."""
    try:
        proc = subprocess.run(
            ["gh", "--version"],
            capture_output=True, text=True, timeout=5,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False, "gh CLI not installed or not on PATH"
    if proc.returncode != 0:
        return False, "gh CLI present but `gh --version` failed"
    try:
        proc = subprocess.run(
            ["gh", "auth", "status"],
            capture_output=True, text=True, timeout=5,
        )
    except subprocess.TimeoutExpired:
        return False, "gh auth status timed out"
    if proc.returncode != 0:
        return False, "gh CLI not authenticated (run `gh auth login` or set GH_TOKEN)"
    return True, None


def _local_v_tags(repo_root: Path) -> list[str]:
    """Return local git tags matching ``v*``, newest first."""
    try:
        proc = subprocess.run(
            ["git", "tag", "--list", "v*", "--sort=-creatordate"],
            cwd=str(repo_root),
            capture_output=True, text=True, timeout=5,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return []
    if proc.returncode != 0:
        return []
    return [t.strip() for t in proc.stdout.splitlines() if t.strip()]


def _default_repo(repo_root: Path) -> Optional[str]:
    """Extract the GitHub repo slug (<owner>/<repo>) from the origin remote, if any."""
    try:
        proc = subprocess.run(
            ["git", "config", "--get", "remote.origin.url"],
            cwd=str(repo_root),
            capture_output=True, text=True, timeout=5,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    url = proc.stdout.strip()
    if not url:
        return None
    # git@github.com:org/repo.git or https://github.com/org/repo(.git)
    for marker in ("github.com:", "github.com/"):
        if marker in url:
            slug = url.split(marker, 1)[1]
            if slug.endswith(".git"):
                slug = slug[:-4]
            return slug
    return None


def _release_exists(tag: str, repo: Optional[str]) -> Optional[bool]:
    """Return True if a Release matching ``tag`` exists, False if not, None on error."""
    cmd = ["gh", "release", "view", tag, "--json", "tagName"]
    if repo:
        cmd.extend(["--repo", repo])
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    except subprocess.TimeoutExpired:
        return None
    if proc.returncode == 0:
        try:
            data = json.loads(proc.stdout or "{}")
            return data.get("tagName") == tag
        except json.JSONDecodeError:
            return True  # gh succeeded but returned non-JSON; trust the exit code
    stderr = (proc.stderr or "").lower()
    if "release not found" in stderr or "no release" in stderr or "404" in stderr:
        return False
    return None  # transient error — treat as unknown, not as a missing release


def run(ctx) -> list[CheckResult]:
    repo_root: Path = ctx["repo_root"]

    # Allow opt-out via env var for offline CI or sandboxed runs.
    if os.environ.get("PK_DOCTOR_SKIP_RELEASE_INTEGRITY") == "1":
        return [CheckResult(
            severity="INFO",
            category="release_integrity",
            id="release_integrity.skipped-env",
            message="skipped (PK_DOCTOR_SKIP_RELEASE_INTEGRITY=1)",
        )]

    tags = _local_v_tags(repo_root)
    if not tags:
        return [CheckResult(
            severity="INFO",
            category="release_integrity",
            id="release_integrity.no-tags",
            message="no v* tags found locally",
        )]

    ok, reason = _gh_available()
    if not ok:
        return [CheckResult(
            severity="INFO",
            category="release_integrity",
            id="release_integrity.gh-unavailable",
            message=f"gh CLI unavailable — {reason}; cannot verify Release presence for {len(tags)} tag(s)",
        )]

    repo_slug = _default_repo(repo_root)
    results: list[CheckResult] = []
    checked = 0
    missing: list[str] = []
    unknown: list[str] = []
    max_tags = int(os.environ.get("PK_DOCTOR_RELEASE_INTEGRITY_MAX", "50") or "50")

    for tag in tags[:max_tags]:
        exists = _release_exists(tag, repo_slug)
        checked += 1
        if exists is False:
            missing.append(tag)
        elif exists is None:
            unknown.append(tag)

    for tag in missing:
        results.append(CheckResult(
            severity="WARN",
            category="release_integrity",
            id="release_integrity.release-missing",
            message=f"git tag {tag} has no matching GitHub Release; run `gh release create {tag}` to fix",
            entity_ref=tag,
            fixable=False,
            suggested_fix=f"gh release create {tag} --title '{tag}' --notes-file <(awk -v v={tag[1:]} '$0 ~ \"^## \\\\[v\" v \"\\\\]\" {{f=1; next}} f && /^## \\[/ {{f=0}} f' CHANGELOG.md)",
        ))
    for tag in unknown:
        results.append(CheckResult(
            severity="INFO",
            category="release_integrity",
            id="release_integrity.release-unknown",
            message=f"could not determine Release status for {tag} (transient gh error)",
            entity_ref=tag,
        ))

    if not missing and not unknown:
        results.append(CheckResult(
            severity="INFO",
            category="release_integrity",
            id="release_integrity.all-tags-released",
            message=f"verified GitHub Release present for all {checked} local v* tag(s)",
        ))
    return results
