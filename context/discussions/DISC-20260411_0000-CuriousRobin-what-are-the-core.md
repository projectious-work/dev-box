---
apiVersion: processkit.projectious.work/v1
kind: Discussion
metadata:
  id: DISC-20260411_0000-CuriousRobin-what-are-the-core
  created: '2026-04-10T22:42:36+00:00'
spec:
  question: What are the core principles and scope boundaries of aibox — what does
    it own, what belongs elsewhere, and what should it never become?
  state: active
  opened_at: '2026-04-10T22:42:36+00:00'
  participants:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
---

# aibox Refocus — Core Principles and Scope

*Originally DISC-002, dated 2026-04-05. Transferred from the pre-v0.8.0 context on processkit reset.*

## Problem Statement

An earlier exploration (DISC-001) of the context system redesign produced 74 decisions with 14 internal contradictions — a sign that the scope had expanded beyond what aibox should be. This discussion restarts from first principles.

## What aibox IS

**aibox is a CLI tool that provides consistent, containerized development environments for working with AI coding agents.**

Analogy: uv is for Python environments. aibox is for AI work environments.

What you get when you run `aibox init`:
- A dev container configured for your project's language/stack
- Skills and process templates scaffolded into your project
- AI provider configuration (CLAUDE.md, mcp.json) ready to use
- A `context/` directory structure for project management artifacts

## Core Principles

**P1: Dev container first.** aibox's primary artifact is a dev container. Slim, configurable, production-ready for AI-assisted development.

**P2: No inner system fallacy.** aibox does NOT re-expose Docker/docker-compose options behind its own configuration layer. aibox.toml contains aibox-specific configuration, not Docker configuration with extra steps.

**P3: Skills are complex, multi-artifact packages.** A skill is a package containing: instructions (markdown, three-level principle), examples, templates, and optionally a Python MCP server. Skills are developed and refined in the process repo, consumed by product repos.

**P4: Three-level principle.** All instruction markdown follows three levels: Level 1 (intro, 1-3 sentences), Level 2 (overview, key concepts and workflows), Level 3 (details, full reference). Directory INDEX.md files provide Level 0.

**P5: 18 primitives as universal building blocks.** WorkItem, LogEntry, DecisionRecord, Artifact, Role, Process/Workflow, StateMachine, Category/Taxonomy, CrossReference, Gate, Metric, Schedule, Scope, Constraint, Context/Environment, Discussion, Actor, Binding. Framework-agnostic; appear in every process methodology.

**P6: Git-based and provider-independent.** Everything versioned in git. Markdown files are the source of truth. No mandatory external infrastructure. Works with any AI provider.

**P7: Simple — one repo per concern.** aibox repo: CLI + container images + devcontainer scaffolding. Process repo: primitives + skills + process templates + MCP server source. Both repos dogfood aibox dev containers.

**P8: Skill MCP servers are Python source code.** Shipped as PEP 723 scripts. Consuming dev container runs the code. Python chosen for readability, modifiability, universal availability.

**P9: Enterprise governance is out of scope.** RBAC enforcement, multi-repo trust architectures, certificate-based authorization, verification manifests belong to a separate platform. aibox provides the development environment and process structure.

**P10: Kubernetes-inspired object model.** All entity files use structured YAML frontmatter: `apiVersion`, `kind`, `metadata` (id, timestamps, labels), `spec` (entity-specific fields).

**P11: Slim base image + composable addons.** Single base image (debian:trixie-slim) + essential dev tooling. Everything else is an addon (YAML definition composing onto base). Addons declare opinionated versions — curated set, sensible defaults.

**P12: Binding as generalized primitive.** A Binding connects any two entities with optional scope, temporality, and conditions. Rule: if a relationship has scope, time, or its own attributes → Binding entity. If it's just "A relates to B" → cross-reference in frontmatter.

## What aibox is NOT

- Not a workflow engine (agents execute processes, not aibox)
- Not an enterprise governance platform (no RBAC, no certificates, no auth policies)
- Not a project management tool (provides primitives; agent+user manage the project)
- Not a CI/CD system (provides dev environment; build/deploy is the project's concern)
- Not a Docker wrapper (scaffolds containers; does not abstract Docker)

## Status

These principles were validated through the v0.16.0 split (DEC-027) and the v0.16.5 MCP registration work (DEC-033). The processkit v0.8.0 reset is consistent with all of them. Discussion remains open for ongoing scope questions.
