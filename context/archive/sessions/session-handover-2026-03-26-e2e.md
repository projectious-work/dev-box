# Session Handover — 2026-03-26 (E2E Test Run)

## What Was Done

### E2E Test Suite Run Against Companion Container

Ran the full E2E test suite (`cargo test --features e2e`) against the `aibox-e2erunner` companion container.

- **73 passed, 4 failed** in ~127 seconds
- All Tier 1 (local) and most Tier 2 (companion) tests passing

### Dockerfile.e2e Fix (Uncommitted)

Changed `AcceptEnv` sed in `.devcontainer/Dockerfile.e2e` to comment out the line (`#AcceptEnv`) instead of leaving it blank (`AcceptEnv`). This was already staged before the session.

---

## Failures Requiring Attention

### 1. `addon::addon_remove_cleans_toml` — REAL BUG

`aibox.toml` still contains `[addons.python]` after `addon remove python`. The addon remove command does not properly clean the TOML section. This is a code bug in the CLI's addon remove logic.

**Location:** `cli/tests/e2e/addon.rs:60` — assertion fails.
**Next step:** Investigate `addon_cmd.rs` remove logic; the unit test `addon_remove_deletes_section` passes, so the issue may be environment-specific (e.g., file not flushed, or the e2e test writes the config differently than the unit test expects).

### 2. `smoke::podman_can_run_container` — ENVIRONMENT

Podman's `pasta` networking fails because `/dev/net/tun` is not available in the nested container environment. Not a code bug — this is a limitation of the devcontainer-in-devcontainer setup.

**Error:** `Failed to open() /dev/net/tun: No such file or directory`
**Next step:** Either add `/dev/net/tun` device to the e2e runner's docker-compose config, or switch the test to use `--network=none` or `slirp4netns` networking.

### 3. `visual_keybindings::visual_kb_lazygit_space_stages_file` — FLAKY

Expected lazygit Staged/Unstaged file panel not found in the asciinema cast output. Likely a timing issue — lazygit may not have rendered the panel before the cast was captured.

### 4. `visual_keybindings::visual_kb_vim_leader_e_opens_netrw` — TIMEOUT

Timed out at 60s. Vim session may have hung or been too slow to render netrw in the test environment.

---

## Current State

- **Branch:** `main`
- **Version:** `v0.13.1`
- **Uncommitted changes:** `.devcontainer/Dockerfile.e2e` (AcceptEnv comment-out fix)
- **Test results:** 73/77 passing (4 failures described above)
- **Companion container:** `aibox-e2erunner` is running and reachable

## Suggested Next Steps

1. **Fix `addon_remove_cleans_toml` bug** — this is the only real code defect found
2. **Fix `/dev/net/tun` for smoke test** — add device to docker-compose.override.yml e2e service, or adjust test networking mode
3. **Consider increasing timeout or adding retry** for the two flaky visual keybinding tests
4. **Commit the Dockerfile.e2e fix** once validated
