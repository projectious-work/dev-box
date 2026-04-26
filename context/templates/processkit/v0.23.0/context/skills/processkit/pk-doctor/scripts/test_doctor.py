#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "pyyaml>=6.0",
#   "jsonschema>=4.0",
# ]
# ///
"""test_doctor.py — smoke tests for pk-doctor (Phase 1).

Seeds a tiny context/ + src/context/ tree in a tempdir and asserts:
- CheckResult shape + severity tallies
- aggregator exit code (0 if no ERROR, 1 otherwise)
- stub LogEntry payload shape (emit called exactly once per run)

Run from any directory:

    python3 context/skills/processkit/pk-doctor/scripts/test_doctor.py

or via the uv shebang if PyYAML/jsonschema aren't globally installed.

Exit 0 = all tests passed.
Exit 1 = at least one test failed.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import textwrap
from datetime import datetime, timezone, timedelta
from pathlib import Path


# ---------------------------------------------------------------------------
# Locate files relative to this script, not cwd.
# ---------------------------------------------------------------------------
_SCRIPTS_DIR = Path(__file__).resolve().parent
_DOCTOR = _SCRIPTS_DIR / "doctor.py"
_REPO_ROOT = Path(__file__).resolve().parents[5]  # skills/processkit/pk-doctor/scripts → repo root
_SCHEMAS_SRC = _REPO_ROOT / "src" / "context" / "schemas"

PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"

failures: list[str] = []


def check(name: str, condition: bool, detail: str = "") -> None:
    if condition:
        print(f"  {PASS}  {name}")
    else:
        print(f"  {FAIL}  {name}" + (f": {detail}" if detail else ""))
        failures.append(name)


# ---------------------------------------------------------------------------
# Fixture tree builder
# ---------------------------------------------------------------------------

def _seed_tree(root: Path) -> None:
    """Populate a minimal repo layout under `root`."""
    # Copy live schemas so schema_filename has something to validate against.
    (root / "src" / "context" / "schemas").mkdir(parents=True)
    for name in ("workitem.yaml", "logentry.yaml", "migration.yaml"):
        src = _SCHEMAS_SRC / name
        if src.is_file():
            (root / "src" / "context" / "schemas" / name).write_text(
                src.read_text(encoding="utf-8"), encoding="utf-8"
            )

    # Stub the drift script — simplest: tree is trivially in-sync.
    scripts = root / "scripts"
    scripts.mkdir()
    drift = scripts / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)

    # --- one valid workitem -----------------------------------------------
    ctx = root / "context"
    (ctx / "workitems").mkdir(parents=True)
    (ctx / "workitems" / "BACK-20260420_1200-GoodApple-valid-item.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: WorkItem
            metadata:
              id: BACK-20260420_1200-GoodApple-valid-item
              created: 2026-04-20T12:00:00Z
            spec:
              title: Valid test item
              state: proposed
              type: task
            ---
            body
            """),
        encoding="utf-8",
    )

    # --- one workitem with filename / id mismatch (WARN) ------------------
    (ctx / "workitems" / "BACK-bad-filename.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: WorkItem
            metadata:
              id: BACK-20260420_1300-BadApple-item
              created: 2026-04-20T13:00:00Z
            spec:
              title: Mismatched filename
              state: proposed
            ---
            """),
        encoding="utf-8",
    )

    # --- one mis-sharded log (WARN from sharding) -------------------------
    (ctx / "logs").mkdir()
    (ctx / "logs" / "LOG-toplevel-misplaced.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: LogEntry
            metadata:
              id: LOG-toplevel-misplaced
              created: 2026-04-20T14:00:00Z
            spec:
              event_type: test.event
              actor: ACTOR-test
              timestamp: 2026-04-20T14:00:00Z
            ---
            """),
        encoding="utf-8",
    )

    # --- one stale pending migration (WARN from migrations) ---------------
    (ctx / "migrations" / "pending").mkdir(parents=True)
    stale_created = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat().replace("+00:00", "Z")
    (ctx / "migrations" / "pending" / "MIG-20260101T000000.md").write_text(
        textwrap.dedent(f"""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: Migration
            metadata:
              id: MIG-20260101T000000
              created: {stale_created}
            spec:
              source: processkit
              from_version: v0.0.1
              to_version: v0.0.2
              state: pending
              summary: stale test migration
            ---
            """),
        encoding="utf-8",
    )
    # Ensure the other two required state-bucket dirs exist.
    (ctx / "migrations" / "in-progress").mkdir()
    (ctx / "migrations" / "applied").mkdir()


# ---------------------------------------------------------------------------
# Run helper
# ---------------------------------------------------------------------------

def _run_doctor(repo_root: Path, *extra: str, stub_path: Path) -> subprocess.CompletedProcess:
    env = os.environ.copy()
    env["PK_DOCTOR_LOGENTRY_STUB"] = str(stub_path)
    # Use the uv shebang for dep isolation.
    cmd = [str(_DOCTOR), "--repo-root", str(repo_root), *extra]
    return subprocess.run(cmd, capture_output=True, text=True, env=env)


# ---------------------------------------------------------------------------
# Test 1: baseline run on seeded fixture
# ---------------------------------------------------------------------------
print("\n[1] doctor.py — baseline detect-only run on seeded fixture")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)

    check("ran without crash (exit 0 or 1)", result.returncode in (0, 1),
          f"got {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    check("stdout contains report header", "pk-doctor v" in result.stdout)
    check("schema_filename section present", "## schema_filename" in result.stdout)
    check("sharding section present", "## sharding" in result.stdout)
    check("migrations section present", "## migrations" in result.stdout)
    check("drift section present", "## drift" in result.stdout)
    check("totals line present", "## totals" in result.stdout)
    check("stub log payload written", stub.is_file())
    if stub.is_file():
        payload = json.loads(stub.read_text())
        check("payload event_type correct", payload.get("event_type") == "doctor.report")
        check("payload has categories", isinstance(payload.get("details", {}).get("categories"), dict))
        check("payload has duration_ms", "duration_ms" in payload.get("details", {}))
        check("payload has doctor_version", payload.get("details", {}).get("doctor_version") == "1.0.0")
        # Should see our seeded findings: bad filename WARN, mis-sharded log WARN,
        # stale migration WARN.
        findings_blob = json.dumps(payload["details"]["top_findings"])
        check("top_findings contains filename-id-mismatch",
              "schema.filename-id-mismatch" in findings_blob,
              findings_blob[:400])
        check("top_findings contains sharding.log-wrong-bucket",
              "sharding.log-wrong-bucket" in findings_blob,
              findings_blob[:400])
        check("top_findings contains migration.stale-pending",
              "migration.stale-pending" in findings_blob,
              findings_blob[:400])

# ---------------------------------------------------------------------------
# Test 2: ERROR causes exit 1; INFO-only run exits 0
# ---------------------------------------------------------------------------
print("\n[2] doctor.py — exit code reflects ERROR tally")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    # Inject a broken-schema workitem that will produce an ERROR.
    (root / "context" / "workitems" / "BACK-20260420_1400-CleanBad-invalid.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: WorkItem
            metadata:
              id: BACK-20260420_1400-CleanBad-invalid
              created: 2026-04-20T14:00:00Z
            spec:
              state: proposed
            ---
            """),  # missing required `title`
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)
    check("exit 1 when ERROR present", result.returncode == 1,
          f"got {result.returncode}\nstdout:\n{result.stdout[-600:]}")
    payload = json.loads(stub.read_text())
    check("ERROR tally non-zero", any(
        c.get("ERROR", 0) > 0 for c in payload["details"]["categories"].values()
    ))

# ---------------------------------------------------------------------------
# Test 3: --category= restricts run
# ---------------------------------------------------------------------------
print("\n[3] doctor.py — --category=drift runs only the drift check")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=drift", stub_path=stub)
    check("exit 0 (clean drift)", result.returncode == 0,
          f"got {result.returncode}\nstderr:\n{result.stderr}")
    check("schema_filename section absent", "## schema_filename" not in result.stdout)
    check("drift section present", "## drift" in result.stdout)
    payload = json.loads(stub.read_text())
    check("only drift category in payload",
          list(payload["details"]["categories"].keys()) == ["drift"])

# ---------------------------------------------------------------------------
# Test 4: --fix + --fix-all mutex
# ---------------------------------------------------------------------------
print("\n[4] doctor.py — --fix and --fix-all are mutually exclusive")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--fix=migrations", "--fix-all", stub_path=stub)
    # argparse mutex groups exit 2 with error on stderr.
    check("mutex enforced (exit 2)", result.returncode == 2,
          f"got {result.returncode}\nstderr:{result.stderr[:200]}")

# ---------------------------------------------------------------------------
# Test 5: single logentry emitted per run (stub file overwritten, not appended)
# ---------------------------------------------------------------------------
print("\n[5] doctor.py — exactly one logentry emission per run")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)
    check("stub exists", stub.is_file())
    content = stub.read_text()
    check("stub parses as single JSON object (not JSONL)",
          content.strip().startswith("{") and content.strip().endswith("}"))
    # JSON object count — a second run should overwrite, not duplicate.
    result2 = _run_doctor(root, stub_path=stub)
    payload2 = json.loads(stub.read_text())
    check("second run overwrote (still valid JSON object)",
          isinstance(payload2, dict) and payload2.get("event_type") == "doctor.report")

# ---------------------------------------------------------------------------
# Test 6: _check_actor_id_pattern unit tests
# ---------------------------------------------------------------------------
print("\n[6] _check_actor_id_pattern — actor two-class ID pattern unit tests")

# Import the function under test directly.
sys.path.insert(0, str(_SCRIPTS_DIR))
from checks.schema_filename import _check_actor_id_pattern  # noqa: E402

_ALLOWED = [
    "assistant", "developer", "jr-architect", "jr-developer",
    "jr-researcher", "pm-claude", "sr-architect", "sr-researcher",
]

# valid identity-class ID → no error
check(
    "identity class OK",
    _check_actor_id_pattern(
        "ACTOR-20260421_0144-AmberDawn-legacy-historical-backfill", _ALLOWED
    ) is None,
)

# valid role-class ID in allowlist → no error
check(
    "role class in allowlist OK",
    _check_actor_id_pattern("ACTOR-pm-claude", _ALLOWED) is None,
)

# role-class ID NOT in allowlist → ERROR
result_not_in_list = _check_actor_id_pattern("ACTOR-hacker", _ALLOWED)
check(
    "role class not in allowlist → error",
    result_not_in_list is not None and "not in x-allowed-role-ids" in result_not_in_list,
    repr(result_not_in_list),
)

# malformed (underscores and caps) → ERROR
result_malformed = _check_actor_id_pattern("ACTOR-Sr_Architect", _ALLOWED)
check(
    "malformed ID (caps+underscores) → error",
    result_malformed is not None,
    repr(result_malformed),
)

# empty-slug → ERROR
result_empty = _check_actor_id_pattern("ACTOR-", _ALLOWED)
check(
    "empty slug (ACTOR-) → error",
    result_empty is not None,
    repr(result_empty),
)

# identity class with broken datetime → ERROR
result_bad_dt = _check_actor_id_pattern("ACTOR-12345-FooBar-x", _ALLOWED)
check(
    "broken datetime in identity class → error",
    result_bad_dt is not None,
    repr(result_bad_dt),
)

# ---------------------------------------------------------------------------
# Test 7: pk-doctor integration — invalid actor ID triggers ERROR
# ---------------------------------------------------------------------------
print("\n[7] pk-doctor integration — invalid actor ID emits ERROR")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    _seed_tree(root)

    # Copy live actor schema into fixture tree.
    actor_schema_src = _SCHEMAS_SRC / "actor.yaml"
    if actor_schema_src.is_file():
        (root / "src" / "context" / "schemas" / "actor.yaml").write_text(
            actor_schema_src.read_text(encoding="utf-8"), encoding="utf-8"
        )

    # Plant an actor with an ID not in the allowlist.
    (root / "context" / "actors").mkdir(parents=True)
    (root / "context" / "actors" / "ACTOR-hacker.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: Actor
            metadata:
              id: ACTOR-hacker
              created: 2026-04-21T00:00:00Z
            spec:
              type: ai-agent
              name: Hacker Bot
              active: true
            ---
            """),
        encoding="utf-8",
    )

    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, stub_path=stub)
    check("exit 1 for invalid actor ID", result.returncode == 1,
          f"got {result.returncode}\nstdout:\n{result.stdout[-600:]}")
    check(
        "invalid_actor_id_pattern in output",
        "invalid_actor_id_pattern" in result.stdout,
        result.stdout[-600:],
    )

# ---------------------------------------------------------------------------
# Test 8: derived-project schema_filename fallback (HappyReef)
# Ensures pk-doctor walks entity files even when the dogfood
# `src/context/schemas/` tree is absent — the bug that hid all
# entity-hygiene findings in aibox-installed repos.
# ---------------------------------------------------------------------------
print("\n[8] schema_filename — derived-project fallback to context/schemas/")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    # No src/ tree — schemas live at context/schemas/ (derived layout).
    (root / "context" / "schemas").mkdir(parents=True)
    src_logentry = _SCHEMAS_SRC / "logentry.yaml"
    if src_logentry.is_file():
        (root / "context" / "schemas" / "logentry.yaml").write_text(
            src_logentry.read_text(encoding="utf-8"), encoding="utf-8"
        )
    # Stub drift script so doctor doesn't blow up there.
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)
    # Seed a malformed LogEntry — should now be caught.
    (root / "context" / "logs").mkdir()
    (root / "context" / "logs" / "LOG-20260425_1100-Fixture-event.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: LogEntry
            metadata:
              id: LOG-20260425_1100-Fixture-event
              created: '2026-04-25T11:00:00+00:00'
            spec:
              event_type: test.fixture
              timestamp: '2026-04-25T11:00:00+00:00'
              summary: missing actor — should be caught
            ---
            """),
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=schema_filename", stub_path=stub)
    check(
        "exit 1 (ERROR found via derived layout)",
        result.returncode == 1,
        f"got {result.returncode}; stdout: {result.stdout[-400:]}",
    )
    check(
        "schema.invalid actor-required ERROR fired",
        "'actor' is a required property" in result.stdout,
        result.stdout[-400:],
    )
    check(
        "walked > 0 entity files (no longer silent zero)",
        "walked 1 entity file" in result.stdout,
        result.stdout[-400:],
    )

# ---------------------------------------------------------------------------
# Test 9: migrations layout fallback (DeepMoss)
# Ensures pk-doctor counts true Migration entities sitting at the top
# level of context/migrations/ (derived-project layout) and skips
# aibox-CLI upgrade docs.
# ---------------------------------------------------------------------------
print("\n[9] migrations — derived-project layout (top-level + applied/) fallback")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    (root / "context" / "migrations" / "applied").mkdir(parents=True)
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)
    # Real Migration entity at the TOP level (derived layout).
    (root / "context" / "migrations" / "MIG-20260425T1100-test-fixture.md").write_text(
        textwrap.dedent("""\
            ---
            apiVersion: processkit.projectious.work/v1
            kind: Migration
            metadata:
              id: MIG-20260425T1100-test-fixture
              created: '2026-04-25T11:00:00+00:00'
            spec:
              source: processkit
              state: pending
            ---
            test fixture migration
            """),
        encoding="utf-8",
    )
    # aibox-CLI upgrade doc — must be filtered out.
    (root / "context" / "migrations" / "20260425_1100_0.18.5-to-0.18.6.md").write_text(
        "# CLI upgrade doc — not a Migration entity\n",
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=migrations", stub_path=stub)
    check(
        "1 pending Migration counted (CLI doc filtered)",
        "1 pending migration" in result.stdout,
        result.stdout[-400:],
    )

# ---------------------------------------------------------------------------
# Test 10: skill_dag — clean roster (no missing refs, no cycles, no violations)
# ---------------------------------------------------------------------------
print("\n[10] skill_dag — clean roster produces 0 errors")

from checks.skill_dag import run as _skill_dag_run  # noqa: E402

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    # category: processkit; skills: alpha (layer 0) and beta (layer 1 uses alpha)
    (skills / "processkit" / "alpha").mkdir(parents=True)
    (skills / "processkit" / "alpha" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: alpha
            metadata:
              processkit:
                layer: 0
            ---
            """),
        encoding="utf-8",
    )
    (skills / "processkit" / "beta").mkdir(parents=True)
    (skills / "processkit" / "beta" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: beta
            metadata:
              processkit:
                layer: 1
                uses:
                  - skill: alpha
                    purpose: depends on alpha
            ---
            """),
        encoding="utf-8",
    )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    errors = [r for r in results if r.severity == "ERROR"]
    infos = [r for r in results if r.id == "skill.dag.summary"]
    check("10a: clean roster — 0 ERRORs", len(errors) == 0,
          f"errors: {[r.message for r in errors]}")
    check("10b: summary INFO emitted", len(infos) == 1, str(infos))
    check("10c: walked 2 skills in summary",
          infos and "walked 2 skill(s)" in infos[0].message,
          infos[0].message if infos else "")

# ---------------------------------------------------------------------------
# Test 11: skill_dag — missing reference triggers ERROR
# ---------------------------------------------------------------------------
print("\n[11] skill_dag — missing reference")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    (skills / "processkit" / "foo").mkdir(parents=True)
    (skills / "processkit" / "foo" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: foo
            metadata:
              processkit:
                layer: 2
                uses:
                  - skill: nonexistent-bar
                    purpose: references a skill that does not exist
            ---
            """),
        encoding="utf-8",
    )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    missing_errors = [r for r in results if r.id == "skill.dag.missing-ref"]
    check("11a: missing-ref ERROR fired",
          len(missing_errors) == 1,
          f"got: {[r.message for r in missing_errors]}")
    check("11b: error message names both skills",
          missing_errors and "foo" in missing_errors[0].message and "nonexistent-bar" in missing_errors[0].message,
          missing_errors[0].message if missing_errors else "")

# ---------------------------------------------------------------------------
# Test 12: skill_dag — 3-node cycle triggers ERROR
# ---------------------------------------------------------------------------
print("\n[12] skill_dag — 3-node cycle detection")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    for sname, dep in [("aaa", "ccc"), ("bbb", "aaa"), ("ccc", "bbb")]:
        (skills / "processkit" / sname).mkdir(parents=True)
        (skills / "processkit" / sname / "SKILL.md").write_text(
            textwrap.dedent(f"""\
                ---
                name: {sname}
                metadata:
                  processkit:
                    layer: 1
                    uses:
                      - skill: {dep}
                        purpose: cyclic dep
                ---
                """),
            encoding="utf-8",
        )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    cycle_errors = [r for r in results if r.id == "skill.dag.cycle"]
    check("12a: cycle ERROR fired",
          len(cycle_errors) >= 1,
          f"got: {[r.message for r in cycle_errors]}")
    check("12b: cycle message contains 'cycle detected'",
          any("cycle detected" in r.message for r in cycle_errors),
          str([r.message for r in cycle_errors]))

# ---------------------------------------------------------------------------
# Test 13: skill_dag — layer violation triggers ERROR
# ---------------------------------------------------------------------------
print("\n[13] skill_dag — layer violation")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    skills = root / "context" / "skills"
    # low (layer 0) uses high (layer 3) — violation
    (skills / "processkit" / "low").mkdir(parents=True)
    (skills / "processkit" / "low" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: low
            metadata:
              processkit:
                layer: 0
                uses:
                  - skill: high
                    purpose: upward layer reference
            ---
            """),
        encoding="utf-8",
    )
    (skills / "processkit" / "high").mkdir(parents=True)
    (skills / "processkit" / "high" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: high
            metadata:
              processkit:
                layer: 3
            ---
            """),
        encoding="utf-8",
    )
    ctx = {"repo_root": root}
    results = _skill_dag_run(ctx)
    layer_errors = [r for r in results if r.id == "skill.dag.layer-violation"]
    check("13a: layer-violation ERROR fired",
          len(layer_errors) == 1,
          f"got: {[r.message for r in layer_errors]}")
    check("13b: error message mentions both layers",
          layer_errors and "layer 0" in layer_errors[0].message and "layer 3" in layer_errors[0].message,
          layer_errors[0].message if layer_errors else "")
    check("13c: error message names both skills",
          layer_errors and "low" in layer_errors[0].message and "high" in layer_errors[0].message,
          layer_errors[0].message if layer_errors else "")

# ---------------------------------------------------------------------------
# Test 14: skill_dag -- --category=skill_dag integration
# ---------------------------------------------------------------------------
print("\n[14] skill_dag — integration via --category=skill_dag")

with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    # Stub drift + minimal seed
    (root / "scripts").mkdir()
    drift = root / "scripts" / "check-src-context-drift.sh"
    drift.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    drift.chmod(0o755)
    # One clean skill under context/skills/
    skills = root / "context" / "skills"
    (skills / "processkit" / "clean-skill").mkdir(parents=True)
    (skills / "processkit" / "clean-skill" / "SKILL.md").write_text(
        textwrap.dedent("""\
            ---
            name: clean-skill
            metadata:
              processkit:
                layer: 0
            ---
            """),
        encoding="utf-8",
    )
    stub = root / ".doctor-logentry.json"
    result = _run_doctor(root, "--category=skill_dag", stub_path=stub)
    check("14a: --category=skill_dag runs without crash",
          result.returncode in (0, 1),
          f"got {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    check("14b: skill_dag section present in output",
          "## skill_dag" in result.stdout,
          result.stdout[-400:])
    check("14c: summary line present",
          "walked" in result.stdout and "skill(s)" in result.stdout,
          result.stdout[-400:])

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print()
if failures:
    print(f"FAILED ({len(failures)} test(s)):", ", ".join(failures))
    sys.exit(1)
else:
    print("All tests passed.")
    sys.exit(0)
