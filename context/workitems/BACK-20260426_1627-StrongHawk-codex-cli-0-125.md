---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260426_1627-StrongHawk-codex-cli-0-125
  created: '2026-04-26T16:27:28+00:00'
  updated: '2026-04-29T10:09:10+00:00'
spec:
  title: 'Codex CLI 0.125.0: /pk-* slash commands not surfaced from ~/.codex/prompts/'
  state: done
  type: bug
  priority: high
  description: |
    ## Symptom

    After v0.21.1 ship (commit 5204969 "feat(harness): scaffold processkit slash commands beyond Claude Code"), aibox writes `pk-resume.md`, `pk-doctor.md`, etc. to `.aibox-home/.codex/prompts/`, which is mounted as `~/.codex/prompts/` inside the container. The files are present (28 `pk-*.md` files plus 27 fully-qualified skill prompts). However, typing `/pk-resume` (or any other `/pk-*` slash command) inside the Codex CLI TUI does not invoke them — Codex reports the command as unavailable.

    ## Environment

    - codex-cli 0.125.0 (`/usr/bin/codex` → `@openai/codex`, native binary at `node_modules/@openai/codex-linux-arm64/vendor/.../codex`).
    - aibox v0.21.1 (commit e773442).
    - Files exist at both `/workspace/.aibox-home/.codex/prompts/pk-resume.md` and `~/.codex/prompts/pk-resume.md`.
    - Front-matter shipped: `argument-hint: ""` and `allowed-tools: []` (Claude Code style).

    ## Hypothesis

    Codex 0.125.0 likely changed the custom-prompt mechanism. Strings in the native binary reference a new `plugin` / `marketplace` surface (`addmarketplace`, `plugin marketplace`, `connector`, `experimental_use_profile`) and `experimental_instructions_file`. The legacy "drop a markdown file in `~/.codex/prompts/` to expose `/<name>`" path may have been removed, renamed, or gated behind a feature flag. Aibox's `harness_commands.rs` Codex profile still writes to the legacy path.

    ## Investigation steps

    1. Read upstream codex-cli 0.125.0 release notes / CHANGELOG. Confirm whether `~/.codex/prompts/` is still the supported custom-prompt path, or whether prompts moved to `codex plugin` / a marketplace.
    2. If the path moved: update `crates/aibox/src/harness/harness_commands.rs` Codex profile to write to the new path (and the SYNC_PERIMETER + doctor check accordingly).
    3. If the front-matter format changed: update the templater. Codex's old format was either bare markdown or a `name`/`description` front-matter — confirm 0.125.0 schema.
    4. If a feature flag is required (`codex features list` mentions feature flags), document the required flag in the AGENTS.md / harness docs.
    5. Add an aibox-doctor check that exercises `codex` end-to-end (e.g. `codex debug prompt-input` or similar) to detect this drift in future releases.

    ## Acceptance

    - `/pk-resume` invokes the status-briefing prompt inside `codex` TUI on a fresh container.
    - All 28 `pk-*` plus 27 fully-qualified skill prompts are listed by codex's prompt picker.
    - Doctor check fails loudly if the codex prompt path drifts again.

    ## Context

    User reported regression: "The /pk-resume command is still not available in codex cli." (2026-04-26 session.)
    Found via `pk-resume` testing during /pk-resume execution, after the v0.21.1 release shipped the harness scaffolding generically.
  started_at: '2026-04-26T16:33:44+00:00'
  completed_at: '2026-04-29T10:09:10+00:00'
---

## Transition note (2026-04-26T16:33:44+00:00)

Starting investigation: research what changed in Codex 0.125.0's prompt-loading mechanism, then determine the fix for cli/src/harness_commands.rs Codex profile.


## Transition note (2026-04-29T10:09:04+00:00)

Reconciliation review: Codex pk command scaffolding now targets .agents/skills/<name>/SKILL.md with legacy prompt cleanup; decision DEC-20260426_1636-MightySky records the integration surface.


## Transition note (2026-04-29T10:09:10+00:00)

Closed during 2026-04-29 reconciliation after confirming Codex Skills scaffolding and doctor/perimeter updates are present in the working tree.
