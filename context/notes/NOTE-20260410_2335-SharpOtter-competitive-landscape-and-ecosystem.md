---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-SharpOtter-competitive-landscape-and-ecosystem
  created: 2026-04-11
spec:
  title: "Competitive Landscape & Ecosystem Research — March 2026"
  type: reference
  state: permanent
  tags: [competitive, landscape, skills-ecosystem, aiuc-1, vibe-coding, kaits]
  skill: research-with-confidence
  source_file: competitive-landscape-2026-03.md
---

# Competitive Landscape & Ecosystem Research — March 2026

Research into four parallel developments affecting kaits' positioning: the Claude Skills ecosystem maturing into an open standard, the vibe-coding platform convergence, the emergence of the first AI agent security standard (AIUC-1), and the strategic implications for kaits as an enterprise agent orchestrator. Conducted 2026-03-21 as a four-stream parallel analysis.

---

## Stream 1: Claude Skills Ecosystem

### Agent Skills Architecture

Agent Skills are folders containing a `SKILL.md` file (YAML frontmatter + Markdown instructions) plus optional `scripts/`, `references/`, and `assets/` directories. They operate on a **progressive disclosure model** with three levels:

- **Level 1 — Metadata (~100 tokens):** `name` and `description` loaded into system prompt at startup for all installed skills, enabling trigger decisions.
- **Level 2 — Instructions (<5000 tokens):** Full `SKILL.md` body loaded when the agent decides a skill is relevant.
- **Level 3 — Resources (unbounded):** Scripts, references, and assets loaded on-demand.

### Open Standard

The format is now an open standard at agentskills.io/specification. Required frontmatter: `name` (max 64 chars) and `description` (max 1024 chars). Optional: `license`, `compatibility`, `metadata`, `allowed-tools`.

### Distribution Channels

- **Claude Code Plugin Marketplace:** GitHub repos as marketplaces.
- **Claude.ai:** Upload or use pre-built skills (paid plans).
- **Official repo:** `anthropics/skills` — 17 reference skills.
- **ClawHub:** Independent community registry (clawhub.ai). npm-like, versioned. Early stage as of March 2026.

### Skill Creator & Eval Framework

Anthropic's `skill-creator` skill guides the full lifecycle: capture intent → test prompts → evals → iterate → optimize description. Supports multi-agent parallel testing, A/B comparisons, benchmark mode (pass rates, token consumption).

**Key best practices:**
- Description is the primary trigger mechanism — make it "a little pushy."
- Progressive disclosure: keep SKILL.md under 500 lines.
- Deterministic code over token generation for repetitive tasks.
- Start with evaluation before building.

---

## Stream 2: Vibe Coding & AI Platform Convergence

| Platform | Key Feature |
|---|---|
| Replit Agent v4 | 200-min autonomous runs, parallel agents, built-in hosting |
| Google Antigravity | Full-stack vibe coding, Firebase auto-provisioning, free prototyping |
| Lovable | Business ops pivot: CSV/PDF processing, file-to-app, SSO+SCIM enterprise tier |
| OpenAI Super App | Merges ChatGPT + Codex + Atlas; explicitly motivated by Anthropic/Claude enterprise success |

**Market segmentation:**

```
Enterprise Agent Governance  — AIUC-1, kaits (~60% ✓), Claude Skills
Developer Augmentation       — Claude Code, Cursor, GitHub Copilot
Vibe Coding (commoditizing)  — Replit, Lovable, Bolt, Google Antigravity
Super-App Convergence        — OpenAI (ChatGPT + Codex + Atlas)
```

---

## Stream 3: AIUC-1 — AI Agent Security Standard

"SOC 2 for AI agents." Backed by 100+ Fortune 500 CISOs; Microsoft, Meta, Google Cloud, Anthropic, Databricks (30+ organizations).

**Six compliance domains:**
1. Data & Privacy — protection against data leaks, PII exposure
2. Security — defense against prompt injection, jailbreaks
3. Safety — mitigation of harmful outputs
4. Reliability — prevention of hallucinations, unauthorized tool calls
5. Accountability — governance structures, vendor oversight
6. Society — guardrails against catastrophic misuse

**kaits coverage assessment:** ~60%. No competitor (CrewAI, LangGraph, AutoGen) advertising AIUC-1 compliance. OpenAI absent from consortium; Anthropic is a member.

---

## Stream 4: Strategic Synthesis

### kaits Positioning

Kaits is **meta** to vibe-coding platforms — it could orchestrate agents that *use* them.
- **Vibe coding:** builds one app → kaits runs your AI department
- **Developer tools:** augment one developer → kaits manages agent teams
- **Super apps:** consolidate AI tools → kaits governs AI agents

### kaits + aibox Relationship

- **aibox** = infrastructure layer (container environment, context schema, skill installation)
- **kaits** = application layer (agent orchestration, governance, company simulation)
- **SKILL.md spec** = shared contract between the two
- **aibox** owns container-level skill infrastructure; **kaits** owns agent-level skill semantics (XP, progression, capability mapping)

### Key Opportunities

1. SKILL.md adoption — align company-plugins with open standard
2. Skill creation as gameplay — agents "learn" skills through use, evals provide XP metrics
3. Agent team templates — pre-configured teams for quick onboarding
4. AIUC-1 first-mover — close the 40% gap for unique enterprise positioning
5. File-to-agent — upload process document → configured agent

### Key Threats

1. Ecosystem convergence — Agent Skills standard mandates features kaits' plugin format lacks
2. ClawHub network effects — community marketplace could eclipse self-hosted distribution
3. Commoditization of orchestration — if Replit/OpenAI add multi-agent governance
4. "Good enough" wins — vibe coding produces mediocre but instant results

### Priority Recommendations

| Priority | Recommendation |
|---|---|
| P1 | Adopt SKILL.md open standard in company-plugins |
| P1 | Skill creation as RPG gameplay mechanic |
| P1 | Agent team templates |
| P2 | AIUC-1 alignment assessment |
| P2 | Prompt injection defense in provider pipeline |
| P3 | External skill marketplace with safety controls |
| P3 | File-to-agent feature |
