---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-SleekPlum-scheduled-recurring-tasks-for
  created: 2026-04-11
spec:
  title: "Scheduled / Recurring Tasks for aibox — Research Report"
  type: reference
  state: permanent
  tags: [scheduling, cron, automation, headless, provider-independence]
  skill: research-with-confidence
  source_file: scheduled-tasks-2026-03.md
---

# Scheduled / Recurring Tasks for aibox — Research Report

**Date:** 2026-03-26
**Status:** Draft

---

## 1. Problem Statement

aibox provides reproducible containerized development environments with built-in AI context
structure. Users increasingly want AI agents to perform work autonomously on a schedule --
nightly code audits, daily standup generation, periodic dependency updates -- without manual
invocation. Today, aibox has no scheduling primitive. This report surveys the landscape of
AI agent scheduling, evaluates design options, and recommends a practical approach that
preserves aibox's provider-independence principle.

---

## 2. Landscape Survey

### 2.1 Claude Code -- Headless Mode and Scheduled Tasks

Claude Code supports non-interactive execution via the `-p` (print) flag, which runs without
a TTY and is suitable for cron or CI integration. Key facts:

- **Headless mode:** `claude -p "audit this codebase for security issues"` runs a one-shot
  task and exits. Uses ~512 MB (vs 1-2 GB for the interactive TUI).
- **Desktop scheduled tasks:** Claude Code Desktop (macOS/Windows only) supports persistent
  scheduled tasks that survive restarts. Not available on Linux.
- **CLI `/loop` command:** Session-scoped scheduling that runs a prompt repeatedly at an
  interval. Expires after 3 days if the Desktop app is not running.
- **Background Agents:** Available on all paid plans. Cloud-based execution of tasks without
  requiring a local machine.
- **Linux pattern:** Set up standard cron jobs that invoke `claude -p` in headless mode.
  This is the recommended approach for server/container environments.

**Relevance to aibox:** The `-p` flag is the key integration point. aibox can invoke any
provider's CLI in non-interactive mode from a scheduler. Claude Code does not provide its
own cross-platform persistent scheduler -- it relies on OS-level cron or its Desktop app.

### 2.2 Aider -- Watch Mode (File-Based Triggering)

Aider uses a file-watching model rather than time-based scheduling:

- **`--watch-files` flag:** Monitors all repo files for special AI comment markers.
- **AI comment syntax:** One-liner comments starting or ending with `AI!` (execute) or
  `AI?` (answer). Example: `// AI! refactor this function to use async/await`
- **No time-based scheduling:** Aider has no built-in cron or interval capability.

### 2.3 GitHub Agentic Workflows

GitHub launched Agentic Workflows in technical preview (February 2026):

- **Markdown-based workflow definition:** Workflows written in plain Markdown in `.github/workflows/`.
- **`gh aw` CLI:** Converts Markdown workflows into GitHub Actions `.lock.yml` files.
- **Trigger types:** Issues, PRs, pushes, comments, manual dispatch, and **schedules**
  (cron expressions with timezone support as of March 2026).
- **Provider-agnostic:** Supports GitHub Copilot (default), Claude, and Codex as execution engines.
- **Status:** Technical preview.

### 2.4 OpenAI Codex -- Automations

The Codex app (desktop + cloud) has first-class automation support:

- **Automations:** Defined by frequency/trigger, instructions (prompt or skill), and an
  optional agent personality. Results land in a "review queue" tab.
- **Use cases at OpenAI:** Daily issue triage, CI failure summaries, daily release briefs, bug scanning.
- **Local vs cloud execution:** Currently local (app must be running); extending to cloud-based.
- **Worktree support:** Can run on a new worktree to avoid conflicts with active development.
- **CLI non-interactive mode:** `codex -p "prompt"` for headless execution.

### 2.5 Cron-Based AI Agent Patterns

**Pattern 1: Standard Cron** — Time-based triggers (`0 2 * * *`). Best for daily reports, audits.

**Pattern 2: Interval Loop** — Agent runs every X minutes/hours. Risk of drift if previous job runs long.

**Pattern 3: Event-Driven Triggers** — Execution in response to external events.

**Pattern 4: Adaptive Scheduling** — The agent sets its own next wakeup time.

**Pattern 5: Heartbeat** — Periodic poll (every 30 minutes). "Anything need attention?"

**Key production concerns:**
- Timeouts are mandatory (LLMs can hang). Always wrap scheduled jobs in strict timeouts.
- AI agents are stateful — they often need to "remember" what they did for the next run.
- Cost control: heartbeats that always invoke an LLM are expensive. "Cheap checks first, model only when needed."

### 2.6 MCP (Model Context Protocol) -- Scheduling Status

MCP does not currently have scheduling or trigger concepts in the specification:

- **2026 roadmap:** Triggers and event-driven updates are listed in the "On the Horizon" section.
- **Current state:** Clients learn about server-side changes by polling or holding SSE connections.

---

## 3. Use Cases

- **Scheduled Code Audits:** Run nightly, report findings.
- **Recurring Test Runs with AI Analysis:** After nightly test suite, pipe output to AI for failure analysis.
- **Daily Project Summaries (Standup Generation):** Every morning at 8 AM, generate standup from git log and open issues.
- **Periodic Dependency Updates:** Weekly check for outdated deps, create branch with updates, run tests.
- **Overnight Refactoring Tasks:** Long-running refactoring executed overnight with worktree isolation.
- **Scheduled Documentation Review:** Weekly review of docs for staleness, broken links, missing coverage.

---

## 4. Design Options

### Option A: CLI Command
`aibox schedule "run /audit every day at 2am"` — natural language parsed into cron.

### Option B: Config in aibox.toml (Recommended)

```toml
[[schedule]]
name = "nightly-audit"
cron = "0 2 * * *"
command = "/audit"
provider = "claude"
timeout = "10m"
workdir = "worktree"
```

`aibox sync` reads `[[schedule]]` entries and generates crontab entries. Schedules are
declarative, version-controlled, and survive container rebuilds.

### Option C: Skill-Based (Teach Agents to Use System Cron)
Zero implementation cost but fragile, non-reproducible, invisible. Non-starter.

### Option D: Integration with Provider-Native Scheduling
Violates aibox's provider-independence principle.

### Option E: Companion Scheduler Container
Architecturally clean but adds significant operational complexity.

---

## 5. Recommendation

**Primary: Option B (Config in aibox.toml) with Option A as a convenience layer.**

1. Config-driven scheduling is the natural fit for aibox — `aibox.toml` is the single source of truth.
2. Provider independence achieved through indirection — `command` field contains the prompt; runtime resolves to `claude -p`, `codex -p`, etc.
3. Container lifecycle solved by regeneration — schedules declared in `aibox.toml` survive rebuilds.
4. `aibox schedule add` as thin wrapper that appends a `[[schedule]]` entry to `aibox.toml`.

### Implementation Phases

| Phase | Deliverable |
|-------|------------|
| Phase 1 | `[[schedule]]` table array in aibox.toml; `aibox sync` generates crontab; execution wrapper script |
| Phase 2 | Structured log at `context/.schedule-log.jsonl`; `aibox schedule status` command |
| Phase 3 | `aibox schedule add/list/remove/run` CLI commands |
| Phase 4 | Worktree isolation, notification hooks, heartbeat mode, dependency chains |

### Example Configuration

```toml
[[schedule]]
name    = "nightly-audit"
cron    = "0 2 * * *"
command = "review src/ for security vulnerabilities and write findings to context/audit-latest.md"
timeout = "10m"

[[schedule]]
name    = "daily-standup"
cron    = "0 8 * * 1-5"
command = "generate a standup summary from the last 24h of git commits and open backlog items"
output  = "context/STANDUPS.md"
timeout = "5m"

[[schedule]]
name     = "weekly-deps"
cron     = "0 3 * * 0"
command  = "check for outdated dependencies, update them, run tests, and report results"
workdir  = "worktree"
timeout  = "15m"
```
