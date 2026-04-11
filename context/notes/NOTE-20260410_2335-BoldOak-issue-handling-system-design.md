---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-BoldOak-issue-handling-system-design
  created: 2026-04-11
spec:
  title: "Issue Handling System Design"
  type: reference
  state: permanent
  tags: [issues, interruptions, sub-agents, delegation, skill, cli, triage]
  skill: research-with-confidence
  source_file: issue-handling-design-2026-03.md
---

# Issue Handling System Design

**Date:** 2026-03-26
**Task:** BACK-046

---

## 1. Problem Statement

Developers working with AI agents get interrupted by bug reports mid-session. The current de facto model — the agent stops what it is doing, handles the issue inline, then resumes — causes:

1. **Context destruction** — agent loses focus on current task; context window polluted
2. **Undocumented issues** — report handled ad-hoc or forgotten entirely
3. **No delegation path** — current agent must handle everything sequentially
4. **No per-issue history** — no way to see prior reports or resolution history

---

## 2. Architecture Options

| Option | Context cost | Tracking | Parallel work |
|---|---|---|---|
| A: Single context switch (status quo) | High | None | Impossible |
| **B: Sub-agent delegation (Recommended)** | Low (only summary re-enters) | Good (writes to BACKLOG.md) | Yes |
| C: Separate session | Zero | Excellent | Yes (with Agent Teams) |
| D: Queue-based (async) | Minimal | Good (ISSUE_QUEUE.md) | Deferred |

---

## 3. Recommended Architecture: B + D Hybrid

**Sub-agent delegation** for urgent issues (P1-critical, production down) + **queue-based capture** for everything else. Severity signals determine the mode.

### Decision Matrix

| Signal | Action |
|--------|--------|
| "urgent", "down", "broken", "production" | **Immediate**: sub-agent delegation |
| "bug", "issue", "problem", "not working" | **Standard**: structured capture to BACKLOG.md |
| "minor", "cosmetic", "nice to have" | **Queued**: append to issue queue for next triage |
| Automated alert (test failure, CI break) | **Standard**: structured capture |

### Issue Entry Format

Issues are captured directly in BACKLOG.md (not a separate ISSUES.md) using the existing table format:

```
| BACK-089 | Bug: Login broken for user X | todo | must |
  Type: bug-report. Reporter: user X. Severity: P1-critical.
  Observed: Login page returns 500 after password submission.
  Environment: EU cluster, Chrome 124. Filed via issue-handling skill, 2026-03-26. |
```

Rationale: Issues become work items once triaged — no need for a separate artifact. The `Type: bug-report` tag distinguishes them.

---

## 4. Skill Design: `issue-handling`

**Trigger:** When user reports a bug, customer problem, test failure, or production incident mid-session.

**Instructions:**
1. Capture: extract What/Who/When/Where and assign severity (P1-critical → P4-low)
2. Decide: interrupt (P1 or user requests) or queue (P2+)
3. Document: write structured entry to BACKLOG.md
4. Optionally file GitHub issue via `gh issue create` (only if project uses GitHub)
5. Report back with one-line confirmation: "Logged BACK-089: Login broken (P1, must). Continuing with <current task>."

**Severity → Priority mapping:** P1→must, P2→should, P3→could, P4→could

**Relationship to existing skills:**

| Skill | Relationship |
|-------|-------------|
| `incident-response` | Issue-handling captures initial report; incident-response handles the response workflow |
| `backlog-context` | Issue-handling writes to BACKLOG.md using the same file and ID scheme |
| `debugging` | Used once issue is triaged and investigation begins |
| `postmortem-writing` | Closes the loop after resolution |

---

## 5. CLI Integration: `aibox issue`

Quick-capture command that writes to BACKLOG.md without needing an active AI agent session.

```bash
aibox issue "login page broken for user X"
aibox issue "export timeout for Acme" --severity p1 --reporter "customer Acme"
aibox issue list                    # show open issues (type: bug-report)
aibox issue triage                  # interactive triage of untriaged issues
```

This is the first aibox command that writes to `context/` files directly, establishing a pattern for future context-manipulation commands.

---

## 6. Implementation Phases

| Phase | Deliverable | Effort |
|-------|------------|--------|
| **Phase 1 (MVP)** | `issue-handling` skill + `aibox issue add` CLI command | ~2-3 days |
| Phase 2 | Sub-agent delegation via Task tool; `references/sub-agent-prompt.md` | ~2-3 days |
| Phase 3 | GitHub Issues integration via `gh issue create` | ~1-2 days per integration |
| Phase 4 | `aibox issue triage` interactive command; standup integration | ~2-3 days |

---

## 7. Open Questions

1. **BACKLOG.md vs ISSUES.md**: Start with BACKLOG.md, split when pain is felt.
2. **ID scheme**: Same BACK-NNN sequence (issues become backlog items once triaged — separate sequences create confusion).
3. **Provider independence**: Sub-agent delegation is Claude Code specific (Task tool). Skill should include fallback path for agents without sub-agent capability.
4. **Severity calibration**: Agent estimates, triage step confirms. Users can override at any time.
