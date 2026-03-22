# Skills Library

dev-box ships **83 curated skills** across 14 categories. Every skill follows the open [SKILL.md standard](https://agentskills.io/specification) and is automatically scaffolded into `.claude/skills/` when you run `dev-box init`.

## What Are Skills?

A skill is a directory containing a `SKILL.md` file (and optional `references/` files) that teaches an AI agent how to perform a specific task. Skills use **progressive disclosure**:

1. **Metadata** (~100 tokens) -- `name` and `description` loaded at startup for all skills
2. **Instructions** (<5000 tokens) -- full `SKILL.md` body loaded when the skill activates
3. **References** (on demand) -- detailed reference files loaded only when needed

```
.claude/skills/
├── kubernetes-basics/
│   ├── SKILL.md              # Main instructions
│   └── references/
│       ├── cluster-architecture.md
│       ├── resource-cheatsheet.md
│       └── troubleshooting.md
├── code-review/
│   └── SKILL.md              # Simple skill, no references
└── ...
```

## Categories at a Glance

| Category | Skills | Description |
|----------|--------|-------------|
| [Process](process.md) | 9 | Backlog, decisions, standups, releases, incidents, retrospectives, agent coordination |
| [Development](development.md) | 11 | Code review, testing, debugging, refactoring, error handling, documentation |
| [Language](language.md) | 7 | Python, Rust, TypeScript, Go, Java, SQL style, LaTeX |
| [Infrastructure](infrastructure.md) | 10 | Docker, Kubernetes, DNS/networking, Terraform, Linux, shell scripting, CI/CD |
| [Architecture](architecture.md) | 4 | Software architecture, DDD, event-driven, system design |
| [Design & Visual](design.md) | 7 | Frontend, Tailwind, Excalidraw, infographics, logos, PixiJS, mobile UX |
| [Data & Analytics](data.md) | 5 | Data science, pipelines, visualization, feature engineering, data quality |
| [AI & ML](ai-ml.md) | 6 | AI fundamentals, RAG, prompt engineering, LLM evaluation, embeddings, ML pipelines |
| [API & Integration](api.md) | 4 | REST API design, GraphQL, gRPC/Protobuf, webhooks |
| [Security](security.md) | 5 | Auth patterns, secure coding, threat modeling, dependency audit, secrets |
| [Observability](observability.md) | 4 | Logging, metrics, distributed tracing, alerting |
| [Database](database.md) | 4 | SQL patterns, data modeling, NoSQL, migrations |
| [Performance](performance.md) | 4 | Profiling, caching, concurrency, load testing |
| [Framework & SEO](framework.md) | 5 | FastAPI, Reflex, pandas/polars, Flutter, SEO |

## Installing and Managing Skills

### Automatic Scaffolding

All 83 skills are scaffolded automatically when you create a new project:

```bash
dev-box init --image python --process managed
# Skills appear in .claude/skills/
```

### Enabling/Disabling Skills

Skills are file-based -- enable or disable them by adding or removing directories:

```bash
# Disable a skill
rm -rf .claude/skills/kubernetes-basics/

# Re-enable by running sync (restores missing skills)
dev-box sync
```

### Updating Skills

When you upgrade dev-box, new skills are added automatically on the next `dev-box sync`. Existing skills are never overwritten -- only missing ones are created.

### Custom Skills

Create your own skills alongside the bundled ones:

```markdown
# .claude/skills/my-custom-skill/SKILL.md
---
name: my-custom-skill
description: What this does and when to use it. Be specific to help trigger detection.
allowed-tools: Bash(npm:*) Read Write
---

# My Custom Skill

## When to Use
Describe trigger conditions.

## Instructions
Step-by-step agent instructions.

## Examples
Scenario-based examples.
```

## SKILL.md Format

Every skill follows the [Agent Skills specification](https://agentskills.io/specification):

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Lowercase kebab-case, max 64 chars |
| `description` | Yes | What it does AND when to use it, max 1024 chars |
| `allowed-tools` | No | Pre-approved tools (e.g., `Bash(kubectl:*) Read Write`) |
| `license` | No | License identifier |
| `compatibility` | No | Environment requirements |
| `metadata` | No | Arbitrary key-value pairs |

### Progressive Disclosure Best Practices

- Keep `SKILL.md` under 500 lines (ideally 50-150)
- Move detailed reference material to `references/*.md`
- Agents load references on demand, keeping context lean
- Include 2-4 scenario-based examples in every skill

## Security

!!! warning "Only install skills from trusted sources"
    Skills contain instructions that AI agents execute. A malicious skill could instruct an agent to modify files or exfiltrate data. Only install skills from sources you trust.

- **Review before installing** -- read the SKILL.md file before adding third-party skills
- **Version control** -- commit `.claude/skills/` to git so changes are tracked
- **Prefer bundled** -- the 83 skills shipped with dev-box are curated and maintained
- **Audit external skills** -- treat them like any dependency
