---
apiVersion: processkit.projectious.work/v1
kind: Artifact
metadata:
  id: ART-20260411_0000-SleekPanda-aibox-product-requirements-document
  created: 2026-04-11T00:00:00Z
  labels:
    type: prd
    status: current
spec:
  title: "aibox Product Requirements Document"
  skill: prd-writing
  related_workitems: []
---

# aibox — Product Requirements Document

**Status:** Current
**Last updated:** 2026-04-11 (transferred from pre-v0.8.0 context on processkit reset)

## Vision

aibox is a CLI tool for reproducible, containerized development environments
with built-in AI context structure. It unifies container management, opinionated
tooling, color theming, and structured work processes into a single binary —
giving solo developers and small teams a "batteries included" dev environment
that works the same everywhere.

Tagline: **"uv for AI work environments."**

## Target Users

- **Solo developers** using AI-assisted workflows who want reproducible, themed,
  opinionated environments without manual Docker/devcontainer setup.
- **Small teams** needing consistent environments across members with structured
  context (backlog, decisions, standups) for AI agent collaboration.
- **Consultants and contractors** who spin up project environments frequently and
  need fast, repeatable scaffolding.

## Core Requirements

- Single static binary CLI (Rust), no runtime dependencies on the host.
- Container-based isolation via Docker/Podman devcontainers.
- `aibox.toml` configuration: container, context, AI, addons, appearance, audio.
- processkit installer: fetch a pinned processkit release, install skills/primitives/
  processes and canonical AGENTS.md template into `context/`.
- Color theming across Zellij, Vim, Yazi, lazygit (6 themes: nord, dracula,
  catppuccin-mocha, catppuccin-latte, tokyo-night, gruvbox).
- Three IDE layouts: dev, focus, cowork.
- AI provider flexibility: Claude, Aider, Gemini, Codex, Copilot, Continue — optional, stackable.
- Addon bundles: infrastructure, kubernetes, cloud providers, documentation tools,
  language runtimes (python, rust, node, go, latex, typst).
- Named environment management (`aibox env`).
- Config reconciliation (`aibox sync`) with three-way diff and migration documents.
- `aibox kit` subcommand: CLI-level visibility into installed processkit skills/processes.

## Non-Goals

- Full CI/CD platform — aibox is for local development, not pipelines.
- Kubernetes orchestrator — single-container focus; multi-service is a future goal.
- IDE replacement — aibox provides the terminal environment, not the editor.
- Enterprise multi-tenant — designed for individual/small team use.
- Cloud hosting — environments run locally (remote dev is a future goal, BACK-20260411_0000-ProudLily-remote-development-research-review).
- Enterprise governance (RBAC, certificates, authorization policies) — out of scope.

## Success Metrics

- Adoption by all projectious.work projects as the standard dev environment.
- Community forks and contributions as signal of external value.
- Time-to-productive for new projects: under 5 minutes from `aibox init` to working
  themed Zellij session with processkit content in place.
- Zero "works on my machine" issues across team members.

## Open Questions

- v1.0 feature completeness definition (BACK-20260411_0000-SoundRabbit-adapt-aibox-self-hosted, PROJ-001 scope).
- Preview companion MVP scope (BACK-20260411_0000-CoolBear-preview-companion-design-review, PROJ-004).
- Remote development first deliverable (BACK-20260411_0000-ProudLily-remote-development-research-review).
