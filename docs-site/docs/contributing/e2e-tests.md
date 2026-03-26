---
sidebar_position: 3
title: E2E Test Catalogue
---

# E2E Test Catalogue

This page describes every end-to-end and integration test in `cli/tests/e2e/`
in plain language. Each entry states the precondition and the expected outcome,
followed by a reference to the exact test function for traceability.

## Test tiers

| Tier | Where it runs | What it needs |
|---|---|---|
| **Tier 1** | Local temp directory | The compiled `aibox` binary only |
| **Tier 1 + Mock** | Local temp directory | Binary + mock `docker`/`podman` scripts on PATH |
| **Tier 2** | Remote SSH companion | `aibox-e2e-testrunner` container reachable (feature flag `e2e`) |

Tier 1 tests run automatically with `cargo test`. Tier 2 tests require the
companion container and the `--features e2e` flag.

---

## Lifecycle — `lifecycle.rs`

**Companion is reachable**
If the SSH connection to `aibox-e2e-testrunner` is attempted, then the host
must respond with `ok`, confirming the companion container is up and reachable
before any other Tier 2 test runs.
`[lifecycle.rs · companion_is_reachable]`

**Init then sync produces valid project**
If `aibox init` is run followed by `aibox sync --no-build`, then
`aibox.toml`, `.devcontainer/Dockerfile`, `.devcontainer/docker-compose.yml`,
and `CLAUDE.md` must all exist in the workspace.
`[lifecycle.rs · lifecycle_init_sync]`

**CLAUDE.md user content is preserved on sync**
If a user edits `CLAUDE.md` after `aibox init` and then runs `aibox sync`,
then the edited content must still be present — aibox must not overwrite
user-modified files.
`[lifecycle.rs · claudemd_preserved_on_sync]`

**Generated files are overwritten on sync**
If a generated file (e.g. `.devcontainer/Dockerfile`) is manually tampered
with and `aibox sync` is run, then the file must contain regenerated content
and the tampered content must be gone.
`[lifecycle.rs · generated_files_overwritten_on_sync]`

**Status reports missing when no container exists**
If `aibox status` is run in a project with no running container, then the
output must contain `missing` or equivalent wording.
`[lifecycle.rs · status_without_container_shows_missing]`

**Managed preset creates expected context files**
If `aibox init --process managed` is run, then `context/BACKLOG.md`,
`context/DECISIONS.md`, `context/STANDUPS.md`, and
`context/project-notes/session-template.md` must all be created, and
`aibox.toml` must reference the `managed` process.
`[lifecycle.rs · init_with_managed_preset_creates_context_files]`

**Software preset creates processes directory**
If `aibox init --process software` is run, then the tracking and standups
files from the `managed` base must exist, and `context/processes/` must also
be created for the architecture package.
`[lifecycle.rs · init_with_software_preset_creates_code_files]`

---

## Addon management — `addon.rs`

**Addon add writes to aibox.toml**
If `aibox addon add python --no-build` is run in an initialized project, then
`aibox.toml` must contain an `[addons.python]` section afterwards.
`[addon.rs · addon_add_modifies_toml]`

**Addon remove cleans aibox.toml**
If a project is initialized with the `python` addon and then `aibox addon
remove python --no-build` is run, then the `[addons.python]` section must no
longer appear in `aibox.toml`.
`[addon.rs · addon_remove_cleans_toml]`

**Addon content appears in generated Dockerfile after sync**
If a project is initialized with the `python` addon and `aibox sync` is run,
then `.devcontainer/Dockerfile` must contain Python-related content (install
commands or references to `uv`).
`[addon.rs · addon_rebuild_includes_tools_in_dockerfile]`

**Addon list shows available addons**
If `aibox addon list` is run in an initialized project, then the output must
list known addons such as `python`.
`[addon.rs · addon_list_shows_available]`

---

## Reset and backup — `reset.rs`

**Reset with backup removes files and creates backup directory**
If `aibox reset --yes` is run in an initialized project, then `aibox.toml`
must be deleted and `.aibox-backup/` must be created containing the backed-up
files.
`[reset.rs · reset_creates_backup]`

**Reset with --no-backup removes all files without creating a backup**
If `aibox reset --no-backup --yes` is run, then `aibox.toml` and
`.devcontainer/` must be deleted and `.aibox-backup/` must not be created.
`[reset.rs · reset_no_backup_deletes_all]`

---

## Doctor diagnostics — `doctor.rs`

**Doctor without a config reports an error**
If `aibox doctor` is run in a directory that has no `aibox.toml`, then the
output must mention the missing config or config error and the command must
still exit 0 (doctor is always non-fatal).
`[doctor.rs · doctor_reports_missing_files]`

**Doctor after init reports healthy checks**
If `aibox doctor` is run immediately after a successful `aibox init`, then
the output must contain at least one passing check indicator (`ok`, `✓`, or
similar).
`[doctor.rs · doctor_after_init_reports_healthy]`

---

## Version upgrade flows — `version_upgrade.rs`

**Generated Dockerfile contains version label**
If `aibox init` is run, then the generated `.devcontainer/Dockerfile` must
contain a `LABEL aibox.version` line so the built image carries a
machine-readable version stamp.
`[version_upgrade.rs · dockerfile_contains_aibox_version_label]`

**Generated Dockerfile writes version to /etc/aibox-version**
If `aibox init` is run, then the generated `.devcontainer/Dockerfile` must
contain a `RUN` statement that writes to `/etc/aibox-version` inside the
image, making the build version queryable from within a running container.
`[version_upgrade.rs · dockerfile_contains_etc_aibox_version_write]`

**Start fails when container image version mismatches config**
If an existing container was built from image `v0.0.1` (mock label) and
`aibox.toml` pins the current version, then `aibox start` must exit non-zero
and output a message containing `mismatch` and a suggestion to run
`aibox sync`.
`[version_upgrade.rs · start_fails_on_image_version_mismatch]`

**Start succeeds when container image version matches config**
If an existing container reports the same image version as the one pinned in
`aibox.toml`, then `aibox start` must not produce a version mismatch error.
`[version_upgrade.rs · start_does_not_error_when_versions_match]`

**Update -y exits zero without hanging**
If `aibox update -y` is run (the global `--yes` flag), then the command must
exit 0 regardless of registry availability — confirming the flag is correctly
wired to `cmd_update` and does not block on an interactive prompt.
`[version_upgrade.rs · update_yes_flag_exits_zero]`

**Update --dry-run does not mention .aibox-version**
If `aibox update --dry-run` is run, then the output must not contain the
phrase `Would update .aibox-version` — that write was removed in BACK-060
because the image version is now tracked exclusively in `aibox.toml`.
`[version_upgrade.rs · update_dry_run_does_not_mention_aibox_version_file]`

**Doctor warns when running container has a stale image label**
If the running container reports `aibox.version=0.0.1` (mock label) but
`aibox.toml` pins the current version, then `aibox doctor` must emit a
warning containing `mismatch` while still exiting 0.
`[version_upgrade.rs · doctor_warns_on_container_version_mismatch]`

**Doctor warns when .aibox-version is outdated**
If `.aibox-version` is overwritten with `0.0.1` (an old CLI version) and
`aibox doctor` is run, then the output must contain `CLI version mismatch`
and suggest running `aibox sync` to update generated files.
`[version_upgrade.rs · doctor_warns_on_cli_version_file_mismatch]`

---

## Migration — `migration.rs`

**Sync updates .aibox-version when it is outdated**
If `.aibox-version` is overwritten with `0.1.0` (an old version) and
`aibox sync` is run, then `.aibox-version` must be updated to a non-empty
value that is no longer `0.1.0`, indicating the migration system ran and
stamped the current version.
`[migration.rs · sync_updates_version_file]`

---

## Update command — `update.rs`

**Update exits zero when registry returns an error**
If `aibox update` is run in a project where the GHCR registry is unreachable
or returns a non-2xx response, then the command must still exit 0 — the error
must be treated as a warning, not a hard failure.
`[update.rs · update_runs_without_crashing_in_derived_project]`

**Update --check exits zero**
If `aibox update --check` is run in an initialized project, then the command
must exit 0 and print output containing either `Current CLI version:` or
`Checking for updates`, regardless of whether the registry is reachable.
`[update.rs · update_check_exits_cleanly]`

---

## Appearance — `appearance.rs`

**All themes render without error and without leftover placeholders**
If `aibox init` is run for each of the seven supported themes
(`gruvbox-dark`, `catppuccin-mocha`, `catppuccin-latte`, `dracula`,
`tokyo-night`, `nord`, `projectious`), then the seeded config files must
contain no unreplaced template placeholders such as `AIBOX_THEME` or
`AIBOX_VIM_COLORSCHEME`.
`[appearance.rs · all_themes_render_without_error]`

**Gruvbox theme sets the correct vim colorscheme and zellij theme**
If `aibox init --theme gruvbox-dark` is run, then `vimrc` must contain
`gruvbox` or `retrobox` as the colorscheme and `config.kdl` must reference
`gruvbox-dark`.
`[appearance.rs · theme_gruvbox_renders_correctly]`

**Catppuccin-mocha theme is reflected in zellij config**
If `aibox init --theme catppuccin-mocha` is run, then `config.kdl` must
reference `catppuccin-mocha`.
`[appearance.rs · theme_catppuccin_mocha_renders]`

**Changing the theme updates all themed tool configs**
If a project is initialized with `gruvbox-dark` and the theme is changed to
`dracula` via `aibox sync`, then `config.kdl` must contain `dracula` and no
longer `gruvbox-dark`, and `vimrc`, `yazi/theme.toml`, `lazygit/config.yml`,
and `starship.toml` must all be non-empty and updated.
`[appearance.rs · theme_change_updates_all_files]`

**Each theme produces matching configs across all tools**
If `aibox init` is run for each of five themes with known vim colorscheme
names, then `vimrc` must contain the exact `colorscheme <name>` line,
`config.kdl` must reference the theme name, and the yazi, lazygit, and
starship configs must all be non-empty.
`[appearance.rs · theme_alignment_all_tools_match_selected_theme]`

**Yazi keymap includes the open-in-editor binding**
If `aibox init` is run, then `yazi/keymap.toml` must contain an `"e"` key
binding that invokes `open-in-editor`.
`[appearance.rs · yazi_keymap_includes_edit_in_pane_binding]`

**All prompt presets produce a non-empty starship config**
If `aibox init` is run for each of the six prompt presets (`default`,
`plain`, `minimal`, `nerd-font`, `pastel`, `bracketed`), then
`starship.toml` must exist and be non-empty.
`[appearance.rs · all_prompts_render_without_error]`

**Default prompt includes directory and git_branch modules**
If `aibox init --prompt default` is run, then `starship.toml` must contain
both `directory` and `git_branch` module sections.
`[appearance.rs · prompt_default_generates_starship]`

**Plain prompt uses ASCII-only symbols**
If `aibox init --prompt plain` is run, then `starship.toml` must not contain
Nerd Font glyph characters (e.g. no `\ue0b0` powerline arrow).
`[appearance.rs · prompt_plain_no_nerd_font]`

---

## Config coverage — `config_coverage.rs`

**Container name appears in docker-compose.yml**
If `aibox.toml` specifies a container name and `aibox sync` is run, then
`docker-compose.yml` must contain that name.
`[config_coverage.rs · container_name_in_compose]`

**Container hostname appears in docker-compose.yml**
If `aibox.toml` specifies a hostname and `aibox sync` is run, then
`docker-compose.yml` must contain that hostname.
`[config_coverage.rs · container_hostname_in_compose]`

**Port mappings appear in docker-compose.yml**
If `aibox.toml` defines ports (e.g. `"8080:80"`) and `aibox sync` is run,
then `docker-compose.yml` must contain those port entries.
`[config_coverage.rs · container_ports_in_compose]`

**Extra packages appear in the generated Dockerfile**
If `aibox.toml` lists extra packages and `aibox sync` is run, then
`.devcontainer/Dockerfile` must contain those package names in an apt install
block.
`[config_coverage.rs · container_extra_packages_in_dockerfile]`

**Environment variables appear in docker-compose.yml**
If `aibox.toml` defines environment variables and `aibox sync` is run, then
`docker-compose.yml` must contain those key-value pairs.
`[config_coverage.rs · container_environment_in_compose]`

**Extra volumes appear in docker-compose.yml**
If `aibox.toml` defines extra volume mounts and `aibox sync` is run, then
`docker-compose.yml` must contain those source and target paths.
`[config_coverage.rs · container_extra_volumes_in_compose]`

**Claude AI provider adds volume mount**
If `aibox.toml` lists `claude` as an AI provider and `aibox sync` is run,
then `docker-compose.yml` must contain a volume mount for the `.claude`
config directory.
`[config_coverage.rs · ai_claude_provider_volume_mount]`

**Aider AI provider adds volume mount**
If `aibox.toml` lists `aider` as an AI provider and `aibox sync` is run,
then `docker-compose.yml` must contain a volume mount for the `.aider`
config directory.
`[config_coverage.rs · ai_aider_provider_volume_mount]`

**Multiple AI providers each add their own volume mounts**
If `aibox.toml` lists both `claude` and `gemini` as providers and `aibox
sync` is run, then `docker-compose.yml` must contain volume mounts for both
`.claude` and `.gemini`.
`[config_coverage.rs · ai_multiple_providers_volume_mounts]`

**Audio enabled adds PulseAudio mounts and socket**
If `aibox.toml` enables audio and `aibox sync` is run, then
`docker-compose.yml` must contain audio-related volume mounts or socket
references.
`[config_coverage.rs · audio_enabled_adds_mounts]`

**Audio disabled produces no audio mounts**
If `aibox.toml` has audio disabled (the default) and `aibox sync` is run,
then `docker-compose.yml` must not contain audio-related content.
`[config_coverage.rs · audio_disabled_no_mounts]`

**Python addon adds install commands to Dockerfile**
If `aibox.toml` includes the `python` addon and `aibox sync` is run, then
`.devcontainer/Dockerfile` must contain Python install instructions.
`[config_coverage.rs · addon_python_in_dockerfile]`

**Rust addon adds rustup install to Dockerfile**
If `aibox.toml` includes the `rust` addon and `aibox sync` is run, then
`.devcontainer/Dockerfile` must contain `rustup` installation instructions.
`[config_coverage.rs · addon_rust_in_dockerfile]`

**Multiple addons each contribute to the Dockerfile**
If `aibox.toml` includes both the `python` and `rust` addons and `aibox sync`
is run, then `.devcontainer/Dockerfile` must contain install content for
both.
`[config_coverage.rs · addon_multiple_in_dockerfile]`

**Core process creates minimal context structure**
If `aibox init --process core` is run, then `CLAUDE.md` and `AIBOX.md` must
exist but no tracking or product files should be created.
`[config_coverage.rs · process_core_creates_minimal_context]`

**Managed process creates backlog and tracking files**
If `aibox init --process managed` is run, then `context/BACKLOG.md` must
exist.
`[config_coverage.rs · process_managed_creates_backlog]`

**Product process creates PRD**
If `aibox init --process full-product` (or the product package) is run,
then `context/PRD.md` must exist.
`[config_coverage.rs · process_product_creates_prd]`

**Research process creates progress file**
If `aibox init --process research-project` (or the research package) is run,
then a research progress file must exist.
`[config_coverage.rs · process_research_creates_progress]`

---

## File preview — `preview.rs`

**svg.yazi plugin is seeded into .aibox-home after init**
If `aibox init` is run, then
`.aibox-home/.config/yazi/plugins/svg.yazi/init.lua` must exist.
`[preview.rs · svg_yazi_plugin_seeded]`

**eps.yazi plugin is seeded into .aibox-home after init**
If `aibox init` is run, then
`.aibox-home/.config/yazi/plugins/eps.yazi/init.lua` must exist.
`[preview.rs · eps_yazi_plugin_seeded]`

**svg.yazi plugin invokes resvg for conversion**
If `svg.yazi/init.lua` is read after init, then its content must reference
`resvg` as the SVG-to-PNG conversion tool.
`[preview.rs · svg_yazi_plugin_uses_resvg]`

**eps.yazi plugin invokes ghostscript for conversion**
If `eps.yazi/init.lua` is read after init, then its content must reference
`gs` (ghostscript) as the EPS-to-PNG conversion tool.
`[preview.rs · eps_yazi_plugin_uses_ghostscript]`

**yazi.toml has a [plugin] section with prepend_previewers**
If `aibox init` is run, then `yazi.toml` must contain a `[plugin]` section
that defines `prepend_previewers`.
`[preview.rs · yazi_toml_has_plugin_section]`

**yazi.toml routes *.svg to the svg previewer**
If `aibox init` is run, then `yazi.toml` must contain a
`prepend_previewers` entry matching `*.svg` with `run = "svg"`.
`[preview.rs · yazi_toml_svg_previewer_entry]`

**yazi.toml routes *.eps to the eps previewer**
If `aibox init` is run, then `yazi.toml` must contain a
`prepend_previewers` entry matching `*.eps` with `run = "eps"`.
`[preview.rs · yazi_toml_eps_previewer_entry]`

**SVG and EPS entries appear before built-in image entries**
If `aibox init` is run, then the `*.svg` and `*.eps` entries in
`prepend_previewers` must appear at a lower byte offset than the `*.jpg`
entry, ensuring first-match semantics dispatch SVG/EPS to the custom
plugins rather than the built-in image previewer.
`[preview.rs · yazi_toml_svg_and_eps_precede_builtin_previewers]`

**sample.svg fixture is valid XML**
If `tests/e2e/fixtures/sample.svg` is read, then its content must start
with `<svg` or `<?xml`, confirming the fixture file is intact.
`[preview.rs · fixture_sample_svg_is_valid_xml]`

**sample.eps fixture has a valid EPS header**
If `tests/e2e/fixtures/sample.eps` is read, then its content must start
with `%!PS-Adobe` or contain `%%BoundingBox`, confirming the fixture file
is intact.
`[preview.rs · fixture_sample_eps_has_eps_header]`

---

## Smoke tests — `smoke.rs`

These tests validate that the Tier 2 companion container's container runtime
is functional end-to-end (Tier 2 only).

**Podman is available on the companion**
If the companion container is queried for `podman --version`, then the
command must succeed and the output must contain `podman`.
`[smoke.rs · podman_available_on_companion]`

**Podman can pull an image**
If `podman pull docker.io/library/alpine:latest` is run on the companion,
then the pull must succeed, confirming rootless registry access works.
`[smoke.rs · podman_can_pull_image]`

**Podman can run a container**
If `podman run --rm alpine echo hello-e2e` is run on the companion, then
the command must succeed and the output must contain `hello-e2e`,
confirming full rootless container execution.
`[smoke.rs · podman_can_run_container]`
