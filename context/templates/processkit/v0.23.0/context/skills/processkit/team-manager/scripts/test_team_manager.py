"""Tests for team-manager skill.

Run with:

    uv run --with pyyaml --with jsonschema --with pytest --with mcp \
        pytest context/skills/processkit/team-manager/scripts/test_team_manager.py -v

Covers:
  - create/get/update/deactivate/reactivate roundtrip
  - list with all filter combos
  - reserve/release/suggest/list_available_names
  - off-pool rejection for ai-agent
  - pool exhaustion
  - init_memory_tree creates all expected subdirs
  - Each of the 10 consistency checks: passing + failing case
  - export + import roundtrip preserves expected structure
  - export redacts confidential/pii files
  - export excludes journal/, relations/, private/
"""
from __future__ import annotations

import json
import os
import shutil
import sys
from pathlib import Path

import pytest

# --- Path bootstrap so we can import scripts and the processkit lib -----------
_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

# Find the processkit lib (mirrors server.py's _find_lib).
def _find_lib() -> Path:
    here = _SCRIPTS_DIR
    while True:
        for c in (here / "src" / "lib", here / "_lib"):
            if (c / "processkit" / "__init__.py").is_file():
                return c
        if here.parent == here:
            raise RuntimeError("processkit lib not found")
        here = here.parent


_LIB = _find_lib()
if str(_LIB) not in sys.path:
    sys.path.insert(0, str(_LIB))

import consistency  # noqa: E402
import export_import  # noqa: E402
import memory_tree  # noqa: E402
import name_pool  # noqa: E402
from processkit import entity as _entity_mod  # noqa: E402


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture()
def project_root(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Create a throwaway project root with AGENTS.md + schemas symlinked in."""
    (tmp_path / "AGENTS.md").write_text("# test project\n")
    (tmp_path / "context").mkdir()

    # Mirror just the team-member schema into the consumer layout so
    # processkit.schema.load_schema("TeamMember") works.
    schemas_dir = tmp_path / "context" / "schemas"
    schemas_dir.mkdir()
    src_schema = _find_repo_root() / "context" / "schemas" / "team-member.yaml"
    shutil.copy(src_schema, schemas_dir / "team-member.yaml")
    # Also copy role schema (for completeness)
    role_schema = _find_repo_root() / "context" / "schemas" / "role.yaml"
    if role_schema.is_file():
        shutil.copy(role_schema, schemas_dir / "role.yaml")

    monkeypatch.chdir(tmp_path)
    # Force processkit.paths.find_project_root to discover AGENTS.md in tmp_path
    # (it walks from cwd).
    # Invalidate schema LRU cache so the test schema is reloaded fresh.
    from processkit import schema as _schema_mod
    _schema_mod.load_schema.cache_clear()
    return tmp_path


def _find_repo_root() -> Path:
    here = Path(__file__).resolve()
    for ancestor in [here, *here.parents]:
        if (ancestor / "AGENTS.md").is_file() and (ancestor / "src").is_dir():
            return ancestor
    raise RuntimeError("repo root not found")


@pytest.fixture()
def pool_path(tmp_path: Path) -> Path:
    """Copy a fresh name-pool.yaml into tmp_path/data/ so tests mutate a throwaway file."""
    src = Path(__file__).resolve().parent.parent / "data" / "name-pool.yaml"
    dest = tmp_path / "data" / "name-pool.yaml"
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src, dest)
    return dest


@pytest.fixture()
def assets_dir() -> Path:
    return Path(__file__).resolve().parent.parent / "assets"


@pytest.fixture()
def server_mod(project_root: Path):
    """Import the MCP server fresh so its tool callables bind to project_root.

    The module caches FastMCP state at import time, so we reset sys.modules
    and re-import for each test to keep them isolated.
    """
    for mod in list(sys.modules):
        if mod in {"server", "consistency", "memory_tree", "name_pool", "export_import"}:
            # Keep helper modules; only reload server.
            if mod == "server":
                del sys.modules[mod]
    mcp_dir = Path(__file__).resolve().parent.parent / "mcp"
    if str(mcp_dir) not in sys.path:
        sys.path.insert(0, str(mcp_dir))
    import server  # noqa: F401
    # Point the skill's pool_path helper at a throwaway pool for create_team_member
    pool_src = Path(__file__).resolve().parent.parent / "data" / "name-pool.yaml"
    pool_copy = project_root / "pool.yaml"
    shutil.copy(pool_src, pool_copy)
    server._pool_path = lambda: pool_copy  # type: ignore[attr-defined]
    return server


# ---------------------------------------------------------------------------
# name_pool
# ---------------------------------------------------------------------------

def test_name_pool_list_available_all(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    names = name_pool.available(pool)
    assert len(names) > 50
    assert "Alice" in names


def test_name_pool_filter_by_kind(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    fem = name_pool.available(pool, "feminine")
    masc = name_pool.available(pool, "masculine")
    neut = name_pool.available(pool, "neutral")
    assert "Alice" in fem and "Alice" not in masc
    assert "Adam" in masc and "Adam" not in fem
    assert "Kai" in neut and "Kai" not in fem


def test_name_pool_reserve_release(pool_path: Path):
    name_pool.reserve(pool_path, "Alice", "alice-chen")
    pool = name_pool.load_pool(pool_path)
    assert name_pool.is_reserved(pool, "Alice") == "alice-chen"
    assert "Alice" not in name_pool.available(pool)

    name_pool.release(pool_path, "Alice")
    pool = name_pool.load_pool(pool_path)
    assert name_pool.is_reserved(pool, "Alice") is None
    assert "Alice" in name_pool.available(pool)


def test_name_pool_reserve_not_in_pool(pool_path: Path):
    with pytest.raises(name_pool.NamePoolError):
        name_pool.reserve(pool_path, "Zorblax", "x")


def test_name_pool_double_reserve_conflict(pool_path: Path):
    name_pool.reserve(pool_path, "Alice", "alice-a")
    with pytest.raises(name_pool.NamePoolError):
        name_pool.reserve(pool_path, "Alice", "alice-b")


def test_name_pool_reserve_same_slug_idempotent(pool_path: Path):
    name_pool.reserve(pool_path, "Alice", "alice-a")
    name_pool.reserve(pool_path, "Alice", "alice-a")


def test_name_pool_suggest(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    n = name_pool.suggest(pool, "feminine")
    assert n in name_pool.available(pool, "feminine")


def test_name_pool_exhaustion(pool_path: Path):
    pool = name_pool.load_pool(pool_path)
    fem = name_pool.available(pool, "feminine")
    for idx, n in enumerate(fem):
        name_pool.reserve(pool_path, n, f"slug-{idx}")
    pool = name_pool.load_pool(pool_path)
    assert name_pool.suggest(pool, "feminine") is None
    assert name_pool.available(pool, "feminine") == []


def test_name_pool_release_unreserved_is_noop(pool_path: Path):
    name_pool.release(pool_path, "Alice")  # never reserved — should not raise


# ---------------------------------------------------------------------------
# memory_tree
# ---------------------------------------------------------------------------

def test_init_memory_tree_creates_subdirs(tmp_path: Path):
    tm_dir = tmp_path / "atlas"
    created = memory_tree.init_tree(tm_dir, "atlas")
    for tier in memory_tree.TIER_SUBDIRS + ("private",):
        assert (tm_dir / tier).is_dir()
        assert (tm_dir / tier / ".gitkeep").is_file()
    assert (tm_dir / "persona.md").is_file()
    assert (tm_dir / "card.json").is_file()
    # Idempotent second call
    created2 = memory_tree.init_tree(tm_dir, "atlas")
    assert created2 == []


def test_init_memory_tree_uses_entity_name_and_role(tmp_path: Path):
    tm_dir = tmp_path / "atlas"
    ent = _entity_mod.new(
        "TeamMember",
        "TEAMMEMBER-atlas",
        {
            "type": "ai-agent",
            "name": "Atlas",
            "slug": "atlas",
            "default_role": "ROLE-architect",
            "default_seniority": "expert",
            "active": True,
        },
    )
    memory_tree.init_tree(tm_dir, "atlas", ent)
    card = json.loads((tm_dir / "card.json").read_text())
    assert card["name"] == "Atlas"
    assert card["role"] == "ROLE-architect"
    assert card["seniority"] == "expert"


# ---------------------------------------------------------------------------
# Lifecycle (via server module)
# ---------------------------------------------------------------------------

def test_create_human_roundtrip(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        email="alice@example.com",
        default_role="ROLE-developer",
        default_seniority="senior",
    )
    assert r.get("id") == "TEAMMEMBER-alice-chen"
    assert Path(r["path"]).is_file()

    got = server_mod.get_team_member("alice-chen")
    assert got["name"] == "Alice Chen"
    assert got["type"] == "human"
    assert got["slug"] == "alice-chen"
    assert got["email"] == "alice@example.com"
    assert got["default_seniority"] == "senior"


def test_create_ai_agent_from_pool_auto_reserves(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
        default_role="ROLE-architect",
    )
    assert "error" not in r, r
    pool = name_pool.load_pool(server_mod._pool_path())
    assert name_pool.is_reserved(pool, "Alice") == "alice"


def test_create_ai_agent_off_pool_rejected(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Atlas", type="ai-agent", slug="atlas",
    )
    assert "error" in r
    assert "name pool" in r["error"]


def test_create_ai_agent_in_pool(server_mod, project_root: Path):
    r = server_mod.create_team_member(
        name="Alice", type="ai-agent", slug="alice",
    )
    assert r.get("id") == "TEAMMEMBER-alice", r
    pool = name_pool.load_pool(server_mod._pool_path())
    assert name_pool.is_reserved(pool, "Alice") == "alice"


def test_create_invalid_type_rejected(server_mod):
    r = server_mod.create_team_member(name="x", type="bogus", slug="x")
    assert "error" in r


def test_create_invalid_slug_rejected(server_mod):
    r = server_mod.create_team_member(name="x", type="human", slug="Not-Valid!")
    assert "error" in r


def test_create_duplicate_slug_rejected(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    assert "error" in r
    assert "already exists" in r["error"]


def test_update_team_member(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.update_team_member(
        "alice-chen", email="a@b.com", handle="alice", default_seniority="expert"
    )
    assert r.get("ok") is True
    assert set(r["updated"]) == {"email", "handle", "default_seniority"}
    got = server_mod.get_team_member("alice-chen")
    assert got["email"] == "a@b.com"
    assert got["handle"] == "alice"
    assert got["default_seniority"] == "expert"


def test_deactivate_reactivate(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.deactivate_team_member("alice-chen")
    assert r.get("ok") is True
    got = server_mod.get_team_member("alice-chen")
    assert got["active"] is False
    assert got["left_at"]
    r2 = server_mod.reactivate_team_member("alice-chen")
    assert r2.get("ok") is True
    got = server_mod.get_team_member("alice-chen")
    assert got["active"] is True
    assert got["left_at"] is None


def test_list_team_members_filters(server_mod):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Bob Lee", type="human", slug="bob-lee",
                                  default_role="ROLE-developer")
    server_mod.create_team_member(name="Alice", type="ai-agent", slug="alice")
    server_mod.deactivate_team_member("bob-lee")

    all_active = server_mod.list_team_members()
    slugs = {m["slug"] for m in all_active}
    assert "alice-chen" in slugs
    assert "alice" in slugs
    assert "bob-lee" not in slugs

    all_including_inactive = server_mod.list_team_members(active_only=False)
    assert "bob-lee" in {m["slug"] for m in all_including_inactive}

    humans = server_mod.list_team_members(type="human")
    assert {m["type"] for m in humans} == {"human"}

    agents = server_mod.list_team_members(type="ai-agent")
    assert {m["type"] for m in agents} == {"ai-agent"}

    by_role = server_mod.list_team_members(active_only=False, role="ROLE-developer")
    assert [m["slug"] for m in by_role] == ["bob-lee"]


def test_get_nonexistent(server_mod):
    r = server_mod.get_team_member("nobody")
    assert "error" in r


# ---------------------------------------------------------------------------
# init_memory_tree tool
# ---------------------------------------------------------------------------

def test_init_memory_tree_tool(server_mod, project_root: Path):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    r = server_mod.init_memory_tree("alice-chen")
    p = Path(r["path"])
    for tier in memory_tree.TIER_SUBDIRS:
        assert (p / tier).is_dir()
        assert (p / tier / ".gitkeep").is_file()


# ---------------------------------------------------------------------------
# Name pool tools
# ---------------------------------------------------------------------------

def test_suggest_and_reserve_tool(server_mod):
    r = server_mod.suggest_name("feminine")
    assert r["kind"] == "feminine"
    name = r["name"]
    s = server_mod.reserve_name(name, f"{name.lower()}-01")
    assert s["ok"] is True
    avail = server_mod.list_available_names("feminine")
    assert name not in avail
    rr = server_mod.release_name(name)
    assert rr["ok"] is True
    assert name in server_mod.list_available_names("feminine")


# ---------------------------------------------------------------------------
# Consistency checks — 10 codes, one passing + one failing each
# ---------------------------------------------------------------------------

def _setup_alice(server_mod, project_root: Path):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role=None, default_seniority="specialist",
    )
    server_mod.init_memory_tree("alice-chen")
    return project_root / "context" / "team-members" / "alice-chen"


def _check_codes(findings, code):
    return [f for f in findings if f["code"] == code]


def test_check_schema_drift_pass(server_mod, project_root: Path, pool_path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.schema") == []


def test_check_schema_drift_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # Break the entity: remove required 'type' field
    tm_md = tm_dir / "team-member.md"
    ent = _entity_mod.load(tm_md)
    ent.spec.pop("type")
    ent.write()
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.schema")


def test_check_tier_missing_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.tier_missing") == []


def test_check_tier_missing_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    shutil.rmtree(tm_dir / "knowledge")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.tier_missing")


def test_check_dangling_ref_fail(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-ghost",
    )
    tm_dir = project_root / "context" / "team-members" / "alice-chen"
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.dangling_ref")


def test_check_dangling_ref_pass(server_mod, project_root: Path, assets_dir):
    # Create a role file so the reference resolves
    roles_dir = project_root / "context" / "roles"
    roles_dir.mkdir(parents=True, exist_ok=True)
    (roles_dir / "ROLE-developer.md").write_text(
        "---\n"
        "apiVersion: processkit.projectious.work/v1\n"
        "kind: Role\n"
        "metadata:\n"
        "  id: ROLE-developer\n"
        "  created: 2026-04-22T00:00:00+00:00\n"
        "spec:\n"
        "  name: developer\n"
        "---\n"
    )
    server_mod.create_team_member(
        name="Alice Chen", type="human", slug="alice-chen",
        default_role="ROLE-developer",
    )
    tm_dir = project_root / "context" / "team-members" / "alice-chen"
    server_mod.init_memory_tree("alice-chen")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.dangling_ref") == []


def test_check_name_collision(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-a")
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-b")
    server_mod.init_memory_tree("alice-a")
    server_mod.init_memory_tree("alice-b")
    report = consistency.check_all(
        project_root,
        project_root / "context" / "team-members",
        server_mod._pool_path(),
        assets_dir,
    )
    collisions = [
        f for findings in report["members"].values()
        for f in findings
        if f["code"] == "team.name_collision"
    ]
    assert collisions


def test_check_name_collision_pass(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Bob Lee", type="human", slug="bob-lee")
    server_mod.init_memory_tree("alice-chen")
    server_mod.init_memory_tree("bob-lee")
    report = consistency.check_all(
        project_root,
        project_root / "context" / "team-members",
        server_mod._pool_path(),
        assets_dir,
    )
    collisions = [
        f for findings in report["members"].values()
        for f in findings
        if f["code"] == "team.name_collision"
    ]
    assert collisions == []


def test_check_name_off_pool_fail(server_mod, project_root: Path, assets_dir):
    # Hand-craft an ai-agent entity with an off-pool name, bypassing create_team_member.
    tm_dir = project_root / "context" / "team-members" / "zz"
    tm_dir.mkdir(parents=True)
    memory_tree.init_tree(tm_dir, "zz")
    ent = _entity_mod.new(
        "TeamMember",
        "TEAMMEMBER-zz",
        {"type": "ai-agent", "name": "Zzz", "slug": "zz", "active": True},
    )
    ent.write(tm_dir / "team-member.md")
    findings = consistency.check_one(project_root, tm_dir, "zz",
                                     server_mod._pool_path(), assets_dir)
    off = _check_codes(findings, "team.name.off_pool")
    assert off


def test_check_name_off_pool_pass(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice", type="ai-agent", slug="alice")
    tm_dir = project_root / "context" / "team-members" / "alice"
    server_mod.init_memory_tree("alice")
    findings = consistency.check_one(project_root, tm_dir, "alice",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.name.off_pool") == []


def test_check_orphan_file_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    (tm_dir / "random.txt").write_text("stray")
    (tm_dir / "stray_dir").mkdir()
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    orphans = _check_codes(findings, "team.drift.orphan_file")
    assert any("random.txt" in f["path"] for f in orphans)
    assert any("stray_dir" in f["path"] for f in orphans)


def test_check_orphan_file_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.drift.orphan_file") == []


def test_check_leak_risk_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    bad = tm_dir / "knowledge" / "secrets.md"
    bad.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: confidential\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "secret\n"
    )
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.sensitivity.leak_risk")


def test_check_leak_risk_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    ok = tm_dir / "knowledge" / "stack.md"
    ok.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: public\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "ok\n"
    )
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.sensitivity.leak_risk") == []


def test_check_private_not_ignored_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # No .gitignore at all → expect warning
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.private.not_ignored")


def test_check_private_ignored_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    (project_root / ".gitignore").write_text("**/private/\n")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.private.not_ignored") == []


def test_check_memory_bad_header_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    bad = tm_dir / "knowledge" / "note.md"
    bad.write_text("no frontmatter at all\n")
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.memory.bad_header")


def test_check_memory_bad_header_pass(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    good = tm_dir / "knowledge" / "note.md"
    good.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: public\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "body\n"
    )
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.memory.bad_header") == []


def test_check_card_stale_fail(server_mod, project_root: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    card = tm_dir / "card.json"
    card.write_text(json.dumps({
        "schemaVersion": "a2a/v0.3",
        "name": "WRONG",
        "role": "ROLE-other",
        "seniority": "junior",
        "skills": [],
        "signature": {"alg": "none", "value": ""},
    }, indent=2))
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.card.stale")


def test_check_card_stale_pass(server_mod, project_root: Path, assets_dir):
    # Freshly-initted card matches entity — should pass.
    tm_dir = _setup_alice(server_mod, project_root)
    findings = consistency.check_one(project_root, tm_dir, "alice-chen",
                                     server_mod._pool_path(), assets_dir)
    assert _check_codes(findings, "team.card.stale") == []


# ---------------------------------------------------------------------------
# Export / Import
# ---------------------------------------------------------------------------

def test_export_roundtrip_excludes_journal_relations_private(
    server_mod, project_root: Path, tmp_path: Path
):
    tm_dir = _setup_alice(server_mod, project_root)
    # Populate each tier with a file
    for tier in ("knowledge", "skills", "lessons", "journal", "relations"):
        f = tm_dir / tier / "note.md"
        f.write_text(
            "---\n"
            f"tier: {tier if tier != 'knowledge' else 'semantic'}\n"
            "source: conversation\n"
            "sensitivity: public\n"
            "created: 2026-04-22T00:00:00+00:00\n"
            "---\n"
            f"body {tier}\n"
        )
    (tm_dir / "private" / "secret.md").write_text("secret\n")

    out = tmp_path / "bundle.tar.gz"
    summary = export_import.export(tm_dir, out)
    assert out.is_file()

    # Inspect contents
    import tarfile
    with tarfile.open(out) as tar:
        names = tar.getnames()
    # Should include knowledge/, skills/, lessons/ and top-level files
    assert any("alice-chen/knowledge/note.md" in n for n in names)
    assert any("alice-chen/skills/note.md" in n for n in names)
    assert any("alice-chen/lessons/note.md" in n for n in names)
    # Should NOT include journal/, relations/, private/
    assert not any("alice-chen/journal/" in n for n in names), names
    assert not any("alice-chen/relations/" in n for n in names), names
    assert not any("alice-chen/private/" in n for n in names), names


def test_export_redacts_confidential(server_mod, project_root: Path, tmp_path: Path):
    tm_dir = _setup_alice(server_mod, project_root)
    public = tm_dir / "knowledge" / "public.md"
    public.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: public\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "public\n"
    )
    confidential = tm_dir / "knowledge" / "secret.md"
    confidential.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: confidential\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "hush\n"
    )
    pii = tm_dir / "knowledge" / "pii.md"
    pii.write_text(
        "---\n"
        "tier: semantic\n"
        "source: conversation\n"
        "sensitivity: pii\n"
        "created: 2026-04-22T00:00:00+00:00\n"
        "---\n"
        "1234\n"
    )

    out = tmp_path / "bundle.tar.gz"
    summary = export_import.export(tm_dir, out)
    import tarfile
    with tarfile.open(out) as tar:
        names = tar.getnames()
    assert any("knowledge/public.md" in n for n in names)
    assert not any("knowledge/secret.md" in n for n in names)
    assert not any("knowledge/pii.md" in n for n in names)
    assert any("secret.md" in r for r in summary["redacted"])
    assert any("pii.md" in r for r in summary["redacted"])


def test_import_roundtrip(server_mod, project_root: Path, tmp_path: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # Ensure the card has a signature field
    card = json.loads((tm_dir / "card.json").read_text())
    card["signature"] = {"alg": "none", "value": ""}
    (tm_dir / "card.json").write_text(json.dumps(card, indent=2))

    out = tmp_path / "bundle.tar.gz"
    export_import.export(tm_dir, out)

    # Remove the original so import into the same project succeeds
    shutil.rmtree(tm_dir)
    assert not (project_root / "context" / "team-members" / "alice-chen").exists()

    result = export_import.import_bundle(
        out, project_root / "context" / "team-members", assets_dir,
    )
    assert result["slug"] == "alice-chen"
    assert (project_root / "context" / "team-members" / "alice-chen" / "team-member.md").is_file()


def test_import_rejects_missing_signature(server_mod, project_root: Path, tmp_path: Path, assets_dir):
    tm_dir = _setup_alice(server_mod, project_root)
    # Strip signature field
    card = json.loads((tm_dir / "card.json").read_text())
    card.pop("signature", None)
    (tm_dir / "card.json").write_text(json.dumps(card, indent=2))

    out = tmp_path / "bundle.tar.gz"
    export_import.export(tm_dir, out)
    shutil.rmtree(tm_dir)

    with pytest.raises(export_import.ImportError):
        export_import.import_bundle(
            out, project_root / "context" / "team-members", assets_dir,
        )


# ---------------------------------------------------------------------------
# check_all summary
# ---------------------------------------------------------------------------

def test_check_all_aggregate(server_mod, project_root: Path, assets_dir):
    server_mod.create_team_member(name="Alice Chen", type="human", slug="alice-chen")
    server_mod.create_team_member(name="Bob Lee", type="human", slug="bob-lee")
    server_mod.init_memory_tree("alice-chen")
    server_mod.init_memory_tree("bob-lee")
    report = consistency.check_all(
        project_root,
        project_root / "context" / "team-members",
        server_mod._pool_path(),
        assets_dir,
    )
    assert report["summary"]["count"] == 2
    assert "alice-chen" in report["members"]
    assert "bob-lee" in report["members"]
