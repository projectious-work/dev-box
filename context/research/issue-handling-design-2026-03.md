# Issue Handling System Design

**Date:** 2026-03-26
**Task:** BACK-046
**Status:** Draft

---

## 1. Problem Statement

Developers working with AI agents get interrupted by bug reports, customer issues,
and production incidents. When a user says "oh wait, user X just reported that login
is broken" in the middle of a refactoring session, several things go wrong:

1. **Context destruction** -- the agent loses focus on the current task. Resuming
   after an interruption is expensive (context window pollution, lost reasoning chains).
2. **Undocumented issues** -- the report gets handled ad-hoc or forgotten entirely.
   There is no structured capture, no severity assignment, no tracking.
3. **No delegation path** -- the current agent must handle everything sequentially.
   There is no way to say "log this and have someone else deal with it" without
   manual process.
4. **No per-issue history** -- when the same user reports a second problem, there is
   no easy way to see their prior reports, environment details, or resolution history.

The aibox context system has BACKLOG.md for planned work and STANDUPS.md for session
notes, but neither is designed for the reactive, interrupt-driven nature of issue
handling. A backlog item is deliberate and planned; an issue is urgent and unplanned.

---

## 2. Landscape Analysis

### 2.1 How AI Coding Agents Handle Interruptions Today

The dominant pattern in 2026 is **no structured interruption handling**. Most AI
coding agents (Claude Code, Codex CLI, Aider, Copilot CLI) operate in a single
conversational thread. When interrupted:

- The user types the interruption inline, breaking the current task's flow.
- The agent context-switches, handles the interruption, then tries to resume.
- Context window usage grows; earlier reasoning may be compacted or lost.
- There is no record that an interruption occurred, what triggered it, or how it
  was resolved.

McKinsey's 2026 analysis of agentic workflows confirms: "Interrupting mid-workflow
destroys the speed advantage, and reviewing partial work without full context leads
to worse decisions."

### 2.2 Claude Code's Task/Sub-agent Model

Claude Code now supports sub-agents via the Task tool:

- **Sub-agents** run in isolated context windows with custom system prompts and
  restricted tool access.
- Up to 7 sub-agents can run in parallel.
- **Agent Teams** (experimental) allow multiple Claude Code sessions to coordinate,
  with one session acting as team lead.
- Sub-agents can be spawned for specific scoped work (file reads, searches, code
  changes) and return results to the parent agent.

This is the key enabling technology for issue handling: the main agent can spawn a
sub-agent to handle an issue without losing its own context.

### 2.3 GitHub Agentic Workflows

GitHub Agentic Workflows (technical preview, Feb 2026) allow:

- Markdown-defined automation that converts to GitHub Actions.
- Native access to issues, PRs, and repository state via GitHub MCP Server.
- Automatic issue triage and labeling.
- Triggers based on issue events, schedules, or manual dispatch.

This provides a potential downstream integration: issues captured by aibox could be
auto-filed to GitHub and picked up by agentic workflows for triage.

### 2.4 Linear/Jira AI Integration

Both Linear and Jira now support AI-driven triage:

- Linear Agent (public beta) can auto-triage issues entering the backlog.
- Factory AI provides cross-platform triage rules based on labels, priority, or
  custom criteria.
- Organizations report up to 40% productivity gains from AI-driven issue assignment.

For aibox, the relevant pattern is: capture locally in the context system, then
optionally sync to an external tracker (GitHub Issues, Linear, Jira) as a separate
concern.

### 2.5 Incident Response Patterns

PagerDuty and Opsgenie workflows follow a consistent model:

1. **Alert** -- automated detection or human report
2. **Triage** -- severity assessment, responder assignment
3. **Mitigate** -- immediate action to reduce impact
4. **Resolve** -- root cause fix
5. **Postmortem** -- document and prevent recurrence

The aibox `incident-response` skill already covers steps 1-5 for production incidents.
The gap is in the **capture and delegation** phase -- getting from "someone mentioned
a problem" to "it is documented and assigned" without derailing the current session.

---

## 3. Architecture Options

### Option A: Single Agent Context Switch

The current de facto model. The agent stops what it is doing, handles the issue
inline, then resumes.

| Aspect | Assessment |
|--------|-----------|
| Complexity | None -- already how things work |
| Context cost | High -- pollutes the current context window |
| Tracking | None -- issue is buried in conversation history |
| Resumption | Poor -- agent must reconstruct prior reasoning |
| Parallel work | Impossible |

**Verdict:** Baseline. Not a solution, just the status quo.

### Option B: Sub-agent Delegation (Recommended for MVP)

The main agent spawns a sub-agent via the Task tool to handle the issue. The
sub-agent runs in its own context window, captures the issue, and returns a
summary. The main agent continues its work with minimal interruption.

| Aspect | Assessment |
|--------|-----------|
| Complexity | Low -- uses existing Claude Code Task tool |
| Context cost | Low -- only the summary re-enters the main context |
| Tracking | Good -- sub-agent writes to BACKLOG.md or ISSUES.md |
| Resumption | Good -- main agent never lost its context |
| Parallel work | Yes -- sub-agent runs independently |

**Flow:**
```
User: "oh wait, user X says login is broken"
  |
  v
Main agent recognizes issue trigger
  |
  v
Spawns sub-agent with issue-handling skill
  - Input: user's report, current project context paths
  - Tools: Read, Edit (scoped to context/), Bash (for gh CLI)
  - Task: capture issue, assess severity, write to tracking file
  |
  v
Sub-agent executes:
  1. Creates structured issue entry in context/BACKLOG.md (or ISSUES.md)
  2. Optionally files GitHub issue via `gh issue create`
  3. Returns one-line summary to main agent
  |
  v
Main agent acknowledges: "Logged BACK-089: Login broken for user X (P1)"
Main agent continues previous work
```

**Verdict:** Best balance of simplicity and effectiveness. Recommended for Phase 1.

### Option C: Separate Session

Save current session state, start a new Claude Code session for the issue, then
resume the original session.

| Aspect | Assessment |
|--------|-----------|
| Complexity | High -- requires session save/restore |
| Context cost | Zero -- completely separate context |
| Tracking | Excellent -- full session dedicated to the issue |
| Resumption | Medium -- session restore is imperfect |
| Parallel work | Yes -- if using Agent Teams |

**Verdict:** Overkill for issue capture. Appropriate for issues requiring deep
investigation, but that is a Phase 2 concern.

### Option D: Queue-based (Async)

Log the issue to a queue file. Process all queued issues in a dedicated triage
session (e.g., at next standup or scheduled interval).

| Aspect | Assessment |
|--------|-----------|
| Complexity | Low -- just append to a file |
| Context cost | Minimal -- one line of acknowledgment |
| Tracking | Good -- queue file is the tracking artifact |
| Resumption | Excellent -- almost no interruption |
| Parallel work | Deferred -- handled later |

**Flow:**
```
User: "user X says login is broken"
  |
  v
Agent appends to context/ISSUE_QUEUE.md:
  - 2026-03-26T14:30 | login broken for user X | reported-by: user | untriaged
  |
  v
Agent: "Queued for triage. Continuing with current work."
  |
  v
[Later, in triage session]
Agent reads ISSUE_QUEUE.md, triages each item, moves to BACKLOG.md
```

**Verdict:** Good complement to Option B. For low-severity reports, queuing is
better than immediate delegation. The skill should support both modes.

---

## 4. Recommended Architecture: B + D Hybrid

Combine sub-agent delegation (for urgent issues) with queue-based capture (for
everything else). The skill determines the mode based on severity signals.

### 4.1 Decision Matrix

| Signal | Action |
|--------|--------|
| User says "urgent", "down", "broken", "production" | **Immediate**: sub-agent delegation |
| User says "bug", "issue", "problem", "not working" | **Standard**: structured capture to BACKLOG.md |
| User says "minor", "cosmetic", "nice to have", "when you get a chance" | **Queued**: append to issue queue for next triage |
| Automated alert (test failure, CI break) | **Standard**: structured capture |

### 4.2 Issue Entry Format

All captured issues use a consistent format in BACKLOG.md:

```markdown
| BACK-089 | Bug: Login broken for user X | todo | must | **Type:** bug-report. **Reporter:** user X. **Severity:** P1-critical. **Observed:** Login page returns 500 after password submission. **Environment:** EU cluster, Chrome 124. **Repro:** [pending]. Filed via issue-handling skill, 2026-03-26T14:30Z. |
```

This reuses the existing BACKLOG.md format (ID, title, status, priority, notes)
rather than introducing a separate ISSUES.md file. Rationale:

- BACKLOG.md is already the source of truth for work items.
- Issues become work items once triaged -- no need for a separate artifact.
- The `Type: bug-report` tag in notes distinguishes issues from planned work.
- Existing backlog-context skill already knows how to manage this file.

### 4.3 When a Separate ISSUES.md Makes Sense

For teams with high issue volume (more than ~5 open issues at a time), a dedicated
`context/ISSUES.md` file prevents BACKLOG.md from becoming noise-dominated. The
file would use the same table format but with additional columns:

```markdown
# Issues

| ID | Title | Status | Severity | Reporter | Reported | Notes |
|----|-------|--------|----------|----------|----------|-------|
| ISS-001 | Login broken for user X | triaging | P1 | user X | 2026-03-26 | EU cluster, Chrome 124 |
```

This is a Phase 2 consideration. For MVP, BACKLOG.md is sufficient.

---

## 5. Skill Design: `issue-handling`

### 5.1 SKILL.md

```markdown
---
name: issue-handling
description: Captures, triages, and delegates reported issues with minimal
  interruption to ongoing work. Use when a user reports a bug, a customer
  problem, a test failure, or a production incident mid-session.
---

# Issue Handling

## When to Use

When the user:
- Reports a bug or problem ("X is broken", "user Y reported...")
- Mentions a customer complaint or support ticket
- Describes a test failure or CI break
- Reports a production incident or alert
- Says "issue", "bug", "incident", "problem", "not working", "broken"

Also triggered by:
- Test output showing failures (when running tests as part of other work)
- Error logs pasted into the conversation

## Instructions

### 1. Capture (do this immediately, before any investigation)

Extract from the user's report:
- **What** is broken (one-line summary)
- **Who** reported it (user name, customer, automated system)
- **When** it was observed (timestamp or "just now")
- **Where** it occurs (environment, URL, service, platform)
- **Severity** estimate:
  - P1-critical: service down, data loss, security breach
  - P2-high: major feature broken, workaround exists
  - P3-medium: minor feature broken, cosmetic, edge case
  - P4-low: enhancement request disguised as bug

### 2. Decide: Interrupt or Queue

**Interrupt current work** (spawn sub-agent or switch context) if:
- Severity is P1-critical
- User explicitly asks for immediate action
- The issue is in code you are currently modifying

**Continue current work** (capture and queue) if:
- Severity is P2 or lower
- User does not request immediate action
- The issue is unrelated to current work

Tell the user which path you are taking and why.

### 3. Document

Add an entry to context/BACKLOG.md:
- Use the next available BACK-NNN ID
- Increment the "Next ID" counter
- Title format: "Bug: <one-line summary>"
- Status: `todo`
- Priority: map severity (P1 -> must, P2 -> should, P3 -> could, P4 -> could)
- Notes: include Type, Reporter, Severity, Observed behavior, Environment,
  and the text "Filed via issue-handling skill, <timestamp>"

### 4. Optionally File Externally

If the project uses GitHub Issues and `gh` CLI is available:
- Run `gh issue create --title "<title>" --body "<structured details>"`
- Add the issue number to the BACKLOG.md entry notes

Do NOT file externally unless:
- The project's CLAUDE.md or work-instructions mention GitHub Issues
- The user explicitly requests it
- A prior pattern of GitHub issue references exists in BACKLOG.md

### 5. Report Back

Give the user a one-line confirmation:
- "Logged BACK-089: Login broken for user X (P1-critical, must). Continuing
  with <current task>."

If severity is P1 and you are continuing other work, add:
- "This is P1-critical. Want me to stop current work and investigate?"

### 6. Triage Queue Processing

When starting a new session or when the user asks to "triage issues":
- Read context/BACKLOG.md
- Filter items with "Filed via issue-handling skill" that have status `todo`
- For each: confirm severity, check for duplicates, update priority if needed
- Present a summary to the user for review

## Examples

**User (mid-refactoring):** "oh wait, customer Acme says their exports are
timing out since yesterday"

**Agent:** Spawns sub-agent to capture the issue. Sub-agent writes:
| BACK-089 | Bug: Export timeout for customer Acme | todo | should |
Type: bug-report. Reporter: customer Acme. Severity: P2-high.
Observed: exports timing out since 2026-03-25. Environment: unknown.
Filed via issue-handling skill, 2026-03-26T14:30Z. |

**Agent (to user):** "Logged BACK-089: Export timeout for Acme (P2-high).
Continuing with the refactoring."

---

**User:** "tests are failing on CI -- auth module returns 401 on valid tokens"

**Agent:** Recognizes this may relate to current work. Captures:
| BACK-090 | Bug: Auth module 401 on valid tokens (CI) | todo | must |
Type: test-failure. Reporter: CI. Severity: P1-critical. Observed: valid
tokens rejected, auth module. Filed via issue-handling skill. |

**Agent:** "Logged BACK-090: Auth 401 on valid tokens (P1-critical). This
is in the auth module -- want me to investigate now or continue current work?"
```

### 5.2 Skill Placement

The skill belongs in `templates/skills/issue-handling/SKILL.md` alongside the
existing 84 skills. It should be auto-deployed by:

- The `managed` process package (teams doing structured work need issue tracking)
- The `product` process package (product development encounters bugs constantly)
- Optionally available for `minimal` and `research` packages via explicit install

### 5.3 Relationship to Existing Skills

| Skill | Relationship |
|-------|-------------|
| `incident-response` | Handles **production incidents** (triage, communicate, fix, postmortem). Issue-handling captures the initial report and delegates; incident-response handles the response workflow. |
| `backlog-context` | Manages BACKLOG.md. Issue-handling writes entries in backlog format, reusing the same file and ID scheme. |
| `standup-context` | Records session activity. Issue-handling events appear in standups as "captured BACK-089" but the standup skill does not manage issues. |
| `agent-management` | Defines multi-agent coordination. Issue-handling uses the sub-agent delegation pattern from agent-management. |
| `debugging` | Provides debugging methodology. Once an issue is triaged and assigned, the debugging skill guides investigation. |
| `postmortem-writing` | Creates postmortems after resolution. Issue-handling captures; postmortem-writing closes the loop. |

---

## 6. CLI Integration

### 6.1 Command: `aibox issue`

A quick-capture command that writes an issue to BACKLOG.md without needing an
active AI agent session.

```
aibox issue "login page broken for user X"
aibox issue "export timeout for Acme" --severity p1 --reporter "customer Acme"
aibox issue list                          # show open issues (type: bug-report)
aibox issue triage                        # interactive triage of untriaged issues
```

**Implementation scope:**

```rust
// cli/src/cli.rs -- new subcommand
/// Capture and track issues in the project backlog
Issue {
    #[command(subcommand)]
    command: IssueCommands,
},

enum IssueCommands {
    /// Quick-capture an issue to BACKLOG.md
    Add {
        /// Issue description
        description: String,
        /// Severity: p1, p2, p3, p4
        #[arg(long, default_value = "p3")]
        severity: String,
        /// Who reported it
        #[arg(long)]
        reporter: Option<String>,
    },
    /// List open issues
    #[command(alias = "ls")]
    List,
    /// Interactive triage of untriaged issues
    Triage,
}
```

**What it does:**

1. Reads `context/BACKLOG.md`
2. Parses the "Next ID" counter
3. Appends a new row with the structured format
4. Increments the counter
5. Optionally runs `gh issue create` if `--github` flag is passed

**What it does NOT do:**

- Spawn an AI agent (this is a pure CLI operation)
- Require a running container
- Replace the skill (the skill handles mid-session captures with context;
  the CLI handles quick captures from the terminal)

### 6.2 Relationship to Existing Commands

| Command | Purpose |
|---------|---------|
| `aibox init` | Sets up project, seeds context files |
| `aibox sync` | Reconciles config with generated files |
| `aibox doctor` | Validates project structure |
| `aibox skill` | Manages installed skills |
| `aibox issue` | **New:** captures and tracks issues in context |

The `issue` command is the first aibox command that writes to `context/` files
directly (rather than to `.devcontainer/` or `.aibox-home/`). This establishes
a pattern for future context-manipulation commands (e.g., `aibox event` from
BACK-073, `aibox decision` for quick ADR capture).

---

## 7. Per-Customer Tracking

### 7.1 Is This Needed for aibox?

aibox is a developer tool, not a customer-facing service. Per-customer tracking
in the traditional sense (customer X has 3 open tickets, SLA is 4 hours) is not
a core aibox concern.

However, the broader vision includes:

- **Per-project tracking** -- when aibox manages multiple environments, issues
  should be scoped to the relevant project. This is already handled by each
  project having its own `context/BACKLOG.md`.
- **Per-reporter tracking** -- the `Reporter` field in issue entries enables
  filtering ("show me all issues reported by CI" or "all issues from customer
  Acme"). No separate data structure needed.

### 7.2 Kaits Orchestration Boundary

Per-customer agent instances (a dedicated issue-handling agent per customer) is
a kaits orchestration concern, not an aibox concern. The aibox issue-handling
skill provides the **capture and local tracking** primitive. Kaits would provide:

- Routing incoming reports to the right project's agent
- Spawning per-customer agent sessions
- Aggregating issue status across projects
- SLA tracking and escalation

For now, aibox should not attempt to solve the orchestration problem. The skill
and CLI command are the right scope.

---

## 8. Implementation Plan

### Phase 1: Skill + CLI (MVP)

**Effort:** ~2-3 days

1. Create `templates/skills/issue-handling/SKILL.md` with the content from
   Section 5.1.
2. Add `aibox issue add` CLI command (Section 6.1) -- parse BACKLOG.md, append
   entry, increment ID counter.
3. Add `aibox issue list` -- filter BACKLOG.md entries with "Type: bug-report"
   in notes.
4. Add issue-handling to the `managed` and `product` process package skill lists.
5. Test: manual capture via CLI, verify BACKLOG.md entries are well-formed.

### Phase 2: Sub-agent Integration

**Effort:** ~2-3 days, depends on Claude Code Task tool stability

1. Add a `references/sub-agent-prompt.md` to the skill directory containing the
   system prompt for the issue-handling sub-agent.
2. Document the sub-agent spawning pattern in the skill instructions (the skill
   teaches the agent how to use the Task tool for delegation).
3. Test: interrupt a refactoring session with a bug report, verify sub-agent
   captures the issue and main agent continues.

### Phase 3: External Integration

**Effort:** ~1-2 days per integration

1. GitHub Issues integration via `gh issue create` in both skill and CLI.
2. Optional `--github` flag on `aibox issue add`.
3. Bidirectional sync consideration: should `aibox issue list` also show GitHub
   issues? (Likely not for MVP -- adds complexity for little value.)

### Phase 4: Triage Workflow

**Effort:** ~2-3 days

1. `aibox issue triage` command that presents untriaged issues interactively.
2. Triage session skill extension: at session start, check for untriaged issues
   and prompt the user.
3. Integration with standup-context skill: include issue summary in standup notes.

---

## 9. Open Questions

1. **BACKLOG.md vs ISSUES.md** -- should high-volume projects use a separate file?
   Recommendation: start with BACKLOG.md, split when pain is felt. The skill can
   be updated to support either without breaking changes.

2. **ID scheme** -- should issues use the same BACK-NNN sequence or a separate
   ISS-NNN sequence? Recommendation: same sequence (BACK-NNN). Issues become
   backlog items once triaged. Separate sequences create confusion about which
   number to reference.

3. **AI-provider independence** -- the sub-agent delegation pattern is Claude Code
   specific (Task tool). For other providers (Codex, Aider), the skill would fall
   back to inline capture (Option A). Is this acceptable? Recommendation: yes,
   the skill instructions should include a fallback path for agents without
   sub-agent capability.

4. **Severity calibration** -- who decides severity? The agent's initial estimate
   may be wrong. Recommendation: the agent estimates, the triage step confirms.
   Users can override at any time with `aibox issue add --severity p1`.

---

## 10. Recommendation

**Start with Phase 1**: the `issue-handling` skill and `aibox issue add` CLI
command. This delivers immediate value (structured capture, no more lost bug
reports) with minimal implementation effort and no architectural risk.

The sub-agent delegation (Phase 2) is the key differentiator but depends on
Claude Code's Task tool maturity. Design the skill to work without sub-agents
first, then layer delegation on top.

Do not build per-customer tracking or external integrations until the basic
capture and triage loop is validated in daily use.

---

## Sources

- [Create custom subagents - Claude Code Docs](https://code.claude.com/docs/en/sub-agents)
- [The Task Tool: Claude Code's Agent Orchestration System](https://dev.to/bhaidar/the-task-tool-claude-codes-agent-orchestration-system-4bf2)
- [Subagents in the SDK - Claude API Docs](https://platform.claude.com/docs/en/agent-sdk/subagents)
- [Claude Code Agent Skills 2.0](https://medium.com/@richardhightower/claude-code-agent-skills-2-0-from-custom-instructions-to-programmable-agents-ab6e4563c176)
- [Extend Claude with skills - Claude Code Docs](https://code.claude.com/docs/en/skills)
- [GitHub Agentic Workflows](https://github.blog/ai-and-ml/automate-repository-tasks-with-github-agentic-workflows/)
- [Agentic workflows for software development (McKinsey)](https://medium.com/quantumblack/agentic-workflows-for-software-development-dc8e64f4a79d)
- [The State of AI Coding Agents 2026](https://medium.com/@dave-patten/the-state-of-ai-coding-agents-2026-from-pair-programming-to-autonomous-ai-teams-b11f2b39232a)
- [AI Bug Triage: Automated GitHub and Jira Issue Assignment](https://www.webelight.com/blog/bug-triage-agents-ai-github-jira-automation)
- [Linear Auto-apply Triage Suggestions](https://linear.app/changelog/2025-09-19-auto-apply-triage-suggestions)
