use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::{AddonsSection, DevBoxConfig};
use crate::output;
use crate::process_registry;

// --- Minimal templates ---
const MINIMAL_CLAUDE_MD: &str = include_str!("../../templates/minimal/CLAUDE.md.template");

// --- Managed templates ---
const MANAGED_CLAUDE_MD: &str = include_str!("../../templates/managed/CLAUDE.md.template");
const MANAGED_DECISIONS: &str = include_str!("../../templates/managed/DECISIONS.md");
const MANAGED_BACKLOG: &str = include_str!("../../templates/managed/BACKLOG.md");
const MANAGED_STANDUPS: &str = include_str!("../../templates/managed/STANDUPS.md");
const MANAGED_GENERAL: &str = include_str!("../../templates/managed/work-instructions/GENERAL.md");

// --- Research templates ---
const RESEARCH_CLAUDE_MD: &str = include_str!("../../templates/research/CLAUDE.md.template");
const RESEARCH_PROGRESS: &str = include_str!("../../templates/research/PROGRESS.md");
const RESEARCH_NOTE_TEMPLATE: &str = include_str!("../../templates/research/research-note.md");
const EXPERIMENTS_README: &str = include_str!("../../templates/research/experiments-README.md");

// --- Product templates ---
const PRODUCT_CLAUDE_MD: &str = include_str!("../../templates/product/CLAUDE.md.template");
const PRODUCT_DECISIONS: &str = include_str!("../../templates/product/DECISIONS.md");
const PRODUCT_BACKLOG: &str = include_str!("../../templates/product/BACKLOG.md");
const PRODUCT_STANDUPS: &str = include_str!("../../templates/product/STANDUPS.md");
const PRODUCT_PROJECTS: &str = include_str!("../../templates/product/PROJECTS.md");
const PRODUCT_PRD: &str = include_str!("../../templates/product/PRD.md");
const PRODUCT_GENERAL: &str = include_str!("../../templates/product/work-instructions/GENERAL.md");
const PRODUCT_DEVELOPMENT: &str =
    include_str!("../../templates/product/work-instructions/DEVELOPMENT.md");
const PRODUCT_TEAM: &str = include_str!("../../templates/product/work-instructions/TEAM.md");

// --- Process templates ---
const PROCESS_README: &str = include_str!("../../templates/processes/README.md");
const PROCESS_RELEASE: &str = include_str!("../../templates/processes/release.md");
const PROCESS_CODE_REVIEW: &str = include_str!("../../templates/processes/code-review.md");
const PROCESS_FEATURE_DEV: &str = include_str!("../../templates/processes/feature-development.md");
const PROCESS_BUG_FIX: &str = include_str!("../../templates/processes/bug-fix.md");

// --- Skill templates ---
const SKILL_BACKLOG_CONTEXT: &str =
    include_str!("../../templates/skills/backlog-context/SKILL.md");
const SKILL_DECISIONS_ADR: &str = include_str!("../../templates/skills/decisions-adr/SKILL.md");
const SKILL_STANDUP_CONTEXT: &str =
    include_str!("../../templates/skills/standup-context/SKILL.md");

// Development skills
const SKILL_CODE_REVIEW: &str = include_str!("../../templates/skills/code-review/SKILL.md");
const SKILL_TESTING_STRATEGY: &str =
    include_str!("../../templates/skills/testing-strategy/SKILL.md");
const SKILL_REFACTORING: &str = include_str!("../../templates/skills/refactoring/SKILL.md");
const SKILL_DOCUMENTATION: &str = include_str!("../../templates/skills/documentation/SKILL.md");
const SKILL_DEBUGGING: &str = include_str!("../../templates/skills/debugging/SKILL.md");

// Process skills
const SKILL_RELEASE_SEMVER: &str = include_str!("../../templates/skills/release-semver/SKILL.md");
const SKILL_INCIDENT_RESPONSE: &str =
    include_str!("../../templates/skills/incident-response/SKILL.md");
const SKILL_RETROSPECTIVE: &str = include_str!("../../templates/skills/retrospective/SKILL.md");

// Language-specific skills
const SKILL_PYTHON_BEST_PRACTICES: &str =
    include_str!("../../templates/skills/python-best-practices/SKILL.md");
const SKILL_RUST_CONVENTIONS: &str =
    include_str!("../../templates/skills/rust-conventions/SKILL.md");
const SKILL_LATEX_AUTHORING: &str = include_str!("../../templates/skills/latex-authoring/SKILL.md");
const SKILL_TYPESCRIPT_PATTERNS: &str =
    include_str!("../../templates/skills/typescript-patterns/SKILL.md");

// Infrastructure skills
const SKILL_DOCKERFILE_REVIEW: &str =
    include_str!("../../templates/skills/dockerfile-review/SKILL.md");
const SKILL_GIT_WORKFLOW: &str = include_str!("../../templates/skills/git-workflow/SKILL.md");
const SKILL_CI_CD_SETUP: &str = include_str!("../../templates/skills/ci-cd-setup/SKILL.md");

// Design & visual skills
const SKILL_EXCALIDRAW: &str = include_str!("../../templates/skills/excalidraw/SKILL.md");
const SKILL_FRONTEND_DESIGN: &str =
    include_str!("../../templates/skills/frontend-design/SKILL.md");
const SKILL_INFOGRAPHICS: &str = include_str!("../../templates/skills/infographics/SKILL.md");
const SKILL_LOGO_DESIGN: &str = include_str!("../../templates/skills/logo-design/SKILL.md");
const SKILL_TAILWIND: &str = include_str!("../../templates/skills/tailwind/SKILL.md");

// Architecture skills
const SKILL_SOFTWARE_ARCHITECTURE: &str =
    include_str!("../../templates/skills/software-architecture/SKILL.md");

// Security skills
const SKILL_DEPENDENCY_AUDIT: &str =
    include_str!("../../templates/skills/dependency-audit/SKILL.md");
const SKILL_SECRET_MANAGEMENT: &str =
    include_str!("../../templates/skills/secret-management/SKILL.md");

// --- Reference files for existing skills ---
const SKILL_REFACTORING_REF_CODE_SMELLS: &str =
    include_str!("../../templates/skills/refactoring/references/code-smells.md");
const SKILL_REFACTORING_REF_GOF_PATTERNS: &str =
    include_str!("../../templates/skills/refactoring/references/gof-patterns.md");
const SKILL_LATEX_AUTHORING_REF_PACKAGES: &str =
    include_str!("../../templates/skills/latex-authoring/references/packages.md");
const SKILL_LATEX_AUTHORING_REF_MATH: &str =
    include_str!("../../templates/skills/latex-authoring/references/math-reference.md");
const SKILL_LATEX_AUTHORING_REF_TIKZ: &str =
    include_str!("../../templates/skills/latex-authoring/references/tikz-reference.md");
const SKILL_EXCALIDRAW_REF_JSON_SCHEMA: &str =
    include_str!("../../templates/skills/excalidraw/references/json-schema.md");
const SKILL_FRONTEND_DESIGN_REF_A11Y: &str =
    include_str!("../../templates/skills/frontend-design/references/accessibility-checklist.md");
const SKILL_INFOGRAPHICS_REF_BEST_PRACTICES: &str =
    include_str!("../../templates/skills/infographics/references/best-practices.md");
const SKILL_LOGO_DESIGN_REF_DESIGN_PRINCIPLES: &str =
    include_str!("../../templates/skills/logo-design/references/design-principles.md");
const SKILL_TAILWIND_REF_CHEATSHEET: &str =
    include_str!("../../templates/skills/tailwind/references/cheatsheet.md");
const SKILL_SOFTWARE_ARCHITECTURE_REF_PATTERNS: &str =
    include_str!("../../templates/skills/software-architecture/references/patterns.md");

// --- New skills (Phase 1-4) ---

// Process skills (new)
const SKILL_AGENT_MANAGEMENT: &str =
    include_str!("../../templates/skills/agent-management/SKILL.md");
const SKILL_AGENT_MANAGEMENT_REF_COORDINATION: &str =
    include_str!("../../templates/skills/agent-management/references/coordination-patterns.md");
const SKILL_ESTIMATION_PLANNING: &str =
    include_str!("../../templates/skills/estimation-planning/SKILL.md");
const SKILL_POSTMORTEM_WRITING: &str =
    include_str!("../../templates/skills/postmortem-writing/SKILL.md");

// Development skills (new)
const SKILL_TDD_WORKFLOW: &str = include_str!("../../templates/skills/tdd-workflow/SKILL.md");
const SKILL_INTEGRATION_TESTING: &str =
    include_str!("../../templates/skills/integration-testing/SKILL.md");
const SKILL_INTEGRATION_TESTING_REF_FIXTURES: &str =
    include_str!("../../templates/skills/integration-testing/references/test-fixtures.md");
const SKILL_ERROR_HANDLING: &str =
    include_str!("../../templates/skills/error-handling/SKILL.md");
const SKILL_DEPENDENCY_MANAGEMENT: &str =
    include_str!("../../templates/skills/dependency-management/SKILL.md");
const SKILL_CODE_GENERATION: &str =
    include_str!("../../templates/skills/code-generation/SKILL.md");

// Language skills (new)
const SKILL_GO_CONVENTIONS: &str =
    include_str!("../../templates/skills/go-conventions/SKILL.md");
const SKILL_GO_CONVENTIONS_REF_PATTERNS: &str =
    include_str!("../../templates/skills/go-conventions/references/go-patterns.md");
const SKILL_JAVA_PATTERNS: &str =
    include_str!("../../templates/skills/java-patterns/SKILL.md");
const SKILL_SQL_STYLE_GUIDE: &str =
    include_str!("../../templates/skills/sql-style-guide/SKILL.md");

// Infrastructure skills (new)
const SKILL_KUBERNETES_BASICS: &str =
    include_str!("../../templates/skills/kubernetes-basics/SKILL.md");
const SKILL_KUBERNETES_REF_ARCHITECTURE: &str =
    include_str!("../../templates/skills/kubernetes-basics/references/cluster-architecture.md");
const SKILL_KUBERNETES_REF_RESOURCES: &str =
    include_str!("../../templates/skills/kubernetes-basics/references/resource-cheatsheet.md");
const SKILL_KUBERNETES_REF_TROUBLESHOOTING: &str =
    include_str!("../../templates/skills/kubernetes-basics/references/troubleshooting.md");
const SKILL_DNS_NETWORKING: &str =
    include_str!("../../templates/skills/dns-networking/SKILL.md");
const SKILL_DNS_NETWORKING_REF_PROTOCOL: &str =
    include_str!("../../templates/skills/dns-networking/references/protocol-reference.md");
const SKILL_DNS_NETWORKING_REF_TOOLS: &str =
    include_str!("../../templates/skills/dns-networking/references/troubleshooting-tools.md");
const SKILL_TERRAFORM_BASICS: &str =
    include_str!("../../templates/skills/terraform-basics/SKILL.md");
const SKILL_CONTAINER_ORCHESTRATION: &str =
    include_str!("../../templates/skills/container-orchestration/SKILL.md");
const SKILL_CONTAINER_ORCHESTRATION_REF_COMPOSE: &str =
    include_str!("../../templates/skills/container-orchestration/references/compose-patterns.md");
const SKILL_LINUX_ADMINISTRATION: &str =
    include_str!("../../templates/skills/linux-administration/SKILL.md");
const SKILL_LINUX_ADMINISTRATION_REF_COMMANDS: &str =
    include_str!("../../templates/skills/linux-administration/references/commands-cheatsheet.md");
const SKILL_LINUX_ADMINISTRATION_REF_SYSTEMD: &str =
    include_str!("../../templates/skills/linux-administration/references/systemd-reference.md");
const SKILL_SHELL_SCRIPTING: &str =
    include_str!("../../templates/skills/shell-scripting/SKILL.md");
const SKILL_SHELL_SCRIPTING_REF_PATTERNS: &str =
    include_str!("../../templates/skills/shell-scripting/references/bash-patterns.md");

// Architecture skills (new)
const SKILL_EVENT_DRIVEN_ARCHITECTURE: &str =
    include_str!("../../templates/skills/event-driven-architecture/SKILL.md");
const SKILL_EVENT_DRIVEN_REF_MESSAGING: &str =
    include_str!("../../templates/skills/event-driven-architecture/references/messaging-patterns.md");
const SKILL_DOMAIN_DRIVEN_DESIGN: &str =
    include_str!("../../templates/skills/domain-driven-design/SKILL.md");
const SKILL_DDD_REF_BUILDING_BLOCKS: &str =
    include_str!("../../templates/skills/domain-driven-design/references/ddd-building-blocks.md");
const SKILL_SYSTEM_DESIGN: &str =
    include_str!("../../templates/skills/system-design/SKILL.md");
const SKILL_SYSTEM_DESIGN_REF_ESTIMATION: &str =
    include_str!("../../templates/skills/system-design/references/estimation-cheatsheet.md");

// Design & visual skills (new)
const SKILL_PIXIJS_GAMEDEV: &str =
    include_str!("../../templates/skills/pixijs-gamedev/SKILL.md");
const SKILL_PIXIJS_REF_API: &str =
    include_str!("../../templates/skills/pixijs-gamedev/references/api-cheatsheet.md");
const SKILL_MOBILE_APP_DESIGN: &str =
    include_str!("../../templates/skills/mobile-app-design/SKILL.md");
const SKILL_MOBILE_APP_DESIGN_REF_PLATFORM: &str =
    include_str!("../../templates/skills/mobile-app-design/references/platform-guidelines.md");

// Data & Analytics skills
const SKILL_DATA_SCIENCE: &str =
    include_str!("../../templates/skills/data-science/SKILL.md");
const SKILL_DATA_SCIENCE_REF_TIDY: &str =
    include_str!("../../templates/skills/data-science/references/tidy-data-principles.md");
const SKILL_DATA_SCIENCE_REF_STATS: &str =
    include_str!("../../templates/skills/data-science/references/statistical-methods.md");
const SKILL_DATA_SCIENCE_REF_VIZ: &str =
    include_str!("../../templates/skills/data-science/references/visualization-guidelines.md");
const SKILL_DATA_PIPELINE: &str =
    include_str!("../../templates/skills/data-pipeline/SKILL.md");
const SKILL_DATA_VISUALIZATION: &str =
    include_str!("../../templates/skills/data-visualization/SKILL.md");
const SKILL_DATA_VISUALIZATION_REF_CHARTS: &str =
    include_str!("../../templates/skills/data-visualization/references/chart-selection.md");
const SKILL_FEATURE_ENGINEERING: &str =
    include_str!("../../templates/skills/feature-engineering/SKILL.md");
const SKILL_DATA_QUALITY: &str =
    include_str!("../../templates/skills/data-quality/SKILL.md");

// AI & ML skills
const SKILL_AI_FUNDAMENTALS: &str =
    include_str!("../../templates/skills/ai-fundamentals/SKILL.md");
const SKILL_AI_FUNDAMENTALS_REF_ML: &str =
    include_str!("../../templates/skills/ai-fundamentals/references/ml-concepts.md");
const SKILL_AI_FUNDAMENTALS_REF_MATH: &str =
    include_str!("../../templates/skills/ai-fundamentals/references/math-foundations.md");
const SKILL_RAG_ENGINEERING: &str =
    include_str!("../../templates/skills/rag-engineering/SKILL.md");
const SKILL_RAG_REF_CHUNKING: &str =
    include_str!("../../templates/skills/rag-engineering/references/chunking-strategies.md");
const SKILL_RAG_REF_RETRIEVAL: &str =
    include_str!("../../templates/skills/rag-engineering/references/retrieval-patterns.md");
const SKILL_RAG_REF_EVALUATION: &str =
    include_str!("../../templates/skills/rag-engineering/references/evaluation.md");
const SKILL_PROMPT_ENGINEERING: &str =
    include_str!("../../templates/skills/prompt-engineering/SKILL.md");
const SKILL_PROMPT_ENGINEERING_REF_TECHNIQUES: &str =
    include_str!("../../templates/skills/prompt-engineering/references/techniques-catalog.md");
const SKILL_LLM_EVALUATION: &str =
    include_str!("../../templates/skills/llm-evaluation/SKILL.md");
const SKILL_EMBEDDING_VECTORDB: &str =
    include_str!("../../templates/skills/embedding-vectordb/SKILL.md");
const SKILL_ML_PIPELINE: &str =
    include_str!("../../templates/skills/ml-pipeline/SKILL.md");
const SKILL_ML_PIPELINE_REF_STAGES: &str =
    include_str!("../../templates/skills/ml-pipeline/references/pipeline-stages.md");

// API & Integration skills
const SKILL_API_DESIGN: &str = include_str!("../../templates/skills/api-design/SKILL.md");
const SKILL_API_DESIGN_REF_REST: &str =
    include_str!("../../templates/skills/api-design/references/rest-conventions.md");
const SKILL_API_DESIGN_REF_OPENAPI: &str =
    include_str!("../../templates/skills/api-design/references/openapi-patterns.md");
const SKILL_GRAPHQL_PATTERNS: &str =
    include_str!("../../templates/skills/graphql-patterns/SKILL.md");
const SKILL_GRPC_PROTOBUF: &str =
    include_str!("../../templates/skills/grpc-protobuf/SKILL.md");
const SKILL_GRPC_REF_PROTO: &str =
    include_str!("../../templates/skills/grpc-protobuf/references/proto-conventions.md");
const SKILL_WEBHOOK_INTEGRATION: &str =
    include_str!("../../templates/skills/webhook-integration/SKILL.md");

// Security skills (new)
const SKILL_AUTH_PATTERNS: &str =
    include_str!("../../templates/skills/auth-patterns/SKILL.md");
const SKILL_AUTH_REF_OAUTH: &str =
    include_str!("../../templates/skills/auth-patterns/references/oauth-flows.md");
const SKILL_AUTH_REF_JWT: &str =
    include_str!("../../templates/skills/auth-patterns/references/jwt-reference.md");
const SKILL_SECURE_CODING: &str =
    include_str!("../../templates/skills/secure-coding/SKILL.md");
const SKILL_SECURE_CODING_REF_OWASP: &str =
    include_str!("../../templates/skills/secure-coding/references/owasp-checklist.md");
const SKILL_THREAT_MODELING: &str =
    include_str!("../../templates/skills/threat-modeling/SKILL.md");

// Observability skills
const SKILL_LOGGING_STRATEGY: &str =
    include_str!("../../templates/skills/logging-strategy/SKILL.md");
const SKILL_LOGGING_REF_STRUCTURED: &str =
    include_str!("../../templates/skills/logging-strategy/references/structured-logging.md");
const SKILL_METRICS_MONITORING: &str =
    include_str!("../../templates/skills/metrics-monitoring/SKILL.md");
const SKILL_METRICS_REF_TYPES: &str =
    include_str!("../../templates/skills/metrics-monitoring/references/metric-types.md");
const SKILL_DISTRIBUTED_TRACING: &str =
    include_str!("../../templates/skills/distributed-tracing/SKILL.md");
const SKILL_ALERTING_ONCALL: &str =
    include_str!("../../templates/skills/alerting-oncall/SKILL.md");

// Database skills
const SKILL_SQL_PATTERNS: &str =
    include_str!("../../templates/skills/sql-patterns/SKILL.md");
const SKILL_SQL_PATTERNS_REF_QUERIES: &str =
    include_str!("../../templates/skills/sql-patterns/references/query-patterns.md");
const SKILL_SQL_PATTERNS_REF_SCHEMA: &str =
    include_str!("../../templates/skills/sql-patterns/references/schema-design.md");
const SKILL_DATABASE_MODELING: &str =
    include_str!("../../templates/skills/database-modeling/SKILL.md");
const SKILL_DATABASE_MODELING_REF_PATTERNS: &str =
    include_str!("../../templates/skills/database-modeling/references/modeling-patterns.md");
const SKILL_NOSQL_PATTERNS: &str =
    include_str!("../../templates/skills/nosql-patterns/SKILL.md");
const SKILL_DATABASE_MIGRATION: &str =
    include_str!("../../templates/skills/database-migration/SKILL.md");

// Performance skills
const SKILL_PERFORMANCE_PROFILING: &str =
    include_str!("../../templates/skills/performance-profiling/SKILL.md");
const SKILL_PERFORMANCE_REF_TOOLS: &str =
    include_str!("../../templates/skills/performance-profiling/references/profiling-tools.md");
const SKILL_CACHING_STRATEGIES: &str =
    include_str!("../../templates/skills/caching-strategies/SKILL.md");
const SKILL_CONCURRENCY_PATTERNS: &str =
    include_str!("../../templates/skills/concurrency-patterns/SKILL.md");
const SKILL_CONCURRENCY_REF_CATALOG: &str =
    include_str!("../../templates/skills/concurrency-patterns/references/patterns-catalog.md");
const SKILL_LOAD_TESTING: &str =
    include_str!("../../templates/skills/load-testing/SKILL.md");

// Framework-specific skills
const SKILL_REFLEX_PYTHON: &str =
    include_str!("../../templates/skills/reflex-python/SKILL.md");
const SKILL_REFLEX_REF_COMPONENTS: &str =
    include_str!("../../templates/skills/reflex-python/references/component-reference.md");
const SKILL_FASTAPI_PATTERNS: &str =
    include_str!("../../templates/skills/fastapi-patterns/SKILL.md");
const SKILL_FASTAPI_REF_ENDPOINTS: &str =
    include_str!("../../templates/skills/fastapi-patterns/references/endpoint-patterns.md");
const SKILL_PANDAS_POLARS: &str =
    include_str!("../../templates/skills/pandas-polars/SKILL.md");
const SKILL_PANDAS_POLARS_REF_COMPARISON: &str =
    include_str!("../../templates/skills/pandas-polars/references/api-comparison.md");
const SKILL_FLUTTER_DEVELOPMENT: &str =
    include_str!("../../templates/skills/flutter-development/SKILL.md");
const SKILL_FLUTTER_REF_WIDGETS: &str =
    include_str!("../../templates/skills/flutter-development/references/widget-catalog.md");

// SEO & Marketing skills
const SKILL_SEO_OPTIMIZATION: &str =
    include_str!("../../templates/skills/seo-optimization/SKILL.md");
const SKILL_SEO_REF_CHECKLIST: &str =
    include_str!("../../templates/skills/seo-optimization/references/technical-seo-checklist.md");

/// Default OWNER.md content — created locally in each project's context/ directory.
const OWNER_CONTENT: &str = r#"# Owner Profile

This file describes the project owner. It helps AI agents understand who they
are working with and tailor their communication and technical approach accordingly.

## About

- **Name:**
- **Role:**
- **Contact:**

## Background

- **Domain expertise:** <!-- e.g., backend systems, data science, DevOps -->
- **Primary languages:** <!-- e.g., Python, Rust, TypeScript -->
- **Years of experience:**

## Preferences

- **Communication style:** <!-- e.g., concise and direct, detailed explanations -->
- **Communication language:** <!-- e.g., English, German, prefer English for code comments -->
- **Code style preferences:** <!-- e.g., minimal comments, explicit types, functional style -->
- **Review preferences:** <!-- e.g., prefer small PRs, want tests for every change -->

## Working Context

- **Timezone:** <!-- e.g., Europe/Berlin -->
- **Working hours:** <!-- e.g., 09:00-18:00 CET -->
- **Current focus:** <!-- e.g., migrating auth system, learning Kubernetes -->
"#;

/// Scaffold the context/ directory using the process registry and selective skill deployment.
///
/// - Resolves process packages and effective skills from config
/// - Creates context/ directory and populates it with template files per package
/// - Creates CLAUDE.md at project root from the template
/// - Replaces {{project_name}} placeholders with the actual project name
/// - Deploys only the skills that belong to the resolved package set
/// - Creates .dev-box-version file
/// - Updates .gitignore with generated file entries and language-specific blocks
pub fn scaffold_context(config: &DevBoxConfig) -> Result<()> {
    let packages = process_registry::resolve_packages(&config.process.packages)
        .map_err(|e| anyhow::anyhow!(e))?;
    let effective_skills = process_registry::resolve_skills(
        &packages,
        &config.skills.include,
        &config.skills.exclude,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    let project_name = &config.container.name;
    let addons = &config.addons;

    output::info(&format!(
        "Scaffolding context for {:?} packages ({} skills)...",
        config.process.packages,
        effective_skills.len()
    ));

    // Always create CLAUDE.md (use product template as most complete)
    let claude_md = render(PRODUCT_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Create context/ directory
    let context = Path::new("context");
    fs::create_dir_all(context)?;

    // Scaffold context files from each package
    for pkg in &packages {
        scaffold_package_context(context, pkg, project_name)?;
    }

    // Scaffold only effective skills
    scaffold_skills_selective(&effective_skills)?;

    // Process declarations
    scaffold_processes(context)?;

    // OWNER.md (local copy) — done via core package's scaffold_package_context,
    // but setup_owner_md handles the shared/ location and backward compat
    setup_owner_md(context)?;

    // Create .dev-box-version
    write_if_missing(Path::new(".dev-box-version"), env!("CARGO_PKG_VERSION"))?;
    output::ok("Created .dev-box-version");

    // Update .gitignore with dev-box entries and language-specific blocks
    update_gitignore(addons)?;

    // Create Dockerfile.local placeholder
    let local_dockerfile = Path::new(crate::config::DEVCONTAINER_DIR).join("Dockerfile.local");
    write_if_missing(
        &local_dockerfile,
        "# Project-specific Dockerfile layers.\n\
         # This file is appended to the generated Dockerfile by `dev-box sync`.\n\
         # It is never overwritten — you own this file.\n\
         #\n\
         # The generated base image is available as the \"dev-box\" stage:\n\
         #   FROM ghcr.io/projectious-work/dev-box:<image>-v<version> AS dev-box\n\
         #\n\
         # Simple usage — add layers directly:\n\
         #   RUN apt-get update && apt-get install -y some-package\n\
         #   RUN npx playwright install --with-deps chromium\n\
         #\n\
         # Advanced usage — multi-stage build referencing the dev-box stage:\n\
         #   FROM node:20 AS builder\n\
         #   RUN npm ci && npm run build\n\
         #\n\
         #   FROM dev-box\n\
         #   COPY --from=builder /app/dist /workspace/dist\n",
    )?;

    output::ok(&format!(
        "Context scaffolded ({:?} packages, {} skills)",
        config.process.packages,
        effective_skills.len()
    ));
    Ok(())
}

/// Scaffold context files for a single process package.
///
/// Creates directories listed in `pkg.directories` and writes context files
/// using template content matched by `template_key`. Unknown template keys
/// are written as simple placeholder files.
fn scaffold_package_context(
    _context: &Path,
    pkg: &process_registry::ProcessPackage,
    project_name: &str,
) -> Result<()> {
    // Create directories
    for dir in pkg.directories {
        let dir_path = Path::new(dir);
        fs::create_dir_all(dir_path)
            .with_context(|| format!("Failed to create {}", dir_path.display()))?;
        // Add .gitkeep for empty directories
        let gitkeep = dir_path.join(".gitkeep");
        write_if_missing(&gitkeep, "")?;
    }

    // Scaffold context files
    for cf in pkg.context_files {
        let file_path = Path::new(cf.path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = template_content_for_key(cf.template_key, project_name);
        write_if_missing(file_path, &content)?;
        output::ok(&format!("Created {}", cf.path));
    }

    Ok(())
}

/// Map a template_key to actual template content.
///
/// Falls back to a simple placeholder for keys that don't have a template yet.
fn template_content_for_key(key: &str, project_name: &str) -> String {
    match key {
        // core
        "devbox_md" => format!(
            "# dev-box Configuration Notes\n\n\
             Project: {}\n\n\
             This file contains notes about the dev-box configuration for this project.\n",
            project_name
        ),
        "owner_md" => OWNER_CONTENT.to_string(),

        // tracking
        "backlog_md" => PRODUCT_BACKLOG.to_string(),
        "decisions_md" => PRODUCT_DECISIONS.to_string(),
        "eventlog_md" => {
            "# Event Log\n\n\
             Chronological record of significant project events.\n\n\
             | Date | Event | Details |\n\
             |------|-------|---------|\n"
                .to_string()
        }

        // standups
        "standups_md" => PRODUCT_STANDUPS.to_string(),

        // handover
        "session_template_md" => {
            "# Session Handover Template\n\n\
             ## Context\n\n\
             ## What was done\n\n\
             ## Open items\n\n\
             ## Next steps\n"
                .to_string()
        }

        // product
        "prd_md" => PRODUCT_PRD.to_string(),
        "projects_md" => PRODUCT_PROJECTS.to_string(),

        // code
        "development_md" => PRODUCT_DEVELOPMENT.to_string(),

        // research
        "progress_md" => RESEARCH_PROGRESS.to_string(),

        // operations
        "team_md" => PRODUCT_TEAM.to_string(),

        // Unknown key — placeholder
        other => format!(
            "# {}\n\nPlaceholder — template for '{}' not yet available.\n",
            other, other
        ),
    }
}

/// Scaffold only the skills that are in the effective skill set.
///
/// Keeps the full skill definitions list but filters deployment to only those
/// skills resolved from the process packages, include, and exclude lists.
fn scaffold_skills_selective(effective_skills: &[String]) -> Result<()> {
    let skills_dir = Path::new(".claude").join("skills");
    fs::create_dir_all(&skills_dir).context("Failed to create .claude/skills")?;

    let all_skills: &[SkillDef] = ALL_SKILL_DEFS;

    let mut deployed = 0;
    for (name, content, refs) in all_skills {
        if effective_skills.iter().any(|s| s == name) {
            let skill_dir = skills_dir.join(name);
            fs::create_dir_all(&skill_dir)
                .with_context(|| format!("Failed to create .claude/skills/{}", name))?;
            write_if_missing(&skill_dir.join("SKILL.md"), content)?;
            if !refs.is_empty() {
                let refs_dir = skill_dir.join("references");
                fs::create_dir_all(&refs_dir).with_context(|| {
                    format!("Failed to create .claude/skills/{}/references", name)
                })?;
                for (ref_name, ref_content) in *refs {
                    write_if_missing(&refs_dir.join(ref_name), ref_content)?;
                }
            }
            deployed += 1;
        }
    }

    output::ok(&format!(
        "Created .claude/skills/ ({} of {} available skills deployed)",
        deployed,
        all_skills.len()
    ));

    Ok(())
}

/// Scaffold minimal process: just CLAUDE.md at root, no context/ directory.
#[allow(dead_code)]
fn scaffold_minimal(project_name: &str) -> Result<()> {
    let claude_md = render(MINIMAL_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");
    Ok(())
}

/// Scaffold managed process.
#[allow(dead_code)]
fn scaffold_managed(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("work-instructions"))
        .context("Failed to create context/work-instructions")?;

    // CLAUDE.md at root
    let claude_md = render(MANAGED_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("DECISIONS.md"), MANAGED_DECISIONS)?;
    output::ok("Created context/DECISIONS.md");

    write_if_missing(&context.join("BACKLOG.md"), MANAGED_BACKLOG)?;
    output::ok("Created context/BACKLOG.md");

    write_if_missing(&context.join("STANDUPS.md"), MANAGED_STANDUPS)?;
    output::ok("Created context/STANDUPS.md");

    write_if_missing(
        &context.join("work-instructions").join("GENERAL.md"),
        MANAGED_GENERAL,
    )?;
    output::ok("Created context/work-instructions/GENERAL.md");

    // Process declarations and skills
    scaffold_processes(context)?;
    scaffold_skills()?;

    // OWNER.md (local copy)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold research process.
#[allow(dead_code)]
fn scaffold_research(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("research")).context("Failed to create context/research")?;
    fs::create_dir_all(context.join("analysis")).context("Failed to create context/analysis")?;

    // CLAUDE.md at root
    let claude_md = render(RESEARCH_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("PROGRESS.md"), RESEARCH_PROGRESS)?;
    output::ok("Created context/PROGRESS.md");

    // Research note template
    write_if_missing(
        &context.join("research").join("_template.md"),
        RESEARCH_NOTE_TEMPLATE,
    )?;
    output::ok("Created context/research/_template.md");

    // .gitkeep for empty dirs
    write_if_missing(&context.join("analysis").join(".gitkeep"), "")?;
    output::ok("Created context/analysis/");

    // Experiments directory
    let experiments = Path::new("experiments");
    fs::create_dir_all(experiments).context("Failed to create experiments/")?;
    write_if_missing(&experiments.join("README.md"), EXPERIMENTS_README)?;
    output::ok("Created experiments/README.md");

    // Process declarations and skills
    scaffold_processes(context)?;
    scaffold_skills()?;

    // OWNER.md (local copy)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold product process (full set).
#[allow(dead_code)]
fn scaffold_product(project_name: &str) -> Result<()> {
    let context = Path::new("context");
    fs::create_dir_all(context.join("work-instructions"))
        .context("Failed to create context/work-instructions")?;
    fs::create_dir_all(context.join("project-notes"))
        .context("Failed to create context/project-notes")?;
    fs::create_dir_all(context.join("ideas")).context("Failed to create context/ideas")?;

    // CLAUDE.md at root
    let claude_md = render(PRODUCT_CLAUDE_MD, project_name);
    write_if_missing(Path::new("CLAUDE.md"), &claude_md)?;
    output::ok("Created CLAUDE.md");

    // Context files
    write_if_missing(&context.join("DECISIONS.md"), PRODUCT_DECISIONS)?;
    output::ok("Created context/DECISIONS.md");

    write_if_missing(&context.join("BACKLOG.md"), PRODUCT_BACKLOG)?;
    output::ok("Created context/BACKLOG.md");

    write_if_missing(&context.join("STANDUPS.md"), PRODUCT_STANDUPS)?;
    output::ok("Created context/STANDUPS.md");

    write_if_missing(&context.join("PROJECTS.md"), PRODUCT_PROJECTS)?;
    output::ok("Created context/PROJECTS.md");

    write_if_missing(&context.join("PRD.md"), PRODUCT_PRD)?;
    output::ok("Created context/PRD.md");

    write_if_missing(
        &context.join("work-instructions").join("GENERAL.md"),
        PRODUCT_GENERAL,
    )?;
    output::ok("Created context/work-instructions/GENERAL.md");

    write_if_missing(
        &context.join("work-instructions").join("DEVELOPMENT.md"),
        PRODUCT_DEVELOPMENT,
    )?;
    output::ok("Created context/work-instructions/DEVELOPMENT.md");

    write_if_missing(
        &context.join("work-instructions").join("TEAM.md"),
        PRODUCT_TEAM,
    )?;
    output::ok("Created context/work-instructions/TEAM.md");

    // .gitkeep for empty dirs
    write_if_missing(&context.join("project-notes").join(".gitkeep"), "")?;
    write_if_missing(&context.join("ideas").join(".gitkeep"), "")?;
    output::ok("Created context/project-notes/ and context/ideas/");

    // Research subfolder with template
    fs::create_dir_all(context.join("research"))
        .context("Failed to create context/research")?;
    write_if_missing(
        &context.join("research").join("_template.md"),
        RESEARCH_NOTE_TEMPLATE,
    )?;
    output::ok("Created context/research/_template.md");

    // Experiments directory
    let experiments = Path::new("experiments");
    fs::create_dir_all(experiments).context("Failed to create experiments/")?;
    write_if_missing(&experiments.join("README.md"), EXPERIMENTS_README)?;
    output::ok("Created experiments/README.md");

    // Process declarations and skills
    scaffold_processes(context)?;
    scaffold_skills()?;

    // OWNER.md (local copy)
    setup_owner_md(context)?;

    Ok(())
}

/// Scaffold process declaration files into context/processes/.
fn scaffold_processes(context: &Path) -> Result<()> {
    let processes = context.join("processes");
    fs::create_dir_all(&processes).context("Failed to create context/processes")?;

    write_if_missing(&processes.join("README.md"), PROCESS_README)?;
    write_if_missing(&processes.join("release.md"), PROCESS_RELEASE)?;
    write_if_missing(&processes.join("code-review.md"), PROCESS_CODE_REVIEW)?;
    write_if_missing(
        &processes.join("feature-development.md"),
        PROCESS_FEATURE_DEV,
    )?;
    write_if_missing(&processes.join("bug-fix.md"), PROCESS_BUG_FIX)?;
    output::ok("Created context/processes/");

    Ok(())
}

/// A skill definition: (directory_name, skill_content, reference_files).
type SkillDef = (&'static str, &'static str, &'static [(&'static str, &'static str)]);

/// All available skills: (directory_name, skill_content, reference_files).
/// Reference files are deployed to .claude/skills/<name>/references/<filename>.
static ALL_SKILL_DEFS: &[SkillDef] = &[
        // Core process skills
        ("backlog-context", SKILL_BACKLOG_CONTEXT, &[]),
        ("decisions-adr", SKILL_DECISIONS_ADR, &[]),
        ("standup-context", SKILL_STANDUP_CONTEXT, &[]),
        // Development skills
        ("code-review", SKILL_CODE_REVIEW, &[]),
        ("testing-strategy", SKILL_TESTING_STRATEGY, &[]),
        ("refactoring", SKILL_REFACTORING, &[
            ("code-smells.md", SKILL_REFACTORING_REF_CODE_SMELLS),
            ("gof-patterns.md", SKILL_REFACTORING_REF_GOF_PATTERNS),
        ]),
        ("documentation", SKILL_DOCUMENTATION, &[]),
        ("debugging", SKILL_DEBUGGING, &[]),
        // Process skills
        ("release-semver", SKILL_RELEASE_SEMVER, &[]),
        ("incident-response", SKILL_INCIDENT_RESPONSE, &[]),
        ("retrospective", SKILL_RETROSPECTIVE, &[]),
        // Language-specific skills
        ("python-best-practices", SKILL_PYTHON_BEST_PRACTICES, &[]),
        ("rust-conventions", SKILL_RUST_CONVENTIONS, &[]),
        ("latex-authoring", SKILL_LATEX_AUTHORING, &[
            ("packages.md", SKILL_LATEX_AUTHORING_REF_PACKAGES),
            ("math-reference.md", SKILL_LATEX_AUTHORING_REF_MATH),
            ("tikz-reference.md", SKILL_LATEX_AUTHORING_REF_TIKZ),
        ]),
        ("typescript-patterns", SKILL_TYPESCRIPT_PATTERNS, &[]),
        // Infrastructure skills
        ("dockerfile-review", SKILL_DOCKERFILE_REVIEW, &[]),
        ("git-workflow", SKILL_GIT_WORKFLOW, &[]),
        ("ci-cd-setup", SKILL_CI_CD_SETUP, &[]),
        // Design & visual skills
        ("excalidraw", SKILL_EXCALIDRAW, &[
            ("json-schema.md", SKILL_EXCALIDRAW_REF_JSON_SCHEMA),
        ]),
        ("frontend-design", SKILL_FRONTEND_DESIGN, &[
            ("accessibility-checklist.md", SKILL_FRONTEND_DESIGN_REF_A11Y),
        ]),
        ("infographics", SKILL_INFOGRAPHICS, &[
            ("best-practices.md", SKILL_INFOGRAPHICS_REF_BEST_PRACTICES),
        ]),
        ("logo-design", SKILL_LOGO_DESIGN, &[
            ("design-principles.md", SKILL_LOGO_DESIGN_REF_DESIGN_PRINCIPLES),
        ]),
        ("tailwind", SKILL_TAILWIND, &[
            ("cheatsheet.md", SKILL_TAILWIND_REF_CHEATSHEET),
        ]),
        // Architecture skills
        ("software-architecture", SKILL_SOFTWARE_ARCHITECTURE, &[
            ("patterns.md", SKILL_SOFTWARE_ARCHITECTURE_REF_PATTERNS),
        ]),
        // Security skills
        ("dependency-audit", SKILL_DEPENDENCY_AUDIT, &[]),
        ("secret-management", SKILL_SECRET_MANAGEMENT, &[]),
        ("auth-patterns", SKILL_AUTH_PATTERNS, &[
            ("oauth-flows.md", SKILL_AUTH_REF_OAUTH),
            ("jwt-reference.md", SKILL_AUTH_REF_JWT),
        ]),
        ("secure-coding", SKILL_SECURE_CODING, &[
            ("owasp-checklist.md", SKILL_SECURE_CODING_REF_OWASP),
        ]),
        ("threat-modeling", SKILL_THREAT_MODELING, &[]),
        // Process skills (new)
        ("agent-management", SKILL_AGENT_MANAGEMENT, &[
            ("coordination-patterns.md", SKILL_AGENT_MANAGEMENT_REF_COORDINATION),
        ]),
        ("estimation-planning", SKILL_ESTIMATION_PLANNING, &[]),
        ("postmortem-writing", SKILL_POSTMORTEM_WRITING, &[]),
        // Development skills (new)
        ("tdd-workflow", SKILL_TDD_WORKFLOW, &[]),
        ("integration-testing", SKILL_INTEGRATION_TESTING, &[
            ("test-fixtures.md", SKILL_INTEGRATION_TESTING_REF_FIXTURES),
        ]),
        ("error-handling", SKILL_ERROR_HANDLING, &[]),
        ("dependency-management", SKILL_DEPENDENCY_MANAGEMENT, &[]),
        ("code-generation", SKILL_CODE_GENERATION, &[]),
        // Language skills (new)
        ("go-conventions", SKILL_GO_CONVENTIONS, &[
            ("go-patterns.md", SKILL_GO_CONVENTIONS_REF_PATTERNS),
        ]),
        ("java-patterns", SKILL_JAVA_PATTERNS, &[]),
        ("sql-style-guide", SKILL_SQL_STYLE_GUIDE, &[]),
        // Infrastructure skills (new)
        ("kubernetes-basics", SKILL_KUBERNETES_BASICS, &[
            ("cluster-architecture.md", SKILL_KUBERNETES_REF_ARCHITECTURE),
            ("resource-cheatsheet.md", SKILL_KUBERNETES_REF_RESOURCES),
            ("troubleshooting.md", SKILL_KUBERNETES_REF_TROUBLESHOOTING),
        ]),
        ("dns-networking", SKILL_DNS_NETWORKING, &[
            ("protocol-reference.md", SKILL_DNS_NETWORKING_REF_PROTOCOL),
            ("troubleshooting-tools.md", SKILL_DNS_NETWORKING_REF_TOOLS),
        ]),
        ("terraform-basics", SKILL_TERRAFORM_BASICS, &[]),
        ("container-orchestration", SKILL_CONTAINER_ORCHESTRATION, &[
            ("compose-patterns.md", SKILL_CONTAINER_ORCHESTRATION_REF_COMPOSE),
        ]),
        ("linux-administration", SKILL_LINUX_ADMINISTRATION, &[
            ("commands-cheatsheet.md", SKILL_LINUX_ADMINISTRATION_REF_COMMANDS),
            ("systemd-reference.md", SKILL_LINUX_ADMINISTRATION_REF_SYSTEMD),
        ]),
        ("shell-scripting", SKILL_SHELL_SCRIPTING, &[
            ("bash-patterns.md", SKILL_SHELL_SCRIPTING_REF_PATTERNS),
        ]),
        // Architecture skills (new)
        ("event-driven-architecture", SKILL_EVENT_DRIVEN_ARCHITECTURE, &[
            ("messaging-patterns.md", SKILL_EVENT_DRIVEN_REF_MESSAGING),
        ]),
        ("domain-driven-design", SKILL_DOMAIN_DRIVEN_DESIGN, &[
            ("ddd-building-blocks.md", SKILL_DDD_REF_BUILDING_BLOCKS),
        ]),
        ("system-design", SKILL_SYSTEM_DESIGN, &[
            ("estimation-cheatsheet.md", SKILL_SYSTEM_DESIGN_REF_ESTIMATION),
        ]),
        // Design & visual skills (new)
        ("pixijs-gamedev", SKILL_PIXIJS_GAMEDEV, &[
            ("api-cheatsheet.md", SKILL_PIXIJS_REF_API),
        ]),
        ("mobile-app-design", SKILL_MOBILE_APP_DESIGN, &[
            ("platform-guidelines.md", SKILL_MOBILE_APP_DESIGN_REF_PLATFORM),
        ]),
        // Data & Analytics skills
        ("data-science", SKILL_DATA_SCIENCE, &[
            ("tidy-data-principles.md", SKILL_DATA_SCIENCE_REF_TIDY),
            ("statistical-methods.md", SKILL_DATA_SCIENCE_REF_STATS),
            ("visualization-guidelines.md", SKILL_DATA_SCIENCE_REF_VIZ),
        ]),
        ("data-pipeline", SKILL_DATA_PIPELINE, &[]),
        ("data-visualization", SKILL_DATA_VISUALIZATION, &[
            ("chart-selection.md", SKILL_DATA_VISUALIZATION_REF_CHARTS),
        ]),
        ("feature-engineering", SKILL_FEATURE_ENGINEERING, &[]),
        ("data-quality", SKILL_DATA_QUALITY, &[]),
        // AI & ML skills
        ("ai-fundamentals", SKILL_AI_FUNDAMENTALS, &[
            ("ml-concepts.md", SKILL_AI_FUNDAMENTALS_REF_ML),
            ("math-foundations.md", SKILL_AI_FUNDAMENTALS_REF_MATH),
        ]),
        ("rag-engineering", SKILL_RAG_ENGINEERING, &[
            ("chunking-strategies.md", SKILL_RAG_REF_CHUNKING),
            ("retrieval-patterns.md", SKILL_RAG_REF_RETRIEVAL),
            ("evaluation.md", SKILL_RAG_REF_EVALUATION),
        ]),
        ("prompt-engineering", SKILL_PROMPT_ENGINEERING, &[
            ("techniques-catalog.md", SKILL_PROMPT_ENGINEERING_REF_TECHNIQUES),
        ]),
        ("llm-evaluation", SKILL_LLM_EVALUATION, &[]),
        ("embedding-vectordb", SKILL_EMBEDDING_VECTORDB, &[]),
        ("ml-pipeline", SKILL_ML_PIPELINE, &[
            ("pipeline-stages.md", SKILL_ML_PIPELINE_REF_STAGES),
        ]),
        // API & Integration skills
        ("api-design", SKILL_API_DESIGN, &[
            ("rest-conventions.md", SKILL_API_DESIGN_REF_REST),
            ("openapi-patterns.md", SKILL_API_DESIGN_REF_OPENAPI),
        ]),
        ("graphql-patterns", SKILL_GRAPHQL_PATTERNS, &[]),
        ("grpc-protobuf", SKILL_GRPC_PROTOBUF, &[
            ("proto-conventions.md", SKILL_GRPC_REF_PROTO),
        ]),
        ("webhook-integration", SKILL_WEBHOOK_INTEGRATION, &[]),
        // Observability skills
        ("logging-strategy", SKILL_LOGGING_STRATEGY, &[
            ("structured-logging.md", SKILL_LOGGING_REF_STRUCTURED),
        ]),
        ("metrics-monitoring", SKILL_METRICS_MONITORING, &[
            ("metric-types.md", SKILL_METRICS_REF_TYPES),
        ]),
        ("distributed-tracing", SKILL_DISTRIBUTED_TRACING, &[]),
        ("alerting-oncall", SKILL_ALERTING_ONCALL, &[]),
        // Database skills
        ("sql-patterns", SKILL_SQL_PATTERNS, &[
            ("query-patterns.md", SKILL_SQL_PATTERNS_REF_QUERIES),
            ("schema-design.md", SKILL_SQL_PATTERNS_REF_SCHEMA),
        ]),
        ("database-modeling", SKILL_DATABASE_MODELING, &[
            ("modeling-patterns.md", SKILL_DATABASE_MODELING_REF_PATTERNS),
        ]),
        ("nosql-patterns", SKILL_NOSQL_PATTERNS, &[]),
        ("database-migration", SKILL_DATABASE_MIGRATION, &[]),
        // Performance skills
        ("performance-profiling", SKILL_PERFORMANCE_PROFILING, &[
            ("profiling-tools.md", SKILL_PERFORMANCE_REF_TOOLS),
        ]),
        ("caching-strategies", SKILL_CACHING_STRATEGIES, &[]),
        ("concurrency-patterns", SKILL_CONCURRENCY_PATTERNS, &[
            ("patterns-catalog.md", SKILL_CONCURRENCY_REF_CATALOG),
        ]),
        ("load-testing", SKILL_LOAD_TESTING, &[]),
        // Framework-specific skills
        ("reflex-python", SKILL_REFLEX_PYTHON, &[
            ("component-reference.md", SKILL_REFLEX_REF_COMPONENTS),
        ]),
        ("fastapi-patterns", SKILL_FASTAPI_PATTERNS, &[
            ("endpoint-patterns.md", SKILL_FASTAPI_REF_ENDPOINTS),
        ]),
        ("pandas-polars", SKILL_PANDAS_POLARS, &[
            ("api-comparison.md", SKILL_PANDAS_POLARS_REF_COMPARISON),
        ]),
        ("flutter-development", SKILL_FLUTTER_DEVELOPMENT, &[
            ("widget-catalog.md", SKILL_FLUTTER_REF_WIDGETS),
        ]),
        // SEO & Marketing
        ("seo-optimization", SKILL_SEO_OPTIMIZATION, &[
            ("technical-seo-checklist.md", SKILL_SEO_REF_CHECKLIST),
        ]),
    ];

/// Scaffold all skills (legacy — deploys everything).
#[allow(dead_code)]
fn scaffold_skills() -> Result<()> {
    let skills_dir = Path::new(".claude").join("skills");
    fs::create_dir_all(&skills_dir).context("Failed to create .claude/skills")?;

    for (name, content, refs) in ALL_SKILL_DEFS {
        let skill_dir = skills_dir.join(name);
        fs::create_dir_all(&skill_dir)
            .with_context(|| format!("Failed to create .claude/skills/{}", name))?;
        write_if_missing(&skill_dir.join("SKILL.md"), content)?;
        if !refs.is_empty() {
            let refs_dir = skill_dir.join("references");
            fs::create_dir_all(&refs_dir)
                .with_context(|| format!("Failed to create .claude/skills/{}/references", name))?;
            for (ref_name, ref_content) in *refs {
                write_if_missing(&refs_dir.join(ref_name), ref_content)?;
            }
        }
    }

    output::ok("Created .claude/skills/");

    Ok(())
}

/// Create OWNER.md in context/shared/ directory.
/// Falls back to context/OWNER.md check for backward compatibility.
fn setup_owner_md(context: &Path) -> Result<()> {
    // Backward compat: if context/OWNER.md exists, don't create shared/ version
    let legacy_path = context.join("OWNER.md");
    if legacy_path.exists() {
        tracing::debug!("context/OWNER.md already exists (legacy location), skipping");
        return Ok(());
    }

    let shared_dir = context.join("shared");
    fs::create_dir_all(&shared_dir)
        .with_context(|| format!("Failed to create {}", shared_dir.display()))?;

    let owner_path = shared_dir.join("OWNER.md");
    if owner_path.exists() {
        tracing::debug!("context/shared/OWNER.md already exists, skipping");
        return Ok(());
    }

    fs::write(&owner_path, OWNER_CONTENT)
        .with_context(|| format!("Failed to write {}", owner_path.display()))?;
    output::ok("Created context/shared/OWNER.md");

    Ok(())
}

/// Returns the list of expected context files for a given set of process packages.
///
/// Uses the process registry to resolve packages and collect their context file paths.
pub fn expected_context_files(packages: &[String]) -> Vec<&'static str> {
    let mut files: Vec<&'static str> = vec!["CLAUDE.md"];

    if let Ok(pkgs) = process_registry::resolve_packages(packages) {
        for pkg in &pkgs {
            for cf in pkg.context_files {
                files.push(cf.path);
            }
        }
        // Process declarations are always scaffolded when there are packages
        if pkgs.len() > 1 || pkgs.iter().any(|p| p.name != "core") {
            files.push("context/processes/README.md");
            files.push("context/processes/release.md");
            files.push("context/processes/code-review.md");
            files.push("context/processes/feature-development.md");
            files.push("context/processes/bug-fix.md");
        }
    }

    files
}

/// Replace {{project_name}} in template content.
pub(crate) fn render(template: &str, project_name: &str) -> String {
    template.replace("{{project_name}}", project_name)
}

/// Write content to a file only if it doesn't already exist.
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

/// Write content to a file only if it differs from the current content.
/// Creates parent directories if needed. Returns true if the file was written.
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

/// Generate a .gitignore with dev-box entries, project-specific section,
/// and language-specific blocks based on the configured addons.
pub(crate) fn update_gitignore(addons: &AddonsSection) -> Result<()> {
    let gitignore_path = Path::new(".gitignore");

    // If .gitignore already exists, just ensure dev-box entries are present
    if gitignore_path.exists() {
        return ensure_devbox_entries(gitignore_path);
    }

    // Create a new .gitignore with full structure
    let mut content = String::new();

    // Project-specific section
    content.push_str(
        "# ── Project-specific ─────────────────────────────────────────────────────────\n",
    );
    content.push_str("# Add your project-specific ignore patterns here.\n\n\n");

    // dev-box generated
    content.push_str(
        "# ── dev-box generated ────────────────────────────────────────────────────────\n",
    );
    content.push_str("# Files generated by dev-box — do not remove these entries.\n");
    content.push_str(".devcontainer/Dockerfile\n");
    content.push_str(".devcontainer/docker-compose.yml\n");
    content.push_str(".devcontainer/devcontainer.json\n");
    content.push_str(".dev-box-home/\n");
    content.push_str(".root/\n");
    content.push_str(".dev-box-version\n");
    content.push_str(".dev-box/\n");
    content.push_str(".dev-box-backup/\n");
    content.push_str(".dev-box-env/\n\n");

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

    // Language-specific blocks based on image flavor
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
    output::ok("Created .gitignore with dev-box and language-specific entries");

    Ok(())
}

/// Ensure dev-box entries exist in an existing .gitignore.
fn ensure_devbox_entries(gitignore_path: &Path) -> Result<()> {
    let required_entries = [
        "# dev-box generated",
        crate::config::DOCKERFILE,
        crate::config::COMPOSE_FILE,
        crate::config::DEVCONTAINER_JSON,
        ".dev-box-home/",
        ".dev-box-version",
        ".dev-box-backup/",
        ".dev-box-env/",
    ];

    let existing = fs::read_to_string(gitignore_path).context("Failed to read .gitignore")?;
    let existing_lines: Vec<&str> = existing.lines().map(|l| l.trim()).collect();

    let mut additions = Vec::new();
    for entry in &required_entries {
        if !existing_lines.contains(entry) {
            // Also check for .root/ (backward compat)
            if *entry == ".dev-box-home/" && existing_lines.contains(&".root/") {
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
    output::ok("Updated .gitignore with dev-box entries");

    Ok(())
}

/// Check that .gitignore has required entries. Used by doctor.
pub fn check_gitignore_entries() -> Vec<String> {
    let gitignore_path = Path::new(".gitignore");
    let mut warnings = Vec::new();

    if !gitignore_path.exists() {
        warnings.push(".gitignore not found — run 'dev-box init' or create one".to_string());
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
        (".dev-box-version", "version lockfile"),
    ];

    for (entry, desc) in &required {
        if !lines.contains(entry) {
            warnings.push(format!(".gitignore missing '{}' ({})", entry, desc));
        }
    }

    // Check for .dev-box-home/ or .root/
    if !lines.contains(&".dev-box-home/") && !lines.contains(&".root/") {
        warnings
            .push(".gitignore missing '.dev-box-home/' (persisted config directory)".to_string());
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Helper to run a closure inside a temp directory, restoring the original
    /// cwd afterwards (best-effort).
    fn in_temp_dir<F: FnOnce()>(f: F) {
        let dir = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        f();
        // Restore — ignore errors (dir may be deleted)
        let _ = std::env::set_current_dir(&original);
    }

    #[test]
    fn render_replaces_project_name() {
        let result = render("Hello {{project_name}}!", "my-app");
        assert_eq!(result, "Hello my-app!");
    }

    #[test]
    fn render_replaces_multiple_occurrences() {
        let result = render("{{project_name}} is {{project_name}}", "foo");
        assert_eq!(result, "foo is foo");
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

    fn test_config_with_packages(packages: Vec<String>) -> DevBoxConfig {
        let mut config = crate::config::test_config();
        config.process.packages = packages;
        config
    }

    fn addons_with(names: &[&str]) -> AddonsSection {
        use crate::config::AddonToolsSection;
        use std::collections::HashMap;
        let mut addons = HashMap::new();
        for name in names {
            addons.insert(
                name.to_string(),
                AddonToolsSection {
                    tools: HashMap::new(),
                },
            );
        }
        AddonsSection { addons }
    }

    #[test]
    #[serial]
    fn scaffold_core_creates_claude_md_and_version() {
        in_temp_dir(|| {
            let config = test_config_with_packages(vec!["core".to_string()]);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists(), "CLAUDE.md should exist");
            assert!(
                Path::new(".dev-box-version").exists(),
                ".dev-box-version should exist"
            );
            // Core package creates DEVBOX.md and OWNER.md
            assert!(Path::new("context/DEVBOX.md").exists());
            // Core skills are deployed
            assert!(Path::new(".claude/skills/agent-management/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_managed_preset_creates_expected_files() {
        in_temp_dir(|| {
            // "managed" is a preset -> core + tracking + standups + handover
            let config = test_config_with_packages(vec!["managed".to_string()]);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new(".dev-box-version").exists());
            // core package
            assert!(Path::new("context/DEVBOX.md").exists());
            // tracking package
            assert!(Path::new("context/DECISIONS.md").exists());
            assert!(Path::new("context/BACKLOG.md").exists());
            assert!(Path::new("context/EVENTLOG.md").exists());
            // standups package
            assert!(Path::new("context/STANDUPS.md").exists());
            // handover package
            assert!(Path::new("context/project-notes/session-template.md").exists());
            // owner
            // Core package creates context/OWNER.md (legacy path detected by setup_owner_md)
            assert!(Path::new("context/OWNER.md").exists());
            // process declarations
            assert!(Path::new("context/processes/README.md").exists());
            assert!(Path::new("context/processes/release.md").exists());
            assert!(Path::new("context/processes/code-review.md").exists());
            assert!(Path::new("context/processes/feature-development.md").exists());
            assert!(Path::new("context/processes/bug-fix.md").exists());
            // Skills from packages (only those with templates get deployed)
            assert!(Path::new(".claude/skills/backlog-context/SKILL.md").exists());
            assert!(Path::new(".claude/skills/decisions-adr/SKILL.md").exists());
            assert!(Path::new(".claude/skills/standup-context/SKILL.md").exists());
            assert!(Path::new(".claude/skills/agent-management/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_research_package_creates_expected_files() {
        in_temp_dir(|| {
            // "research" is a package, core is always added
            let config = test_config_with_packages(vec!["research".to_string()]);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            // research package context files
            assert!(Path::new("context/PROGRESS.md").exists());
            // research package directories
            assert!(Path::new("context/research/.gitkeep").exists());
            assert!(Path::new("context/analysis/.gitkeep").exists());
            assert!(Path::new("experiments/.gitkeep").exists());
            // owner
            // Core package creates context/OWNER.md (legacy path detected by setup_owner_md)
            assert!(Path::new("context/OWNER.md").exists());
            // process declarations
            assert!(Path::new("context/processes/README.md").exists());
            // Research skills deployed
            assert!(Path::new(".claude/skills/data-science/SKILL.md").exists());
            assert!(Path::new(".claude/skills/data-visualization/SKILL.md").exists());
            assert!(Path::new(".claude/skills/feature-engineering/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_full_product_preset_creates_all_expected_files() {
        in_temp_dir(|| {
            // "full-product" preset -> many packages
            let config = test_config_with_packages(vec!["full-product".to_string()]);
            scaffold_context(&config).unwrap();
            assert!(Path::new("CLAUDE.md").exists());
            assert!(Path::new(".dev-box-version").exists());
            // tracking
            assert!(Path::new("context/DECISIONS.md").exists());
            assert!(Path::new("context/BACKLOG.md").exists());
            // standups
            assert!(Path::new("context/STANDUPS.md").exists());
            // product
            assert!(Path::new("context/PROJECTS.md").exists());
            assert!(Path::new("context/PRD.md").exists());
            // code
            assert!(Path::new("context/work-instructions/DEVELOPMENT.md").exists());
            // operations
            assert!(Path::new("context/work-instructions/TEAM.md").exists());
            // handover
            assert!(Path::new("context/project-notes/session-template.md").exists());
            // owner
            // Core package creates context/OWNER.md (legacy path detected by setup_owner_md)
            assert!(Path::new("context/OWNER.md").exists());
            // process declarations
            assert!(Path::new("context/processes/README.md").exists());
            // Skills from code package
            assert!(Path::new(".claude/skills/code-review/SKILL.md").exists());
            assert!(Path::new(".claude/skills/testing-strategy/SKILL.md").exists());
            // Skills from security package
            assert!(Path::new(".claude/skills/secure-coding/SKILL.md").exists());
            // Skills from operations package
            assert!(Path::new(".claude/skills/ci-cd-setup/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn scaffold_selective_skills_excludes_unselected() {
        in_temp_dir(|| {
            // core-only config should NOT deploy code-review skill
            let config = test_config_with_packages(vec!["core".to_string()]);
            scaffold_context(&config).unwrap();
            assert!(
                !Path::new(".claude/skills/code-review/SKILL.md").exists(),
                "code-review should not be deployed for core-only"
            );
            assert!(
                !Path::new(".claude/skills/data-science/SKILL.md").exists(),
                "data-science should not be deployed for core-only"
            );
        });
    }

    #[test]
    #[serial]
    fn scaffold_with_include_adds_extra_skill() {
        in_temp_dir(|| {
            let mut config = test_config_with_packages(vec!["core".to_string()]);
            config.skills.include = vec!["flutter-development".to_string()];
            scaffold_context(&config).unwrap();
            assert!(
                Path::new(".claude/skills/flutter-development/SKILL.md").exists(),
                "flutter-development should be deployed via include"
            );
        });
    }

    #[test]
    #[serial]
    fn scaffold_with_exclude_removes_skill() {
        in_temp_dir(|| {
            // managed preset includes tracking -> backlog-context skill
            let mut config = test_config_with_packages(vec!["managed".to_string()]);
            config.skills.exclude = vec!["backlog-context".to_string()];
            scaffold_context(&config).unwrap();
            assert!(
                !Path::new(".claude/skills/backlog-context/SKILL.md").exists(),
                "backlog-context should be excluded"
            );
            // Other tracking skills should still be present
            assert!(Path::new(".claude/skills/decisions-adr/SKILL.md").exists());
        });
    }

    #[test]
    #[serial]
    fn claude_md_contains_project_name() {
        in_temp_dir(|| {
            let config = test_config_with_packages(vec!["core".to_string()]);
            scaffold_context(&config).unwrap();
            let content = fs::read_to_string("CLAUDE.md").unwrap();
            assert!(
                content.contains("test-proj"),
                "CLAUDE.md should contain project name"
            );
        });
    }

    #[test]
    #[serial]
    fn gitignore_includes_python_block() {
        in_temp_dir(|| {
            update_gitignore(&addons_with(&["python"])).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("__pycache__/"));
            assert!(content.contains("*.py[cod]"));
            assert!(content.contains(".dev-box-home/"));
        });
    }

    #[test]
    #[serial]
    fn gitignore_includes_latex_block() {
        in_temp_dir(|| {
            update_gitignore(&addons_with(&["latex"])).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("*.aux"));
            assert!(content.contains("*.synctex.gz"));
        });
    }

    #[test]
    #[serial]
    fn gitignore_includes_rust_block() {
        in_temp_dir(|| {
            update_gitignore(&addons_with(&["rust"])).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("target/"));
        });
    }

    #[test]
    #[serial]
    fn gitignore_combined_addons() {
        in_temp_dir(|| {
            update_gitignore(&addons_with(&["python", "latex"])).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("__pycache__/"));
            assert!(content.contains("*.aux"));
        });
    }

    #[test]
    #[serial]
    fn update_gitignore_preserves_existing_content() {
        in_temp_dir(|| {
            fs::write(".gitignore", "node_modules/\n*.log\n").unwrap();
            update_gitignore(&AddonsSection::default()).unwrap();
            let content = fs::read_to_string(".gitignore").unwrap();
            assert!(content.contains("node_modules/"));
            assert!(content.contains("*.log"));
            assert!(content.contains(".dev-box-home/") || content.contains(".root/"));
        });
    }

    #[test]
    #[serial]
    fn owner_md_has_extended_fields() {
        in_temp_dir(|| {
            let config = test_config_with_packages(vec!["managed".to_string()]);
            scaffold_context(&config).unwrap();
            // Core package creates context/OWNER.md, which triggers
            // setup_owner_md's legacy path detection (skips shared/).
            let content = fs::read_to_string("context/OWNER.md").unwrap();
            assert!(content.contains("Domain expertise"));
            assert!(content.contains("Timezone"));
            assert!(content.contains("Communication language"));
        });
    }

    #[test]
    fn expected_context_files_core_only() {
        let files = expected_context_files(&["core".to_string()]);
        assert!(files.contains(&"CLAUDE.md"));
        assert!(files.contains(&"context/DEVBOX.md"));
        assert!(files.contains(&"context/OWNER.md"));
    }

    #[test]
    fn expected_context_files_managed_preset() {
        let files = expected_context_files(&["managed".to_string()]);
        assert!(files.contains(&"CLAUDE.md"));
        assert!(files.contains(&"context/BACKLOG.md"));
        assert!(files.contains(&"context/DECISIONS.md"));
        assert!(files.contains(&"context/STANDUPS.md"));
        assert!(files.contains(&"context/processes/README.md"));
    }

    #[test]
    fn expected_context_files_with_product_package() {
        let files = expected_context_files(&["product".to_string()]);
        assert!(files.contains(&"CLAUDE.md"));
        assert!(files.contains(&"context/PRD.md"));
        assert!(files.contains(&"context/PROJECTS.md"));
    }

    #[test]
    fn expected_context_files_with_code_package() {
        let files = expected_context_files(&["code".to_string()]);
        assert!(files.contains(&"context/work-instructions/DEVELOPMENT.md"));
    }

    #[test]
    fn expected_context_files_with_operations_package() {
        let files = expected_context_files(&["operations".to_string()]);
        assert!(files.contains(&"context/work-instructions/TEAM.md"));
    }
}
