---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-SleekDove-mozilla-cq-evaluation-march
  created: 2026-04-11
spec:
  title: "Mozilla cq Evaluation — March 2026"
  type: reference
  state: permanent
  tags: [cq, mozilla, shared-knowledge, mcp, skills, integration, evaluation]
  skill: research-with-confidence
  source_file: mozilla-cq-evaluation-2026-03.md
---

# Mozilla cq Evaluation — March 2026

Evaluation of Mozilla.ai's cq ("colloquy") tool for potential integration with aibox. cq is a shared agent knowledge system — "Stack Overflow for AI agents" — that lets coding agents persist, share, and query collective knowledge to avoid rediscovering the same failures independently.

---

## 1. What Is cq

- **Local-first**: Works offline with a private SQLite database; no server required for basic use
- **Three-tier knowledge model**: Local (private) → Team/Org (shared within company) → Global Commons (public, community-governed)
- **Model-agnostic**: Works with any LLM or agent framework
- **Open standard**: Apache 2.0 license, open protocol and formats

**Six MCP tools**: `query`, `propose`, `confirm`, `flag`, `reflect`, `status`

**Installation:**
```bash
claude plugin marketplace add mozilla-ai/cq
claude plugin install cq
```

**Repository health (as of March 2026):** ~750 stars, v0.4.0, 65 commits, exploratory prototype, less than 1 month old, backed by Mozilla.ai.

---

## 2. Relationship to aibox

**Not overlapping. Complementary at a different layer.**

| Dimension | aibox skills | cq knowledge |
|---|---|---|
| Nature | Procedural ("how to do X") | Declarative ("X has pitfall Y") |
| Authorship | Human-curated, version-controlled | Agent-proposed, human-approved |
| Scope | Project/team process | Cross-project technical facts |
| Distribution | aibox skill registry | MCP server + tiered stores |

- **vs. aibox context system**: aibox context = project-specific state ("what are *this project's* decisions?"); cq = cross-project technical knowledge ("what have *all agents* learned about Stripe's API?")
- **vs. aibox addons**: cq is an MCP server + agent plugin, not a development tool; loose fit for addon model

---

## 3. Integration Options

| Option | Description | Verdict |
|---|---|---|
| A: Addon (container-level) | Install cq MCP server in devcontainer | Poor fit — addon alone delivers half the value |
| **B: Skill (agent-level)** | `cq-integration` skill with triggers for query/propose | **Best fit if we integrate at all** |
| **C: Documentation only** | Mention cq as compatible companion tool | **Recommended for now** |
| D: Skip entirely | Ignore for now | Misses emerging category signal |

---

## 4. Recommendation

**Short-term: Option C — Documentation only.**
- Too young (< 1 month, exploratory prototype) for integration effort
- Global knowledge commons empty or near-empty
- Trust model entirely theoretical at this stage

**Medium-term (Q3 2026): Reassess for Option B (skill) when:**
- cq reaches v1.0+
- Global commons has >1000 confirmed knowledge units
- Trust/graduation model operational
- ≥2 other major agent platforms have adopted cq
- Mozilla.ai demonstrates sustained maintenance

**Key insight:** Track the *category* (shared agent knowledge as open standard), not just the project. If cq fails but the category succeeds, integrate the winner.
