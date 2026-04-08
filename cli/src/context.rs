//! Context and project-root scaffolding that aibox owns directly.
//!
//! After v0.16.0, the bulk of project content (skills, primitives,
//! processes, the canonical `AGENTS.md` template) lives in processkit
//! and is installed at `aibox init` time by [`crate::content_init`] via
//! the install map in [`crate::content_install`]. This module owns only
//! the slice of project setup that is intrinsic to aibox itself:
//!
//! - `.gitignore` (created and kept current with aibox-required entries)
//! - `.aibox-version` (CLI version marker for migrations)
//! - `.devcontainer/Dockerfile.local` and
//!   `.devcontainer/docker-compose.override.yml` placeholders
//! - Provider thin-pointer files at the project root (`CLAUDE.md`,
//!   future `CODEX.md`, …) that point at processkit-shipped `AGENTS.md`,
//!   gated on the `[ai].providers` list
//! - The empty `context/` directory itself (processkit content lands here
//!   later, during the same init pass)
//!
//! Everything else — `BACKLOG.md`, `DECISIONS.md`, `STANDUPS.md`, work
//! instructions, the canonical `AGENTS.md`, all 100+ skills — is owned
//! by processkit and arrives via the content-source install pipeline.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{AddonsSection, AiProvider, AiboxConfig};
use crate::output;

// ---------------------------------------------------------------------------
// Provider thin-pointer template
// ---------------------------------------------------------------------------

/// Thin-pointer body written to `CLAUDE.md` when the user has
/// `[ai] providers = ["claude", ...]`. The canonical instructions live
/// in `AGENTS.md` (shipped by processkit). Claude Code auto-loads
/// `CLAUDE.md`, so this pointer file exists solely to satisfy that
/// convention without duplicating instructions.
const CLAUDE_POINTER_TEMPLATE: &str = r#"# CLAUDE.md — {{PROJECT_NAME}}

> **Pointer file.** Canonical instructions live in [`AGENTS.md`](./AGENTS.md).
>
> Claude Code auto-loads `CLAUDE.md` on startup, so this thin file exists
> only to satisfy that convention. Edit `AGENTS.md` (shipped by
> processkit) — not this file. Any changes here will not be picked up by
> other agent harnesses (Codex CLI, Continue, OpenCode, …).

Read **[`AGENTS.md`](./AGENTS.md)** in the project root for project
instructions. It is the single, provider-neutral entry point for any AI
agent (or human) working on this project.
"#;

// ---------------------------------------------------------------------------
// Template rendering — Class A placeholder vocabulary (DEC-032 / BACK-117)
// ---------------------------------------------------------------------------

/// Render a template by substituting `{{NAME}}` placeholders from a
/// vocabulary map. **Unknown placeholders are left literal** so the
/// agent's onboarding protocol can find and fill them with the project
/// owner — see DEC-032 for the three-class model (A: aibox-rendered,
/// B: owner-supplied, C: discoverable).
///
/// Implementation: simple `str::replace` per known key. Class B/C
/// placeholders aren't in the map, so they survive unchanged.
pub(crate) fn render(template: &str, vars: &HashMap<&'static str, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        out = out.replace(&placeholder, value);
    }
    out
}

/// Build the Class A substitution vocabulary from an `AiboxConfig`.
///
/// The 11 entries below are the contract aibox commits to render in
/// any installed file marked as templated. Adding a new entry is a
/// coordination step (requires an aibox release); processkit's
/// AGENTS.md template can use these freely. Class B (owner-supplied)
/// and Class C (discoverable) placeholders are NOT in the map and
/// survive `render()` unchanged.
///
/// See DEC-032 for the design and the full rationale.
pub(crate) fn build_substitution_map(
    config: &AiboxConfig,
) -> HashMap<&'static str, String> {
    let mut m: HashMap<&'static str, String> = HashMap::new();

    // 1. PROJECT_NAME — the container name doubles as the project name.
    m.insert("PROJECT_NAME", config.container.name.clone());

    // 2. CONTAINER_HOSTNAME
    m.insert("CONTAINER_HOSTNAME", config.container.hostname.clone());

    // 3. CONTAINER_USER — the Unix user inside the container.
    m.insert("CONTAINER_USER", config.container.user.clone());

    // 4. AIBOX_VERSION — the CLI version that performed the install.
    //    Re-evaluated on every `aibox sync` (because render runs at
    //    install time, not at template-author time).
    m.insert("AIBOX_VERSION", env!("CARGO_PKG_VERSION").to_string());

    // 5. AIBOX_BASE — base image flavor (currently always "debian").
    m.insert("AIBOX_BASE", config.aibox.base.to_string());

    // 6. PROCESSKIT_SOURCE — full URL of the configured upstream.
    m.insert("PROCESSKIT_SOURCE", config.processkit.source.clone());

    // 7. PROCESSKIT_VERSION — pinned tag, or empty string when unset.
    let pk_version = if config.processkit.version
        == crate::config::PROCESSKIT_VERSION_UNSET
    {
        String::new()
    } else {
        config.processkit.version.clone()
    };
    m.insert("PROCESSKIT_VERSION", pk_version);

    // 8. INSTALL_DATE — current date at render time, YYYY-MM-DD.
    m.insert(
        "INSTALL_DATE",
        chrono::Utc::now().format("%Y-%m-%d").to_string(),
    );

    // 9. ADDONS — sorted, comma-separated [addons.X] keys.
    let mut addon_names: Vec<String> = config.addons.addons.keys().cloned().collect();
    addon_names.sort();
    m.insert("ADDONS", addon_names.join(", "));

    // 10. AI_PROVIDERS — sorted, comma-separated [ai].providers.
    let mut providers: Vec<String> =
        config.ai.providers.iter().map(|p| p.to_string()).collect();
    providers.sort();
    m.insert("AI_PROVIDERS", providers.join(", "));

    // 11. CONTEXT_PACKAGES — comma-separated [context].packages.
    m.insert(
        "CONTEXT_PACKAGES",
        config.context.packages.join(", "),
    );

    m
}

// ---------------------------------------------------------------------------
// File I/O helpers
// ---------------------------------------------------------------------------

/// Write `content` to `path` only if the file does not already exist.
/// Creates parent directories as needed.
pub(crate) fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        tracing::debug!("Skipping existing file: {}", path.display());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write: {}", path.display()))?;
    Ok(())
}

/// Write `content` to `path` only if it differs from the current contents.
/// Creates parent directories as needed. Returns `true` if a write happened.
pub(crate) fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if existing == content {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Project scaffolding (init time)
// ---------------------------------------------------------------------------

/// Set up the aibox-owned slice of a project: provider thin pointers,
/// the empty `context/` directory, `.aibox-version`, `.gitignore`, and
/// the user-owned `.devcontainer/` overlay placeholders.
///
/// Called from `cmd_init` *before* [`crate::content_init::install_content_source`]
/// installs processkit content. The two layers compose: aibox sets up
/// the bare project skeleton, then processkit fills it with skills,
/// primitives, processes, and the canonical `AGENTS.md`.
pub fn scaffold_context(config: &AiboxConfig) -> Result<()> {
    let addons = &config.addons;

    output::info("Scaffolding project skeleton...");

    // 1. Provider thin pointers (CLAUDE.md, future CODEX.md, …) per
    //    [ai].providers. The pointers reference AGENTS.md, which
    //    processkit installs in the same init pass.
    scaffold_provider_pointers(config)?;

    // 2. context/ directory exists so processkit content has a home.
    //    A .gitkeep keeps the directory present until processkit
    //    populates it.
    let context = Path::new("context");
    fs::create_dir_all(context).context("Failed to create context/")?;
    write_if_missing(&context.join(".gitkeep"), "")?;

    // 3. .aibox-version — used by migration detection on subsequent syncs.
    write_if_missing(Path::new(".aibox-version"), env!("CARGO_PKG_VERSION"))?;
    output::ok("Created .aibox-version");

    // 4. .gitignore — aibox entries plus language-specific blocks based
    //    on the configured addons.
    update_gitignore(addons)?;

    // 5. User-owned .devcontainer/ overlay placeholders. Never
    //    overwritten on re-init or sync.
    let local_dockerfile = Path::new(crate::config::DEVCONTAINER_DIR).join("Dockerfile.local");
    write_if_missing(
        &local_dockerfile,
        "# Project-specific Dockerfile layers.\n\
         # This file is appended to the generated Dockerfile by `aibox sync`.\n\
         # It is never overwritten — you own this file.\n\
         #\n\
         # The generated base image is available as the \"aibox\" stage:\n\
         #   FROM ghcr.io/projectious-work/aibox:<image>-v<version> AS aibox\n\
         #\n\
         # Simple usage — add layers directly:\n\
         #   RUN apt-get update && apt-get install -y some-package\n\
         #   RUN npx playwright install --with-deps chromium\n\
         #\n\
         # Advanced usage — multi-stage build referencing the aibox stage:\n\
         #   FROM node:20 AS builder\n\
         #   RUN npm ci && npm run build\n\
         #\n\
         #   FROM aibox\n\
         #   COPY --from=builder /app/dist /workspace/dist\n",
    )?;

    let compose_override =
        Path::new(crate::config::DEVCONTAINER_DIR).join("docker-compose.override.yml");
    write_if_missing(
        &compose_override,
        "# Docker Compose override — project-specific services and overrides.\n\
         # This file is never overwritten by `aibox sync`. You own it.\n\
         #\n\
         # Docker Compose automatically merges this with the generated\n\
         # docker-compose.yml (strategic merge by service name).\n\
         # When present, `aibox sync` wires it into devcontainer.json.\n\
         #\n\
         # Example — add a PostgreSQL sidecar:\n\
         #\n\
         #   services:\n\
         #     postgres:\n\
         #       image: postgres:16\n\
         #       environment:\n\
         #         POSTGRES_PASSWORD: dev\n\
         #       ports:\n\
         #         - \"5432:5432\"\n\
         #\n\
         # Example — add depends_on to the main service:\n\
         #\n\
         #   services:\n\
         #     my-project:            # must match [container] name in aibox.toml\n\
         #       depends_on:\n\
         #         - postgres\n",
    )?;

    output::ok("Project skeleton ready");
    Ok(())
}

/// Write thin-pointer entry files for each AI provider in
/// `[ai].providers` that has a markdown convention. Today only Claude
/// Code uses a top-level `CLAUDE.md`; other providers (Aider, Gemini,
/// Mistral) use config files (`.aider.conf.yml`, `.gemini/settings.json`,
/// `.mistral/config.json`) which are scaffolded elsewhere.
///
/// The pointer points at `AGENTS.md`, which processkit installs into
/// the project root in the same init pass.
fn scaffold_provider_pointers(config: &AiboxConfig) -> Result<()> {
    let vars = build_substitution_map(config);
    for provider in &config.ai.providers {
        match provider {
            AiProvider::Claude => {
                let body = render(CLAUDE_POINTER_TEMPLATE, &vars);
                write_if_missing(Path::new("CLAUDE.md"), &body)?;
                output::ok("Created CLAUDE.md (pointer to AGENTS.md)");
            }
            // Aider/Gemini/Mistral don't use a top-level markdown file.
            AiProvider::Aider | AiProvider::Gemini | AiProvider::Mistral => {}
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// .gitignore management
// ---------------------------------------------------------------------------

/// Generate a `.gitignore` with aibox entries, project-specific section,
/// and language-specific blocks based on the configured addons.
pub(crate) fn update_gitignore(addons: &AddonsSection) -> Result<()> {
    let gitignore_path = Path::new(".gitignore");

    // If .gitignore already exists, just ensure aibox entries are present.
    if gitignore_path.exists() {
        return ensure_aibox_entries(gitignore_path);
    }

    // Create a new .gitignore with full structure.
    let mut content = String::new();

    // Project-specific section
    content.push_str(
        "# ── Project-specific ─────────────────────────────────────────────────────────\n",
    );
    content.push_str("# Add your project-specific ignore patterns here.\n\n\n");

    // aibox generated
    content.push_str(
        "# ── aibox generated ────────────────────────────────────────────────────────\n",
    );
    content.push_str("# Files generated by aibox — do not remove these entries.\n");
    content.push_str(".aibox-home/\n");
    content.push_str(".root/\n");
    content.push_str(".aibox-version\n");
    content.push_str(".aibox/\n");
    content.push_str(".aibox-backup/\n");
    content.push_str(".aibox-env/\n");
    // Runtime cache for fetched processkit / aibox content. Reproducible
    // from aibox.lock; never tracked.
    content.push_str("context/.cache/\n\n");

    // Privacy tier (DEC-030): any directory named `private` anywhere
    // under context/ is excluded recursively, no matter how deep. Lets
    // a user keep personal notes, drafts, and per-user state alongside
    // the project-public context content without risk of leaking it.
    content.push_str(
        "# ── Privacy tier (DEC-030) ──────────────────────────────────────────────────\n",
    );
    content.push_str("# Any directory named `private` under context/ is never tracked.\n");
    content.push_str("# Use this for personal notes, drafts, secrets, and per-user state.\n");
    content.push_str("context/**/private/\n");
    content.push_str("context/private/\n\n");

    // OS generated
    content.push_str(
        "# ── OS generated files ───────────────────────────────────────────────────────\n",
    );
    content.push_str(".DS_Store\n");
    content.push_str(".DS_Store?\n");
    content.push_str("._*\n");
    content.push_str(".Spotlight-V100\n");
    content.push_str(".Trashes\n");
    content.push_str("Thumbs.db\n");
    content.push_str("ehthumbs.db\n\n");

    // Editor/IDE
    content.push_str(
        "# ── Editor / IDE ─────────────────────────────────────────────────────────────\n",
    );
    content.push_str("*.swp\n");
    content.push_str("*.swo\n");
    content.push_str("*~\n");
    content.push_str(".idea/\n\n");

    // Language-specific blocks based on configured addons
    if addons.has_python() {
        content.push_str(
            "# ── Python ───────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("__pycache__/\n");
        content.push_str("*.py[cod]\n");
        content.push_str("*$py.class\n");
        content.push_str("*.egg-info/\n");
        content.push_str("*.egg\n");
        content.push_str("dist/\n");
        content.push_str("build/\n");
        content.push_str(".eggs/\n");
        content.push_str(".venv/\n");
        content.push_str("venv/\n");
        content.push_str(".pytest_cache/\n");
        content.push_str(".mypy_cache/\n");
        content.push_str(".ruff_cache/\n");
        content.push_str("htmlcov/\n");
        content.push_str(".coverage\n");
        content.push_str(".coverage.*\n");
        content.push_str("site/\n\n");
    }

    if addons.has_latex() {
        content.push_str(
            "# ── LaTeX ────────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("*.aux\n");
        content.push_str("*.bbl\n");
        content.push_str("*.blg\n");
        content.push_str("*.fdb_latexmk\n");
        content.push_str("*.fls\n");
        content.push_str("*.lof\n");
        content.push_str("*.log\n");
        content.push_str("*.lot\n");
        content.push_str("*.out\n");
        content.push_str("*.toc\n");
        content.push_str("*.synctex.gz\n");
        content.push_str("*.nav\n");
        content.push_str("*.snm\n");
        content.push_str("*.vrb\n");
        content.push_str("*.bcf\n");
        content.push_str("*.run.xml\n");
        content.push_str("out/\n\n");
    }

    if addons.has_addon("typst") {
        content.push_str(
            "# ── Typst ────────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("# Typst produces PDFs directly — ignore build outputs if applicable\n\n");
    }

    if addons.has_rust() {
        content.push_str(
            "# ── Rust ─────────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("target/\n");
        content.push_str("Cargo.lock\n\n");
    }

    if addons.has_node() {
        content.push_str(
            "# ── Node.js ──────────────────────────────────────────────────────────────────\n",
        );
        content.push_str("node_modules/\n");
        content.push_str(".next/\n");
        content.push_str("dist/\n");
        content.push_str(".env.local\n");
        content.push_str(".env.*.local\n");
        content.push_str(".nuxt/\n");
        content.push_str(".output/\n");
        content.push_str(".cache/\n");
        content.push_str("coverage/\n\n");
    }

    fs::write(gitignore_path, content).context("Failed to write .gitignore")?;
    output::ok("Created .gitignore with aibox and language-specific entries");

    Ok(())
}

/// Ensure aibox entries exist in an existing `.gitignore`. Append any
/// that are missing without disturbing user-authored content.
fn ensure_aibox_entries(gitignore_path: &Path) -> Result<()> {
    let required_entries = [
        "# aibox generated",
        ".aibox-home/",
        ".aibox-version",
        ".aibox-backup/",
        ".aibox-env/",
        // Runtime cache for fetched processkit / aibox content.
        // Reproducible from aibox.lock; never tracked.
        "context/.cache/",
        // Privacy tier (DEC-030): any directory named `private` under
        // context/ is never tracked. Two patterns to cover both the
        // top-level case and arbitrary nesting depth.
        "context/private/",
        "context/**/private/",
    ];

    let existing = fs::read_to_string(gitignore_path).context("Failed to read .gitignore")?;
    let existing_lines: Vec<&str> = existing.lines().map(|l| l.trim()).collect();

    let mut additions = Vec::new();
    for entry in &required_entries {
        if !existing_lines.contains(entry) {
            // Backward compat: tolerate the legacy `.root/` name.
            if *entry == ".aibox-home/" && existing_lines.contains(&".root/") {
                continue;
            }
            additions.push(*entry);
        }
    }

    if additions.is_empty() {
        return Ok(());
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(&additions.join("\n"));
    content.push('\n');

    fs::write(gitignore_path, content).context("Failed to write .gitignore")?;
    output::ok("Updated .gitignore with aibox entries");

    Ok(())
}

/// Check that `.gitignore` has the required aibox entries. Used by `aibox doctor`.
pub fn check_gitignore_entries() -> Vec<String> {
    let gitignore_path = Path::new(".gitignore");
    let mut warnings = Vec::new();

    if !gitignore_path.exists() {
        warnings.push(".gitignore not found — run 'aibox init' or create one".to_string());
        return warnings;
    }

    let content = match fs::read_to_string(gitignore_path) {
        Ok(c) => c,
        Err(_) => {
            warnings.push("Could not read .gitignore".to_string());
            return warnings;
        }
    };

    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();

    let required = [
        (".devcontainer/Dockerfile", "generated Dockerfile"),
        (
            ".devcontainer/docker-compose.yml",
            "generated docker-compose",
        ),
        (
            ".devcontainer/devcontainer.json",
            "generated devcontainer.json",
        ),
        (".aibox-version", "version lockfile"),
    ];

    for (entry, desc) in &required {
        if !lines.contains(entry) {
            warnings.push(format!(".gitignore missing '{}' ({})", entry, desc));
        }
    }

    // Either `.aibox-home/` or the legacy `.root/` is acceptable.
    if !lines.contains(&".aibox-home/") && !lines.contains(&".root/") {
        warnings
            .push(".gitignore missing '.aibox-home/' (persisted config directory)".to_string());
    }

    warnings
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Run a closure inside a temp directory, restoring cwd afterwards.
    fn in_temp_dir<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        f();
        let _ = std::env::set_current_dir(&original);
    }

    /// Helper: build a tiny vars map for tests that don't need a full
    /// `AiboxConfig`.
    fn vars(pairs: &[(&'static str, &str)]) -> HashMap<&'static str, String> {
        pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn render_replaces_known_class_a_placeholder() {
        let v = vars(&[("PROJECT_NAME", "my-app")]);
        assert_eq!(
            render("Hello {{PROJECT_NAME}}!", &v),
            "Hello my-app!"
        );
    }

    #[test]
    fn render_replaces_multiple_occurrences() {
        let v = vars(&[("PROJECT_NAME", "foo")]);
        assert_eq!(
            render("{{PROJECT_NAME}} is {{PROJECT_NAME}}", &v),
            "foo is foo"
        );
    }

    #[test]
    fn render_leaves_unknown_placeholders_literal() {
        // Class B/C placeholders are filled by processkit's onboarding
        // skill in conversation with the project owner — aibox MUST
        // pass them through untouched.
        let v = vars(&[("PROJECT_NAME", "foo")]);
        let out = render(
            "{{PROJECT_NAME}} — {{PROJECT_DESCRIPTION}} ({{BUILD_COMMAND}})",
            &v,
        );
        assert_eq!(
            out,
            "foo — {{PROJECT_DESCRIPTION}} ({{BUILD_COMMAND}})"
        );
    }

    #[test]
    fn render_leaves_lowercase_legacy_placeholder_literal() {
        // The pre-v0.16.4 lowercase {{project_name}} alias was dropped
        // (per session decision: clean up, no version compatibility).
        // Existing pointer files using the lowercase form will need
        // re-init.
        let v = vars(&[("PROJECT_NAME", "foo")]);
        assert_eq!(
            render("{{project_name}}", &v),
            "{{project_name}}",
            "lowercase alias was deliberately removed"
        );
    }

    #[test]
    fn render_handles_empty_template() {
        let v = vars(&[("PROJECT_NAME", "foo")]);
        assert_eq!(render("", &v), "");
    }

    #[test]
    fn render_handles_template_with_no_placeholders() {
        let v = vars(&[("PROJECT_NAME", "foo")]);
        assert_eq!(
            render("Plain text with no markers.", &v),
            "Plain text with no markers."
        );
    }

    #[test]
    fn build_substitution_map_includes_all_class_a_keys() {
        // The 11-key Class A vocabulary is the contract aibox commits
        // to render. Adding/removing entries is breaking and requires
        // a coordination step with processkit (DEC-032).
        let config = crate::config::test_config();
        let map = build_substitution_map(&config);
        for key in [
            "PROJECT_NAME",
            "CONTAINER_HOSTNAME",
            "CONTAINER_USER",
            "AIBOX_VERSION",
            "AIBOX_BASE",
            "PROCESSKIT_SOURCE",
            "PROCESSKIT_VERSION",
            "INSTALL_DATE",
            "ADDONS",
            "AI_PROVIDERS",
            "CONTEXT_PACKAGES",
        ] {
            assert!(
                map.contains_key(key),
                "Class A vocabulary missing key: {}",
                key
            );
        }
        assert_eq!(
            map.len(),
            11,
            "Class A vocabulary must be exactly 11 entries; \
             update DEC-032 + RELEASE-NOTES + this test together"
        );
    }

    #[test]
    fn build_substitution_map_processkit_version_unset_renders_empty() {
        let mut config = crate::config::test_config();
        config.processkit.version = crate::config::PROCESSKIT_VERSION_UNSET.to_string();
        let map = build_substitution_map(&config);
        assert_eq!(map["PROCESSKIT_VERSION"], "");
    }

    #[test]
    fn build_substitution_map_install_date_is_yyyy_mm_dd() {
        let config = crate::config::test_config();
        let map = build_substitution_map(&config);
        let date = &map["INSTALL_DATE"];
        // YYYY-MM-DD: 10 chars, two dashes at fixed positions
        assert_eq!(date.len(), 10, "INSTALL_DATE should be YYYY-MM-DD");
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn write_if_missing_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        write_if_missing(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn write_if_missing_does_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();
        write_if_missing(&path, "new content").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    }

    #[test]
    fn write_if_missing_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("c.txt");
        write_if_missing(&path, "deep").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "deep");
    }

    #[test]
    fn write_if_changed_writes_first_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.txt");
        let written = write_if_changed(&path, "v1").unwrap();
        assert!(written);
        assert_eq!(fs::read_to_string(&path).unwrap(), "v1");
    }

    #[test]
    fn write_if_changed_no_op_when_identical() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.txt");
        fs::write(&path, "v1").unwrap();
        let written = write_if_changed(&path, "v1").unwrap();
        assert!(!written);
    }

    #[test]
    fn write_if_changed_overwrites_when_different() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.txt");
        fs::write(&path, "v1").unwrap();
        let written = write_if_changed(&path, "v2").unwrap();
        assert!(written);
        assert_eq!(fs::read_to_string(&path).unwrap(), "v2");
    }

    // ── scaffold_context ────────────────────────────────────────────────

    #[test]
    #[serial]
    fn scaffold_creates_aibox_version_and_context_dir() {
        in_temp_dir(|| {
            let mut config = crate::config::test_config();
            // Make .devcontainer/ exist so the placeholders can land.
            fs::create_dir_all(crate::config::DEVCONTAINER_DIR).unwrap();
            // Default test_config has Claude as a provider.
            config.container.name = "scaffold-test".to_string();
            scaffold_context(&config).unwrap();
            assert!(Path::new(".aibox-version").exists());
            assert!(Path::new("context").is_dir());
            assert!(Path::new(".gitignore").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_writes_thin_claude_pointer_when_claude_enabled() {
        in_temp_dir(|| {
            fs::create_dir_all(crate::config::DEVCONTAINER_DIR).unwrap();
            let config = crate::config::test_config();
            scaffold_context(&config).unwrap();
            let body = fs::read_to_string("CLAUDE.md").unwrap();
            assert!(
                body.contains("Pointer file"),
                "default mode should write a thin pointer CLAUDE.md"
            );
            assert!(
                body.contains("AGENTS.md"),
                "thin pointer must reference AGENTS.md"
            );
        });
    }

    #[test]
    #[serial]
    fn scaffold_does_not_overwrite_existing_claude_md() {
        in_temp_dir(|| {
            fs::create_dir_all(crate::config::DEVCONTAINER_DIR).unwrap();
            let user = "# CLAUDE.md\n\nMy hand-written instructions.\n";
            fs::write("CLAUDE.md", user).unwrap();
            let config = crate::config::test_config();
            scaffold_context(&config).unwrap();
            let after = fs::read_to_string("CLAUDE.md").unwrap();
            assert_eq!(after, user, "user CLAUDE.md must be preserved");
        });
    }

    #[test]
    #[serial]
    fn scaffold_skips_claude_pointer_when_provider_not_listed() {
        in_temp_dir(|| {
            fs::create_dir_all(crate::config::DEVCONTAINER_DIR).unwrap();
            let mut config = crate::config::test_config();
            config.ai.providers = vec![AiProvider::Aider];
            scaffold_context(&config).unwrap();
            assert!(
                !Path::new("CLAUDE.md").exists(),
                "no CLAUDE.md should be written when Claude is not in providers"
            );
        });
    }

    // ── update_gitignore ────────────────────────────────────────────────

    #[test]
    #[serial]
    fn update_gitignore_creates_when_missing() {
        in_temp_dir(|| {
            let addons = AddonsSection::default();
            update_gitignore(&addons).unwrap();
            let body = fs::read_to_string(".gitignore").unwrap();
            assert!(body.contains(".aibox-home/"));
            assert!(body.contains("context/.cache/"));
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_creates_privacy_tier_rules() {
        in_temp_dir(|| {
            let addons = AddonsSection::default();
            update_gitignore(&addons).unwrap();
            let body = fs::read_to_string(".gitignore").unwrap();
            assert!(
                body.contains("context/private/"),
                "missing top-level private/ rule"
            );
            assert!(
                body.contains("context/**/private/"),
                "missing recursive private/ rule"
            );
            assert!(
                body.contains("DEC-030"),
                "should reference the formal decision"
            );
        });
    }

    #[test]
    #[serial]
    fn ensure_aibox_entries_appends_privacy_tier_to_existing() {
        in_temp_dir(|| {
            // Pre-existing .gitignore with the v0.16.2 entries but no privacy rule.
            fs::write(
                ".gitignore",
                "# user\nmy-secret\n\n# aibox generated\n.aibox-home/\n.aibox-version\n.aibox-backup/\n.aibox-env/\ncontext/.cache/\n",
            )
            .unwrap();
            let addons = AddonsSection::default();
            update_gitignore(&addons).unwrap();
            let body = fs::read_to_string(".gitignore").unwrap();
            assert!(body.contains("my-secret"), "user entries preserved");
            assert!(
                body.contains("context/private/"),
                "privacy rule appended on upgrade"
            );
            assert!(body.contains("context/**/private/"));
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_appends_aibox_entries_to_existing() {
        in_temp_dir(|| {
            fs::write(".gitignore", "# user\nmy-secret\n").unwrap();
            let addons = AddonsSection::default();
            update_gitignore(&addons).unwrap();
            let body = fs::read_to_string(".gitignore").unwrap();
            assert!(body.contains("my-secret"), "user entries preserved");
            assert!(body.contains(".aibox-home/"));
            assert!(body.contains("context/.cache/"));
        });
    }

    #[test]
    #[serial]
    fn check_gitignore_entries_warns_when_missing() {
        in_temp_dir(|| {
            let warnings = check_gitignore_entries();
            assert!(!warnings.is_empty());
            assert!(warnings[0].contains(".gitignore not found"));
        });
    }
}
