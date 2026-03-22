# Competitive Landscape & Ecosystem Research — March 2026

Research into four parallel developments affecting kaits' positioning: the Claude Skills ecosystem maturing into an open standard, the vibe-coding platform convergence, the emergence of the first AI agent security standard (AIUC-1), and the strategic implications for kaits as an enterprise agent orchestrator. Conducted 2026-03-21 as a four-stream parallel analysis. Covers: Agent Skills open specification, ClawHub marketplace, Skill Creator eval framework, Replit Agent v4, Google Antigravity (full-stack vibe coding), Lovable's business ops pivot, OpenAI Super App convergence, and AIUC-1 compliance domains. Each stream includes specific actionable recommendations for kaits and the dev-box project.

---

## Sources

| # | URL | Topic |
|---|---|---|
| 1 | https://x.com/trq212/status/2027463795355095314?lang=de | Tweet on Claude Skills ecosystem |
| 2 | https://github.com/anthropics/skills | Anthropic official skills repository |
| 3 | https://claude.com/blog/equipping-agents-for-the-real-world-with-agent-skills | Agent Skills blog post |
| 4 | https://claude.com/blog/improving-skill-creator-test-measure-and-refine-agent-skills | Skill Creator improvements blog |
| 5 | https://clawhub.ai/ | ClawHub community skill registry |
| 6 | https://replit.com/ | Replit Agent platform |
| 7 | https://blog.google/innovation-and-ai/technology/developers-tools/full-stack-vibe-coding-google-ai-studio/ | Google AI Studio Antigravity |
| 8 | https://lovable.dev/ | Lovable AI app builder |
| 9 | https://lovable.dev/blog/go-beyond-building-full-stack-apps-with-lovable | Lovable business ops expansion |
| 10 | https://www.heise.de/news/OpenAI-Super-App-soll-ChatGPT-und-andere-KI-Funktionen-zusammenfuehren-11219286.html | OpenAI Super App |
| 11 | https://www.aiuc-1.com/ | AIUC-1 AI Agent Security Standard |

---

## Stream 1: Claude Skills Ecosystem {#skills-ecosystem}

### 1.1 Agent Skills Architecture {#skills-architecture}

Agent Skills are folders containing a `SKILL.md` file (YAML frontmatter + Markdown instructions) plus optional `scripts/`, `references/`, and `assets/` directories. They operate on a **progressive disclosure model** with three levels:

- **Level 1 — Metadata (~100 tokens):** `name` and `description` loaded into system prompt at startup for all installed skills, enabling trigger decisions.
- **Level 2 — Instructions (<5000 tokens):** Full `SKILL.md` body loaded when the agent decides a skill is relevant.
- **Level 3 — Resources (unbounded):** Scripts, references, and assets loaded on-demand. Scripts can execute without being loaded into context.

### 1.2 Open Standard {#open-standard}

The format is now an open standard at [agentskills.io/specification](https://agentskills.io/specification), maintained separately from Anthropic. Required frontmatter fields: `name` (max 64 chars, lowercase+hyphens) and `description` (max 1024 chars). Optional fields: `license`, `compatibility`, `metadata` (arbitrary key-value), `allowed-tools` (experimental pre-approved tool list).

### 1.3 Distribution Model {#distribution}

Skills are distributed via multiple channels:
- **Claude Code Plugin Marketplace:** GitHub repos as marketplaces. Install with `/plugin marketplace add <org>/<repo>`, then `/plugin install <plugin>@<marketplace>`.
- **Claude.ai:** Upload or use pre-built skills (paid plans).
- **Claude API:** Skills API for programmatic use.
- **Official repo:** `anthropics/skills` on GitHub — 17 reference skills (algorithmic-art, brand-guidelines, canvas-design, claude-api, doc-coauthoring, docx, frontend-design, internal-comms, mcp-builder, pdf, pptx, skill-creator, slack-gif-creator, theme-factory, web-artifacts-builder, webapp-testing, xlsx).

### 1.4 ClawHub — Community Registry {#clawhub}

ClawHub (clawhub.ai) is an independent, npm-like community skill registry — "the skill dock for sharp agents." Part of the OpenClaw project ecosystem.

- **Model:** Versioned, rollback-ready publishing. Install via `npx clawhub@latest install <skill-name>`.
- **Discovery:** Vector-powered search.
- **Current state:** Early stage — "Highlighted skills" and "Popular skills" sections empty (March 2026).
- **Relationship to Anthropic:** No official connection. Independent community effort.

**Implication:** The skills ecosystem is fragmenting into official (Anthropic GitHub marketplace) and community (ClawHub) distribution channels, similar to npm vs. alternative registries.

### 1.5 Skill Creator & Eval Framework {#skill-creator}

Anthropic's `skill-creator` skill (485 lines) guides the full skill development lifecycle:

1. Capture intent → interview for edge cases → write SKILL.md draft
2. Create test prompts → save to `evals/evals.json`
3. Run tests → evaluate qualitatively + quantitatively
4. Iterate → expand test set → optimize description

**Key capabilities:**
- **Eval framework:** Test prompts with expected outcomes, pass/fail reporting, regression detection across model updates.
- **Benchmark mode:** Pass rates, elapsed time, token consumption tracking. CI/CD integration.
- **Multi-agent parallel testing:** Independent agents run evals in isolation.
- **Comparator agents:** A/B testing between skill versions, blinded evaluation.
- **Description optimizer:** Analyzes descriptions against sample prompts, reduces false positive/negative triggering.

**Skill taxonomy:**
- **Capability uplift skills:** Encode techniques for superior output (test to detect when base model makes them obsolete).
- **Encoded preference skills:** Sequence workflows matching team/org processes (test to maintain fidelity).

### 1.6 Best Practices {#best-practices}

- **Description is the primary trigger mechanism.** Must include what the skill does AND when to use it. Make descriptions "a little pushy" — Claude tends to undertrigger.
- **Progressive disclosure:** Keep SKILL.md under 500 lines. Move reference material to separate files.
- **Deterministic code over token generation.** Bundle Python/Bash scripts for repetitive tasks.
- **Start with evaluation.** Identify capability gaps through representative task testing before building skills.
- **Domain-organized references.** Separate files per variant (e.g., AWS/GCP/Azure).
- **Security:** Install skills only from trusted sources. Malicious skills can introduce vulnerabilities or direct exfiltration.

---

## Stream 2: Vibe Coding & AI Platform Convergence {#vibe-coding}

### 2.1 Replit Agent v4 {#replit}

Full AI-agent app builder. Agent writes code, sets up infrastructure, tests, deploys — all from natural language. Up to 200-minute autonomous runs, parallel agents, built-in auth/database/hosting/monitoring. Mobile app (iOS/Android). Free tier; Core $20/month.

### 2.2 Google AI Studio + Antigravity {#google-antigravity}

"Full-stack vibe coding" (announced 2026-03-18). The Antigravity coding agent takes over entirely from a text prompt — plans, writes multi-file code, runs tests, self-corrects. Firebase integration: auto-detects data persistence/auth needs, provisions Firestore + Firebase Auth with one click. Supports React, Angular, Next.js, third-party libraries. Powered by Gemini 3.1 Pro. Free for prototyping.

### 2.3 Lovable — Business Operations Pivot {#lovable}

Pivoting from vibe-coding app builder to business operations platform (2026-03-19 blog post):
- **Data analysis:** CSV/PDF processing with Python code execution in sandbox
- **Document generation:** Reports, invoices, pitch decks
- **File-to-app:** Spreadsheets/PDFs/screenshots → functional apps with real databases and RBAC
- **Integrations:** Amplitude, Slack, Granola meeting transcripts
- **Pricing:** Free (5 credits/day), Pro ($25/mo), Business ($50/mo + SSO), Enterprise (audit logs, SCIM)

**Key insight:** Lovable's enterprise tier (SSO, SCIM, audit logs) is table-stakes IAM. No concept of agent autonomy boundaries, capability profiles, or governance depth. The "file-to-app" pattern (upload spreadsheet → working app) is notable as inspiration for a "file-to-agent" feature.

### 2.4 OpenAI Super App {#openai-superapp}

Planned desktop application merging ChatGPT, Codex, and Atlas browser into a single hub. Led by Fidji Simo. Phased strategy: expand Codex with non-coding agentic capabilities → integrate ChatGPT and Atlas. OpenAI internally describes "heightened urgency," explicitly citing Anthropic/Claude enterprise success as a "wake-up call."

### 2.5 Market Segmentation {#market-segments}

The landscape splits into three tiers:

```
┌─────────────────────────────────────────────────────┐
│              ENTERPRISE AGENT GOVERNANCE             │
│  AIUC-1 Standard ←→ kaits (~60% ✓) ←→ Claude Skills │
├─────────────────────────────────────────────────────┤
│              DEVELOPER AUGMENTATION                  │
│  Claude Code · Cursor · GitHub Copilot               │
├─────────────────────────────────────────────────────┤
│         VIBE CODING (commoditizing)                  │
│  Replit · Lovable · Bolt · Google Antigravity        │
├─────────────────────────────────────────────────────┤
│              SUPER-APP CONVERGENCE                   │
│  OpenAI (ChatGPT + Codex + Atlas)                    │
└─────────────────────────────────────────────────────┘
```

Kaits occupies the top layer — none of the lower tiers do agent orchestration with governance, RPG progression, and company simulation.

---

## Stream 3: AIUC-1 — AI Agent Security Standard {#aiuc-1}

### 3.1 What It Is {#aiuc1-overview}

The first industry standard for AI agent security, safety, and reliability — "SOC 2 for AI agents."

- **Backed by:** 100+ Fortune 500 CISOs; consortium includes Microsoft, Meta, Google Cloud, Anthropic, Databricks (30+ organizations)
- **Technical contributors:** MITRE, Cisco, Stanford, ElevenLabs, UiPath
- **First auditor:** Schellman (accredited third-party)
- **Update cadence:** Quarterly refreshes
- **Framework alignment:** ISO 42001, MITRE ATLAS, NIST AI RMF, EU AI Act, OWASP Top 10 / Agentic Top 10

### 3.2 Six Compliance Domains {#aiuc1-domains}

1. **Data & Privacy** — protection against data leaks, PII exposure, unauthorized model training
2. **Security** — defense against prompt injection, jailbreaks, adversarial attacks
3. **Safety** — mitigation of harmful outputs via testing and human review
4. **Reliability** — prevention of hallucinations and unauthorized tool calls
5. **Accountability** — governance structures, approval processes, vendor oversight
6. **Society** — guardrails against catastrophic misuse

### 3.3 kaits Coverage Assessment {#aiuc1-kaits}

| AIUC-1 Domain | kaits Module(s) | Current Coverage | Gap |
|---|---|---|---|
| Data & Privacy | `kaits/sandbox/policy.py` (path/env filtering) | Partial | No PII detection, no data classification |
| Security | `kaits/sandbox/` (SecurityPolicy, executor) | Partial | No prompt injection defense, no jailbreak detection |
| Safety | Autonomy boundaries (BACK-020), human review gates | Good | No harmful-output classifier |
| Reliability | `kaits/degradation.py`, `kaits/capabilities.py` | Good | No hallucination detection |
| Accountability | `kaits/sandbox/audit.py`, event log, budget tracking | Strong | Missing formal compliance export |
| Society | Autonomy levels, human oversight | Minimal | No catastrophic-misuse guardrails |

**Overall: ~60% coverage.** No competitor in the agent orchestration space (CrewAI, LangGraph, AutoGen) is advertising AIUC-1 compliance. OpenAI is notably absent from the consortium; Anthropic is a member.

---

## Stream 4: Strategic Synthesis {#synthesis}

### 4.1 kaits Positioning {#positioning}

Kaits is **meta** to vibe-coding platforms — it could orchestrate agents that *use* them. The differentiation is:
- **Vibe coding:** builds one app → kaits runs your AI department
- **Developer tools:** augment one developer → kaits manages agent teams
- **Super apps:** consolidate AI tools → kaits governs AI agents

### 4.2 Opportunities {#opportunities}

1. **SKILL.md adoption** — align company-plugins with the open standard for cross-tool portability
2. **Skill creation as gameplay** — agents "learn" skills through use, evals provide XP metrics
3. **Agent team templates** — pre-configured teams ("startup squad", "DevOps team") for quick onboarding
4. **AIUC-1 first-mover** — close the 40% gap for unique enterprise positioning
5. **File-to-agent** — upload process document → configured agent with triggers and capabilities
6. **Eval framework** — adapt skill-creator's eval pattern for kaits agent performance tracking
7. **Description optimization** — audit skill trigger descriptions against undertrigger finding

### 4.3 Threats {#threats}

1. **Ecosystem convergence** — if Agent Skills standard mandates features kaits' plugin format lacks
2. **ClawHub network effects** — community marketplace could eclipse self-hosted distribution
3. **Commoditization of orchestration** — if Replit/OpenAI add multi-agent governance
4. **"Good enough" wins** — vibe coding produces mediocre but instant results; kaits' governance may seem heavyweight
5. **Google's scale** — free prototyping with Antigravity pulls casual users

### 4.4 kaits + dev-box Relationship {#kaits-devbox}

The SKILL.md open standard, agent team templates, and marketplace features create overlap between kaits (PROJ-001) and dev-box (PROJ-004). Analysis:

- **dev-box** provides the development container environment, context schema (`product` process), and tooling setup. It is the **infrastructure layer** — what environment you work in.
- **kaits** provides the agent orchestration, governance, and company simulation. It is the **application layer** — how agents work together.
- **Overlap zone:** Both projects need skill/plugin management, process definitions, and team configuration. The SKILL.md standard affects both: dev-box should support skill installation/management in containers, while kaits should consume skills as agent capabilities.
- **Resolution:** dev-box owns the container-level skill infrastructure (installation, discovery, security scanning). kaits owns the agent-level skill semantics (XP, progression, capability mapping). The SKILL.md spec is the shared contract.

#### 4.4.1 dev-box v0.4.0 Features Confirming the Model {#devbox-v040}

dev-box (as of v0.4.0, 2026-03-21) already implements mechanisms that address content ownership concerns:

**Migration system (v0.3.0+):**
- `dev-box doctor` detects schema version mismatches between `dev-box.toml` and `.dev-box-version`
- Generates 3 artifacts in `.dev-box/migration/`: `schema-diff.md` (structural changes), `migration-prompt.md` (AI-ready prompt for Claude Code), `checklist.md` (human-readable)
- Critical design principle: *"Migration artifacts describe structural changes. They do not migrate content."*
- kaits integration: Coach/PM agent reads migration-prompt.md, compares with current processes, proposes changes, human approves

**Backup (v0.3.9):**
- `dev-box backup` creates timestamped snapshots of all managed files (dev-box.toml, .devcontainer/, .dev-box-home/, context/, CLAUDE.md)
- `dev-box reset` with safety flags (--no-backup, --dry-run)
- Nothing is ever lost during upgrades or changes

**Environment management (v0.4.0):**
- `dev-box env create/switch/list/delete/status` — named environments
- Snapshots `dev-box.toml`, `CLAUDE.md`, and `context/` (excluding shared) to `.dev-box-env/<name>/`
- Switch stops container, saves current state, restores target, regenerates .devcontainer/
- kaits integration: environment switching maps to kaits multi-project model

**Shared folder (v0.4.0):**
- `context/shared/` persists across all environments, never moved during env switch
- kaits integration: cross-project config, shared process definitions, company-level context

**File ownership model:**
- dev-box owns infrastructure: `.devcontainer/` files — regenerated on `generate`, "do not hand-edit"
- User owns content: `context/` files — never overwritten, only structural migration via migration-prompt.md
- Flavor changes are additive — files never auto-deleted when switching process flavors

**Plugin system (roadmap):**
- Planned extensibility for custom commands and image overlays
- Potential integration point for kaits-specific container provisioning

#### 4.4.2 Standalone vs. Combined Use {#standalone-combined}

| Scope | dev-box alone | dev-box + kaits |
|---|---|---|
| Process scaffolding | `--process` flag (minimal/managed/research/product) | Same foundation |
| Process execution | Human follows conventions manually with Claude Code | AI agents run processes (PM=backlog, Coach=standups) |
| Process customization | User edits `context/processes/*.md` directly | User designs processes in kaits company simulator |
| Schema upgrades | migration-prompt.md → human/Claude Code applies | migration-prompt.md → kaits agent evaluates + proposes |
| Environment switching | `dev-box env switch` copies files | Maps to kaits multi-project model |
| Skills | SKILL.md installed in container, used by Claude Code | + XP tracking, proficiency, capability mapping |
| Team | Solo developer + Claude Code | AI agent teams with RPG progression |
| Governance | Implicit (developer discipline) | Explicit (sandbox, audit, autonomy boundaries, AIUC-1) |

### 4.5 Credit-Based Pricing {#pricing}

All major platforms converge on consumption-based pricing: Lovable ($25/mo for ~150 credits), Replit (compute-minute pricing), Google (token-based via Gemini API). When kaits commercializes, a credit/token model tied to agent compute-minutes and LLM calls is the market expectation.

---

## Recommendations {#recommendations}

### Priority 1 — High

| # | Recommendation | Impact | Effort |
|---|---|---|---|
| R1 | Adopt SKILL.md open standard in company-plugins | Cross-tool portability, ecosystem alignment | Low |
| R2 | File dev-box issues for SKILL.md support | Container-level skill management | Low |
| R3 | Skill creation as RPG gameplay mechanic | Unique retention loop, extends skill tree | Medium |
| R4 | Agent team templates | Fast onboarding, addresses "cold start" problem | Medium |
| R5 | Analyze kaits↔dev-box responsibility boundary | Prevent overlap, clarify ownership | Low |

### Priority 2 — Medium

| # | Recommendation | Impact | Effort |
|---|---|---|---|
| R6 | AIUC-1 alignment assessment | Enterprise differentiator, first-mover | Medium |
| R7 | AIUC-1 compliance audit feature | Self-assessment export from existing modules | Medium |
| R8 | Prompt injection defense in provider pipeline | OWASP Agentic Top 10 #1 | Medium |
| R9 | PII scanning in sandbox audit pipeline | Data & Privacy domain coverage | Medium |

### Priority 3 — Low / Future

| # | Recommendation | Impact | Effort |
|---|---|---|---|
| R10 | External skill marketplace with safety controls | Community growth, supply chain risk | High |
| R11 | File-to-agent feature | Unique differentiator, complex implementation | High |
| R12 | Audit existing skill descriptions for trigger optimization | Quick win, prevents undertriggering | Low |
| R13 | Credit-based pricing model design | Commercial readiness | Medium |
