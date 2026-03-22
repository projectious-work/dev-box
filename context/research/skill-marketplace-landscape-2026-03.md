# Skill Marketplace Landscape — March 2026

Research conducted 2026-03-22 to inform skill library expansion planning.

## Agent Skills Standard

Anthropic released the Agent Skills specification as an open standard via agentskills.io/specification. Format: directory with SKILL.md (YAML frontmatter + markdown). Adopted by Microsoft, OpenAI (Codex CLI), Cursor, GitHub Copilot, Windsurf, Gemini CLI.

Progressive disclosure model:
- **Level 1 — Metadata (~100 tokens):** name + description loaded at startup
- **Level 2 — Instructions (<5000 tokens):** full SKILL.md body on activation
- **Level 3 — Resources (unbounded):** scripts/, references/, assets/ on demand

Official repo: github.com/anthropics/skills — 17 reference skills including frontend-design (breakout hit: 124K-277K installs).

## Major Marketplaces

| Platform | Scale | Model | Notes |
|----------|-------|-------|-------|
| SkillsMP (skillsmp.com) | ~97,000 skills | Free, GitHub-scraped | Largest by volume. No curation. |
| Skills.sh (Vercel) | ~40,000+ skills | Free, CLI install (`npx skills add`) | Leaderboard. Top skill: 26K installs |
| LobeHub | ~7,000+ skills | Free | AI-evaluated quality scores |
| ClawHub/OpenClaw (clawhub.ai) | ~13,700 skills | Free, versioned registry | npm-like: `npx clawhub install` |
| agent-skills.md | Curated directory | Free | Browse/preview/download |
| Skills Directory (skillsdirectory.com) | Curated | Free | Security-focused: every skill scanned |
| PRPM (prpm.dev) | 7,000+ | npm-like prompt package manager | Auto-converts between formats |
| Cursor Marketplace | Official (Feb 2026) | Partners: Amplitude, AWS, Figma | Bundles MCP + skills + agents |

## Most Popular Skills (by install count, March 2026)

1. `find-skills` — 418,600 installs (meta-skill for discovery)
2. `vercel-react-best-practices` — 176,400
3. `web-design-guidelines` — 137,000
4. `remotion-best-practices` — 126,000
5. `frontend-design` (Anthropic official) — 124,100-277,000

Pattern: frontend/React dominates. Infrastructure and data skills vastly underserved.

## Quality Crisis

HuggingFace analysis of 40K+ skills found:
- **46.3% are duplicates or near-duplicates**
- Growth is hype-driven (18.5x increase in 20 days tied to social media), not quality-driven
- Community consensus: 80% of skills are "slop"
- Ecosystem lacks "an AI npm" that surfaces best tools

## Category Taxonomies (cross-platform synthesis)

**High demand, well served:** Frontend/framework-specific (React, Next.js, Vue)
**High demand, underserved:** Infrastructure/DevOps, Database/data modeling, Security, AI/ML/RAG
**Medium demand:** Testing/TDD, API design, Documentation
**Low representation:** Observability, Architecture (beyond basic clean code), Domain-specific backends (CQRS, DDD)

## Best Practice Patterns

**High-quality skills:**
- Under 150 lines SKILL.md, reference files for depth
- Clear trigger conditions in description ("a little pushy" per Anthropic — agents undertrigger)
- Testable outcomes, not just instructions
- Reference materials bundled (cheatsheets, checklists)
- Framework-version-aware, not generic advice

**Low-quality skills:**
- Vague/broad instructions ("write good code")
- No trigger conditions, no metadata
- Copy-pasted documentation
- No testing or validation

## Implications for dev-box

Our curated library addresses the quality gap. Key differentiators:
1. Progressive disclosure with reference files
2. Consistent format across all 83 skills
3. Cross-cutting composability (skills reference each other)
4. Universal applicability (language-agnostic where possible)
5. Security vetting (trusted source, no prompt injection risk)
6. Offline-first (embedded in binary)

## Sources

- agentskills.io/specification
- github.com/anthropics/skills
- skillsmp.com, skills.sh, lobehub.com/skills, clawhub.ai
- skillsdirectory.com, agent-skills.md, prpm.dev
- cursor.com/marketplace, cursor.directory
- HuggingFace blog: agent-skills-analysis
- "Your Agent Skills Are All Slop" (12gramsofcarbon.com)
