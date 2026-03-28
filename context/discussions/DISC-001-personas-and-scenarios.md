---
id: DISC-001-appendix
title: "DISC-001 Appendix: Personas and Validation Scenarios (Detailed)"
status: active
date: 2026-03-28
parent: DISC-001
---

# DISC-001 Appendix: Personas and Validation Scenarios

This document captures the detailed persona definitions and validation scenario
walkthroughs for the context system redesign. It serves as the primary reference
for validating design decisions from DISC-001 against real-world usage patterns.

---

## Part 1: Naming Consistency Resolution

Before the walkthroughs, we resolve naming inconsistencies found across the design
documents.

### Directory naming: `context/items/` → `context/<primitive>/`

**Decision:** Rename `context/items/work/` to `context/work-item/` etc. — each primitive
gets a top-level directory under `context/` named after the primitive type. This is
consistent with the Kubernetes convention where resource types have their own paths.

| Primitive | Directory | Kind (YAML) | ID Prefix | Filename Pattern |
|---|---|---|---|---|
| Work Item | `context/work-item/` | `WorkItem` | `BACK-` | see below |
| Decision | `context/decision/` | `Decision` | `DEC-` | see below |
| Actor | `context/actor/` | `Actor` | `ACTOR-` | see below |
| Role | `context/role/` | `Role` | `ROLE-` | see below |
| Process | `context/process/` | `Process` | `PROC-` | see below |
| Gate | `context/gate/` | `Gate` | `GATE-` | see below |
| Metric | `context/metric/` | `Metric` | `MET-` | see below |
| Schedule | `context/schedule/` | `Schedule` | `SCHED-` | see below |
| Scope | `context/scope/` | `Scope` | `SCOPE-` | see below |
| Constraint | `context/constraint/` | `Constraint` | `CON-` | see below |
| Discussion | `context/discussion/` | `Discussion` | `DISC-` | see below |
| Artifact | `context/artifact/` | `Artifact` | `ART-` | see below |
| Event | `context/event/` | (JSONL) | `EVT-` | `YYYY-MM.jsonl` |

**Inconsistencies fixed:**
- Gate/Checkpoint: standardized to `Gate` everywhere (directory, kind, prefix)
- Scope/Project: standardized to `Scope` with single prefix `SCOPE-`
  (the word "project" is a subtype of scope, not a different primitive)

### Filename convention: human-readable names, not just word-IDs

**Decision (revised):** Filenames follow a Kubernetes-inspired pattern where the user
chooses a human-meaningful name, and the word-ID is stored inside the YAML only.

**Two patterns based on entity lifecycle:**

**Pattern A — Human-named entities (low-volume, long-lived):**
Actors, Roles, Processes, Gates, Metrics, Schedules, Constraints, Scopes.
These are created by humans and have meaningful names.

Filename: `<KIND>-<human-chosen-name>.md`
Examples:
- `ACTOR-alex-chen.md` (not `ACTOR-bold-crane.md`)
- `ROLE-admin.md` (not `ROLE-swift-oak.md`)
- `ROLE-developer.md`
- `PROC-code-review.md`
- `GATE-merge-readiness.md`
- `SCOPE-v1-launch.md`
- `SCHED-weekly-review.md`
- `CON-wip-limit.md`

The word-ID (`bold-crane`) lives in `metadata.id` inside the YAML. The filename is the
human-readable name. `aibox lint` checks that filenames are unique within a directory.
If a name conflict occurs, the CLI prompts for a different name.

**Pattern B — Auto-generated entities (high-volume, created by agents):**
Work Items, Decisions, Discussions, Events, Artifacts.
These are created frequently, often by agents, and benefit from automatic naming.

Filename: `<KIND>-<word-id>-<content-slug>.md`
Examples:
- `BACK-calm-lark-dark-mode-toggle.md`
- `DEC-keen-fox-use-postgresql.md`
- `DISC-bright-owl-database-choice.md`
- `ART-swift-tern-api-spec-v2.md`

The content slug is derived from the title at creation time. It does NOT update if the
title changes — it's a historical hint (like GitHub PR URLs). The word-ID ensures
uniqueness; the slug provides human context.

**Rationale:** When a human sees `ACTOR-bold-crane.md` they waste mental energy trying to
connect "bold crane" to Alex. But `ACTOR-alex-chen.md` is immediately clear. For work
items, the volume is high and agents create them — the word-ID is the stable identifier,
and the slug provides enough context for browsing.

---

## Part 2: The 6 Personas

### Persona 1: Alex Chen — Solo Developer

**Profile:**
- Freelance full-stack developer, 5 years experience
- Works on 2-3 client projects simultaneously
- Uses Claude Code as primary AI assistant
- Comfortable with terminal workflows
- Values speed over ceremony — minimal process overhead
- No team — works alone with AI agents

**Expertise:** Python, TypeScript, React, Docker, PostgreSQL

**Goals:**
- Ship client features fast with minimal overhead
- Keep a clean record of decisions for client handoffs
- Never lose context between work sessions

**Frustrations:**
- Setting up project structure from scratch every time
- Forgetting why I made a decision 3 weeks ago
- Losing track of what's done vs in-progress across projects

**Process needs:** `managed` package
**Typical project:** 20-100 work items, 1-3 month duration
**Agent usage:** One Claude Code agent, daily interaction

---

### Persona 2: Dr. Priya Sharma — Research Scientist

**Profile:**
- Computational biologist at a university lab
- Leads a team of 2 PhD students and 1 postdoc
- Manages research involving data analysis pipelines, paper writing, grant reporting
- Uses Python for analysis, LaTeX for papers
- Needs reproducibility and audit trails for publications

**Expertise:** Python, R, bioinformatics, statistics, LaTeX

**Goals:**
- Track experiments and outcomes reproducibly
- Manage paper writing pipeline from draft to publication
- Keep grant milestones on track
- Coordinate work across lab members without heavy tooling

**Frustrations:**
- No structured way to track which experiments led to which findings
- Paper revisions are chaotic — losing track of reviewer comments
- Grant reporting requires digging through months of unstructured notes

**Process needs:** `research` package (expanded)
**Typical project:** 50-200 items per research project, 6-24 month duration
**Agent usage:** AI assistant for literature review, data analysis, writing

---

### Persona 3: Maria Torres — Small Team Lead

**Profile:**
- Engineering lead at a 15-person startup
- Manages a team of 4 developers
- Responsible for release planning, code quality, and technical decisions
- Uses GitHub for code, wants structured context for AI-assisted development
- Needs RBAC — developers shouldn't change release processes

**Expertise:** Go, Kubernetes, system design, team management

**Goals:**
- Consistent development processes across the team
- Clear ownership and accountability for work items
- Decisions documented so new hires understand context
- Metrics to show engineering health to leadership

**Frustrations:**
- Every developer has a different workflow
- Decisions made in Slack disappear into history
- No visibility into engineering throughput

**Process needs:** `software` or `full-product`
**Typical project:** 200-500 work items, ongoing
**Agent usage:** Multiple agents — one per developer, plus CI/review agents

---

### Persona 4: Sam Rodriguez — Consultant / Contractor

**Profile:**
- Independent data engineering consultant
- Takes 4-6 short engagements per year (2-8 weeks each)
- Needs fast spin-up, intensive work, clean handoff to client
- Context and decisions must be transferable
- Values reusable project templates

**Expertise:** Python, Spark, Airflow, dbt, cloud infrastructure

**Goals:**
- Zero-to-productive in minutes for each new engagement
- Clean handoff packages for clients
- Reusable project templates across engagements
- Track time and deliverables for invoicing

**Frustrations:**
- Each client has different tooling — re-setup every time
- Handoff documents are painful to create retroactively
- Losing track of what was decided vs what was just discussed

**Process needs:** `managed` or `consulting`
**Typical project:** 10-50 items, 2-8 week duration
**Agent usage:** One AI agent, intensive daily use

---

### Persona 5: kaits — The Company Simulator

**Profile:**
- Not a human — kaits is the multi-agent company simulator
- Uses aibox as infrastructure
- Creates repos, initializes projects, assigns agent teams, monitors progress
- Needs fully programmatic/scriptable access — no interactive prompts

**Capabilities:** Manages agents with any expertise via orchestration

**Goals:**
- Programmatically create and manage dozens of aibox projects
- Assign agent teams to projects with correct roles
- Monitor progress across all projects via metrics
- Apply company-level processes (SAFe, OKRs) across projects

**Requirements:**
- aibox must be fully scriptable — no interactive prompts
- Need predictable file structures for cross-project queries
- Schema changes must be migratable programmatically

**Process needs:** `full-product` per project
**Typical scale:** 50K+ items across all projects
**Agent usage:** Dozens of agents across multiple projects, using aibox skills

---

### Persona 6: Jordan Lee — Creative / Content Producer

**Profile:**
- Technical writer and content strategist at a SaaS company
- Manages documentation, blog posts, tutorial videos, marketing content
- Works with AI for drafting and editing
- Needs editorial workflow: draft → review → approval → publish
- Manages a content calendar

**Expertise:** Technical writing, Markdown, content strategy, SEO

**Goals:**
- Track content from idea through publication
- Editorial review workflow with approval gates
- Manage content calendar (schedules/cadences)
- Link content to product features and releases

**Frustrations:**
- Content ideas get lost in Slack
- No structured review/approval workflow
- Can't see what content is in flight vs published

**Process needs:** `managed` + `editorial`
**Typical project:** 100-300 content items, ongoing
**Agent usage:** AI assistant for drafting, editing, SEO optimization

---

## Part 3: Detailed Scenario Walkthroughs

### Scenario 1: Solo Dev Starts a New Project

**Persona:** Alex (Solo Developer)
**Story:** As Alex, I want to run a single command to set up a fully configured project
with work tracking and AI context, so that I can start coding without spending time on
infrastructure setup.

**Preconditions:**
- Alex has aibox installed on the host machine
- Docker is running
- No existing project directory

**Flow:**

```
$ aibox init myproject --image python --process managed
```

**Step 1: aibox creates project structure**

```
myproject/
  aibox.toml
  CLAUDE.md
  context/
    work-item/              # empty, ready for work items
    decision/               # empty, ready for decisions
    actor/
      ACTOR-alex-chen.md    # auto-created from git config
    role/
      ROLE-admin.md         # default admin role, Alex assigned
    event/
      2026-03.jsonl         # empty, ready for events
    schemas/
      state-machines/
        work-item-default.yaml
        decision-lifecycle.yaml
      categories/
        priority.yaml
        work-type.yaml
    .aibox/
      templates/v1.0.0/     # originals for future migration
  .claude/
    skills/
      event-log-management/
      workitem-management/
      decision-record-management/
      actor-profile-management/
      role-management/
      context-archiving/
      standup-context/
      session-handover/
```

**Step 2: ACTOR-alex-chen.md auto-generated**

```yaml
---
apiVersion: aibox/v1
kind: Actor
metadata:
  id: ACTOR-bold-crane
  created_at: 2026-03-28T10:00:00Z
  labels:
    type: human
spec:
  name: "Alex Chen"
  type: human
  email: "alex@example.com"
  roles: [ROLE-admin]
---

Project owner. Profile to be enriched on first session.

## Overview

(Agent will ask about expertise, preferences, and working style on first session.)
```

Note: filename is `ACTOR-alex-chen.md` (human-readable), while `metadata.id` is the
word-ID `ACTOR-bold-crane` (machine-stable). The filename is derived from `spec.name`.

**Step 3: ROLE-admin.md auto-generated**

```yaml
---
apiVersion: aibox/v1
kind: Role
metadata:
  id: ROLE-swift-oak
  created_at: 2026-03-28T10:00:00Z
spec:
  name: "Admin"
  state: active
  permissions:
    - "Can create, edit, and delete all entity types"
    - "Can modify processes, state machines, gates, and constraints"
    - "Can assign and change roles"
  restrictions: []
  escalation: "No escalation needed — this is the highest authority role"
  filled_by: [ACTOR-bold-crane]
---

Full administrative access. Assigned to the project creator by default.
```

**Step 4: SQLite index initialized**

`aibox init` creates the SQLite index database (gitignored) from the initial files.
This ensures the index exists from the very beginning — not deferred to a later
`aibox sync`. The index is empty but ready.

**Step 5: Alex enters the container**

```
$ aibox up
```

**Step 6: First agent session**

Agent reads CLAUDE.md → context/ → skills → sees Actor profile for Alex.
Agent says: *"Welcome to myproject. I see you're Alex Chen (Admin). Before we start,
can you tell me a bit about this project and your preferences? I'd like to update your
profile so I can work with you more effectively."*

Alex describes the project. Agent enriches the Actor profile and creates initial context.

**Time: ~3 minutes from `aibox init` to first productive interaction.**

**Solutions for identified issues:**
- **Auto-generated Actor needs enrichment:** Skill instructs agent to ask for preferences
  on first session if the Actor profile's overview section says "to be enriched."
- **CLAUDE.md template includes:** "At session start, check active schedules and recent
  handover documents" — this is baked into the scaffolded template.
- **Managed package has no code processes:** Correct — that's intentional. Alex upgrades
  to `software` if they want code-review, release, etc.

---

### Scenario 2: User Has an Idea for a Feature

**Persona:** Alex (Solo Developer)
**Story:** As Alex, I want to tell my agent about a feature idea and have it properly
tracked, so that I don't lose ideas and can prioritize them later.

**Preconditions:**
- Project running with "managed" package
- Alex is in an active session

**Flow:**

Alex says: *"I have an idea — we should add dark mode toggle to the settings page"*

**Agent's decision process:**

1. Agent hears "I have an idea" — this could mean:
   a) Track it as a work item (most common)
   b) Start a discussion about it (if the idea needs exploration)
   c) Just note it informally

2. Agent uses `workitem-management` skill, but first asks:
   *"Dark mode toggle — sounds good. Want me to track this as a backlog item, or would
   you like to discuss the approach first?"*

3. **Path A — Track as work item** (Alex says "just track it"):
   - Agent runs `aibox id generate --kind WorkItem` → gets word-ID `calm-lark`
   - Agent creates `context/work-item/BACK-calm-lark-dark-mode-toggle.md`:

```yaml
---
apiVersion: aibox/v1
kind: WorkItem
metadata:
  id: BACK-calm-lark
  created_at: 2026-03-28T10:30:00Z
  updated_at: 2026-03-28T10:30:00Z
  labels:
    priority: medium
    type: feature
spec:
  title: "Add dark mode toggle to settings page"
  state: draft
  owner: ACTOR-bold-crane
  scope: null
  refs: []
---

Add a dark mode toggle to the settings page allowing users to switch between
light and dark themes.

## Overview

- Toggle in settings page UI
- Persists preference in local storage
- Respects system preference as default

## Details

(To be fleshed out when work begins.)
```

   - Agent appends event to `context/event/2026-03.jsonl`
   - Agent responds: *"Created BACK-calm-lark — 'Dark mode toggle'. Draft, medium
     priority. Want me to change the priority or add more details?"*

4. **Path B — Discuss first** (Alex says "let's think about it"):
   - Agent uses `discussion-management` skill → creates discussion entity
   - Explores: what does dark mode mean for the app? CSS variables? System preference?
     Specific color palette?
   - When discussion concludes → creates work item from the conclusions

**Vocabulary mapping (all trigger workitem-management):**
- "Add to backlog" / "create a ticket" / "file a bug" → create work item directly
- "I have an idea" / "maybe we should" / "what if" → agent asks: track or discuss?
- "We need to fix X" / "this is broken" → create bug, priority high
- "Let's plan X" → could be work item or discussion depending on scope

**Default priority inference:**
- "Idea" / "maybe" / "nice to have" → `priority: low`, `state: draft`
- Feature request (no urgency) → `priority: medium`, `state: draft`
- "We need" / "we should" (implies commitment) → `priority: medium`, `state: ready`
- "Bug" / "broken" / "fix" → `priority: high`, `state: ready`
- "Critical" / "urgent" / "production down" → `priority: critical`, `state: ready`
- Agent always allows override: *"I set this to medium — want to change it?"*

---

### Scenario 3: User Asks "What Am I Working On?"

**Persona:** Alex (Solo Developer)
**Story:** As Alex, I want to ask my agent for a status overview and get an accurate
picture, so I can decide what to focus on today.

**Preconditions:**
- Project has 15 work items in various states
- SQLite index exists (created at init, updated by aibox sync)

**Flow:**

Alex says: *"What am I working on? What's the status?"*

**Agent's process:**

1. Agent uses `workitem-management` skill → "Query" operation
2. **Primary path: query the SQLite index** (fastest, preferred)
   - Agent suggests or runs `aibox sync` if index might be stale
   - Queries: `SELECT id, title, state, priority, owner FROM work_items WHERE state NOT IN ('done', 'cancelled') ORDER BY priority, state`
3. **Fallback: scan files directly** (if index unavailable)
   - Reads `context/work-item/` directory
   - For each file, reads ONLY the YAML frontmatter (fast)
   - Groups by state, sorts by priority

4. Agent presents:
```
In Progress (3):
  BACK-bold-fox   — Fix login timeout          [critical]
  BACK-swift-oak  — API auth refactor          [high]
  BACK-calm-lark  — Dark mode toggle           [medium]

Ready (5):
  BACK-keen-jay   — DB migration               [high, blocked by BACK-swift-oak]
  BACK-warm-dove  — Email notifications        [medium]
  ...

Blocked (1):
  BACK-keen-jay   — DB migration               [blocked by BACK-swift-oak]

Draft (6): (ideas not yet committed to)
  ...
```

5. Agent also checks recent events:
   *"Since your last session 2 days ago: you completed BACK-brave-lynx (CI pipeline setup)
   and BACK-bold-fox had 3 events logged."*

6. Agent suggests: *"BACK-bold-fox is critical — want to focus there? Also, BACK-keen-jay
   is blocked by the auth refactor."*

**Scaling behavior:**

| Items | Strategy | Performance |
|---|---|---|
| 1-50 | File scan fine, index also works | Instant |
| 50-200 | Index preferred, file scan acceptable | <1 sec |
| 200-1000 | Index required | <2 sec |
| 1000+ | Index required, `aibox sync` as prerequisite | 2-10 sec sync |

**INDEX.md design (structural, not statistical):**

INDEX.md describes the directory's PURPOSE and SCHEMA, not its current state:

```markdown
# Work Items

This directory contains work item entity files (kind: WorkItem).

## Structure
- Files follow pattern: `BACK-<word-id>-<content-slug>.md`
- Sharding: by year/month when configured (e.g., `2026/03/`)
- Frontmatter: apiVersion aibox/v1, kind WorkItem

## Subtypes
- `task` — general work to be done
- `bug` — defect to fix
- `feature` — new capability
- `story` — user-facing requirement
- `chore` — maintenance / housekeeping

## State Machine
See `context/schemas/state-machines/work-item-default.yaml`

## Related Skills
- `workitem-management` — create, update, query work items
- `event-log-management` — log all changes
```

This INDEX.md is:
- **Stable** — it describes structure, not counts or states (those change constantly)
- **Useful for agents** — an agent can read this to understand what the directory contains
  without scanning files
- **Easy to maintain** — only changes when the schema changes, not when items are added
- **Auto-generated by `aibox init`** and updated by `aibox migrate` on schema changes

**What INDEX.md does NOT contain:**
- Number of items (changes constantly)
- Items grouped by state (that's a query, not a directory description)
- Recent activity (that's the event log)
- Statistics (that's the SQLite index + metrics)

---

### Scenario 4: Work Item Lifecycle Across Multiple Sessions

**Persona:** Alex (Solo Developer)
**Story:** As Alex, I want my work items to maintain continuity across sessions, so that
context is never lost when I resume work.

**Preconditions:**
- BACK-swift-oak is in `ready` state, assigned to Alex
- Alex ends session (Agent A does handover)
- Alex starts new session next day (Agent B picks up)

**Session 1 — Agent A works for 2 hours:**

1. Alex says: *"Let's work on BACK-swift-oak (API auth refactor)"*
2. Agent A reads `context/work-item/BACK-swift-oak-api-auth-refactor.md`
3. Agent A transitions state: `ready` → `in-progress`, logs state-change event
4. During work:
   - Agent discovers dependency on BACK-keen-jay → logs event, adds cross-reference
   - Alex and agent discuss JWT vs sessions → agent creates DEC-keen-fox
   - Agent implements token generation → logs progress events
5. Open threads emerge during work:
   - "Token expiry duration not decided (15min? 1hr?)"
   - "Need to update API docs for new auth flow"

**Where open threads are stored:**
Open threads are tracked as work items with `state: draft` and a ref back to the parent:

```yaml
# BACK-warm-jay-token-expiry-decision.md
kind: WorkItem
spec:
  title: "Decide token expiry duration"
  state: draft
  subtype: task
  refs:
    - type: relates-to
      target: BACK-swift-oak
```

Alternatively, if the thread is small, it's noted in the parent work item's body under
a "## Open Questions" section. The skill instructs: *"For significant open threads,
create a sub-item. For small questions, note them in the parent's Open Questions section."*

6. **Alex says "let's wrap up"**

Agent A uses `session-handover` skill. The handover is created based on:
- Events logged THIS session (agent reads the event log for today's entries)
- The skill's instructions: "Summarize what was done, list open threads, describe next
  steps. Reference all entity IDs touched."

Handover document: `context/handover/20260328-1600-session-handover.md`

```yaml
---
apiVersion: aibox/v1
kind: Artifact
metadata:
  id: ART-brave-wren
  created_at: 2026-03-28T16:00:00Z
  labels:
    type: session-handover
spec:
  title: "Session handover 2026-03-28 14:00-16:00"
---

Session handover for the next agent.

## What Was Done
- Started BACK-swift-oak (API auth refactor) — transitioned to in-progress
- Discovered dependency: BACK-keen-jay (DB migration) blocked by this
- Decision DEC-keen-fox: Use JWT with refresh tokens (not session cookies)
- Implemented: token generation endpoint (/auth/token and /auth/refresh)
- Created sub-item: BACK-warm-jay (decide token expiry duration)

## Open Threads
- Token expiry duration: 15min vs 1hr (tracked as BACK-warm-jay)
- API docs need updating for new auth flow (not yet tracked)

## Next Steps
- Implement token validation middleware
- Resolve BACK-warm-jay (token expiry)
- Once auth complete, unblock BACK-keen-jay (DB migration)

## Entities Touched
- BACK-swift-oak (state: in-progress)
- DEC-keen-fox (created, accepted)
- BACK-warm-jay (created, draft)
- 7 events logged
```

**Session 2 — Agent B (next day):**

1. Agent B starts, reads CLAUDE.md which includes:
   *"At session start: check context/handover/ for the most recent handover document.
   Also check active schedules."*
2. Agent B reads `context/handover/20260328-1600-session-handover.md` (most recent by
   date prefix in filename)
3. Agent B reads BACK-swift-oak entity + recent events for deeper context
4. Agent B greets:
   *"Welcome back. Yesterday you worked on BACK-swift-oak (API auth refactor). Key
   progress: JWT token generation implemented (DEC-keen-fox). Open question: token expiry
   duration (BACK-warm-jay). Next: implement validation middleware. Ready to continue?"*
5. Alex confirms, work continues seamlessly.

**What if there are many handover documents?**
- Agent reads ONLY the most recent (sorted by date prefix: `20260328-1600-...`)
- If the gap is >7 days, agent says: *"It's been 8 days since the last session. There
  are 3 handover documents. Want me to summarize all of them, or just the most recent?"*
- Agent prompts the user rather than guessing

**AI-provider independence:**
The handover mechanism is entirely file-based — it works with any LLM/agent provider.
CLAUDE.md instructs "read handover at session start," which any agent can follow.
The event log is JSONL (universal format). No provider-specific hooks required.

---

### Scenario 5: Two Humans Collaborate with RBAC

**Persona:** Maria (Team Lead, Admin) + Bob (Developer)
**Story:** As Maria, I want my team's agents to respect role-based access control, so
that developers can't accidentally change release processes.

**Preconditions:**
- Project with "software" package
- Maria has Admin role; Bob has Developer role
- Release process defined as `PROC-release.md`

**Human identity resolution:**

The agent needs to know WHO is currently speaking. Identity is resolved in this order:

1. **Environment variable** (most explicit): `AIBOX_USER=bob@company.com`
2. **AI provider identity** (if available):
   - Claude Code: reads from Anthropic account authentication
   - GitHub Copilot: reads GitHub user identity
   - Self-hosted LLMs: no identity — falls back to other methods
3. **Git config**: `git config user.email` → matches against Actor entities
4. **Explicit command**: `aibox whoami` sets identity for the session
5. **Agent asks**: If no identity resolved, agent asks: *"Who am I working with today?"*

The resolved identity maps to an Actor entity via the `spec.email` field.

**Multi-human repos:**

Each human has their OWN Actor file in the shared repo. Actor files contain:
- Name, expertise, communication preferences (non-sensitive)
- Role assignments

Actor files do NOT contain:
- Passwords, tokens, or secrets
- Private personal information (address, phone)
- Anything covered by data protection regulations

For sensitive personal preferences, each human can have a local-only file:
`~/.aibox/personal-profile.yaml` (never committed to git). The agent reads this
in addition to the shared Actor file.

**Forking vs shared repo:**
- Small teams (2-5): shared repo, everyone commits to main or feature branches.
  Actor files for all team members are in the repo. RBAC is probabilistic — the
  agent checks permissions but doesn't mechanically enforce.
- Larger teams: fork + PR workflow. Each developer's fork has their own actor file
  as the primary. PRs bring changes into the main repo.

**Flow — Bob tries to modify a process:**

1. Bob says: *"Change the release process to skip the staging deploy"*
2. Agent resolves Bob's identity → finds `ACTOR-bob-smith.md` → sees role: Developer
3. Agent reads `ROLE-developer.md`:
   ```
   restrictions:
     - "Cannot modify kind: Process, StateMachine, Gate, Constraint, Role"
   escalation: "Ask Maria Torres (Admin) for changes outside your permissions"
   ```
4. Agent responds: *"I can't modify the release process — your Developer role restricts
   process changes. Maria Torres has Admin permissions for this. Want me to create a
   work item requesting this change?"*
5. Agent logs escalation event

**Flow — Maria modifies the process:**

1. Maria says: *"Remove the staging deploy step from the release process"*
2. Agent resolves Maria → Admin role → no restrictions on Process
3. Agent edits `PROC-release.md`, removes staging step
4. Logs modification event
5. Confirms: *"Updated release process. Removed staging deploy step."*

**What if Bob says "Maria approved this verbally"?**
Agent responds: *"I understand Maria may have approved this, but I need to follow the
role restrictions. Options: (1) I create a work item for Maria to action, (2) Maria
tells her agent directly, (3) Maria temporarily assigns you a Process Editor role.
Which would you prefer?"*

The agent is firm but offers pathways — never a dead end.

**What if Maria leaves the company?**

The `role-management` skill includes instructions for role hygiene:
- *"When checking escalation paths, verify the escalation target's Actor is still active.
  If the Actor's state is `inactive` or missing, warn and suggest the user assign a new
  holder for the role."*
- This is probabilistic — the agent checks when it encounters the escalation path, not
  proactively. But `aibox lint` can also flag: "ROLE-admin has filled_by reference to
  ACTOR-maria-torres whose state is inactive."

**Scenario: Maria leaves, Bob encounters the escalation:**
1. Bob asks to change a process → agent says "escalate to Maria Torres"
2. Agent checks Maria's Actor file → sees `state: inactive`
3. Agent says: *"Maria Torres is marked as inactive. The Admin role needs a new holder.
   Please ask your team to assign a new Admin, or contact the project owner."*

---

### Scenario 6: Design Discussion Leading to a Decision

**Persona:** Alex (Solo Developer)
**Story:** As Alex, I want a structured discussion about a design choice and the decision
recorded, so that future me understands why I chose PostgreSQL.

**Flow:**

1. Alex: *"Let's discuss whether to use PostgreSQL or SQLite for this project"*

2. Agent recognizes this warrants a discussion (multiple alternatives, long-term
   implications). Creates `context/discussion/DISC-bright-owl-database-choice.md`:

```yaml
---
apiVersion: aibox/v1
kind: Discussion
metadata:
  id: DISC-bright-owl
  created_at: 2026-03-28
  labels:
    topic: architecture
spec:
  title: "Database choice: PostgreSQL vs SQLite"
  state: active
  participants: [ACTOR-bold-crane]
  refs: []
---

Discussion on which database to use for the primary data store.

## Overview

Evaluating PostgreSQL and SQLite as primary database for myproject.
Key criteria: scalability, deployment simplicity, team expertise.

## Discussion

### §2.1 PostgreSQL — Pros and Cons

**Pros:** Full SQL support, concurrent connections, JSONB, extensions...
**Cons:** Requires separate server, more operational overhead...

### §2.2 SQLite — Pros and Cons

**Pros:** Zero deployment, file-based, embedded...
**Cons:** No concurrent writes, limited to single-machine...

### §2.3 Criteria That Matter for This Project

- We need concurrent API requests → favors PostgreSQL
- Deployment target is a single VPS → SQLite could work
- Alex has strong PostgreSQL experience → favors PostgreSQL
```

3. Back and forth — agent appends §2.4, §2.5 as new arguments emerge

4. Alex: *"OK, let's go with PostgreSQL"*

5. Agent creates `context/decision/DEC-keen-fox-use-postgresql.md`:

```yaml
---
apiVersion: aibox/v1
kind: Decision
metadata:
  id: DEC-keen-fox
  created_at: 2026-03-28
  labels: {}
spec:
  title: "Use PostgreSQL for primary database"
  state: accepted
  date: 2026-03-28
  decision_makers: [ACTOR-bold-crane]
  superseded_by: null
  refs:
    - type: justified-by
      target: DISC-bright-owl
---

Use PostgreSQL as the primary database for myproject.

## Context

Need a database for API data storage. Concurrent request handling required.

## Decision

PostgreSQL. Provides concurrent access, JSONB for flexible schemas, and aligns
with existing team expertise.

## Alternatives

**SQLite** — rejected. Cannot handle concurrent writes needed for API use case.

## Consequences

- Requires PostgreSQL server in deployment (Docker Compose or managed service)
- Migrations needed (use Alembic)
- Additional operational monitoring
```

6. Agent updates discussion: `state: concluded`, adds ref to DEC-keen-fox
7. Logs creation and decision events

**When NOT to create a discussion:**
If Alex says *"let's use PostgreSQL"* directly (no deliberation), the agent skips the
discussion entity and creates only the Decision Record. The skill instructs: *"Create a
formal discussion only when the user wants to explore alternatives, when there's
genuine uncertainty, or when the reasoning process itself needs to be preserved."*

---

### Scenario 7: Setting Up a Weekly Review Cadence

**Persona:** Dr. Priya (Research Scientist)
**Story:** As Priya, I want a weekly review cadence where my agent reminds me to review
lab progress, so experiments and papers stay on track.

**Flow:**

1. Priya: *"I want a weekly review every Friday afternoon"*

2. Agent creates `context/schedule/SCHED-weekly-lab-review.md`:

```yaml
---
apiVersion: aibox/v1
kind: Schedule
metadata:
  id: SCHED-warm-lark
  created_at: 2026-03-28
spec:
  title: "Weekly Lab Review"
  subtype: cadence
  state: active
  pattern: "weekly(friday)"
  duration: "1h"
  last_completed: null
---

Weekly review of lab progress across all research streams.

## Overview

Every Friday afternoon, review progress on experiments, papers, and grant milestones.

## Agenda Template

1. Review each active research stream (experiments completed, blocked, planned)
2. Paper pipeline status (drafts, submissions, reviews pending)
3. Grant milestone check (upcoming deadlines, deliverables)
4. Action items for next week
5. Any blockers or resource needs
```

3. Logs creation event

**Following Friday — session starts:**

4. Agent reads CLAUDE.md → follows instruction: "At session start, check active schedules"
5. Agent reads all Schedule entities → finds SCHED-warm-lark:
   - Pattern: weekly(friday)
   - last_completed: null (never done)
   - Today is Friday → due!
6. Agent: *"Good afternoon. Your weekly lab review is due — you haven't done one yet.
   Want me to prepare a summary of this week's progress?"*

**Provider-native scheduling integration:**

Some AI providers offer native scheduling:
- **Claude Code:** Has remote agents / triggers that can run on a cron schedule
- **Other providers:** May have similar features

When available, aibox can optionally hook into the provider's scheduling:
```toml
[scheduling]
provider_native = true   # use provider's cron if available
fallback = "session-check"  # always check schedules at session start too
```

If `provider_native = true` and Claude Code is the provider, aibox configures a remote
trigger that starts a session on Friday and runs the weekly review. If the provider
doesn't support native scheduling, the fallback is the session-start check.

**This is opt-in and provider-aware** — the core mechanism (session-start schedule check)
is always active and provider-independent. Native scheduling is a bonus.

**What if Priya misses Friday?**
- Saturday: *"Your weekly review is 1 day overdue."*
- Monday: *"Your weekly review is 3 days overdue. Want to do it now?"*
- After 2 weeks: *"You've missed 2 weekly reviews. Want to catch up or skip?"*

---

### Scenario 8: aibox Schema Migration (v1 → v2)

**Persona:** Alex (Solo Developer)
**Story:** As Alex, I want aibox updates to be safe and guided, so my context isn't
broken by schema changes.

**Flow:**

1. Alex runs `aibox update` → CLI updates to v2
2. aibox stores new templates in `context/.aibox/templates/v2.0.0/`
3. aibox generates `context/.aibox/migration/v1-to-v2.md`:

```markdown
# Migration: aibox/v1 → aibox/v2

## Changes
- WorkItem: `metadata.labels.priority` is now required (was in spec.priority)
- Decision: added optional `spec.consequences` field
- Event schema: added optional `correlation_id` field
- State machine: `blocked` state renamed to `waiting` in work-item-default

## Instructions for your agent
1. For all WorkItem files: move `spec.priority` to `metadata.labels.priority`
2. Update `apiVersion: aibox/v1` to `apiVersion: aibox/v2` in all entity files
3. In state machine files: rename `blocked` to `waiting`
4. For work items in state `blocked`: update to `waiting`
5. Never edit files without asking the user first
6. After migration, ask the user to run `aibox lint` to verify
```

4. `aibox lint` detects pre-migration state and warns:
   *"23 files have apiVersion: aibox/v1 but CLI is v2. Migration needed."*

5. Next session — agent reads the migration document:
   *"aibox updated to v2. There's a migration for 23 work items: move priority to labels,
   rename 'blocked' to 'waiting'. Shall I proceed? I'll do it file by file."*

6. Alex approves. Agent migrates, logging each change as an event.
7. Alex runs `aibox lint` → all clear.

**For large projects (5000 files):** Agent migrates in batches of 50, reporting progress:
*"Migrated 50/5000 files (1%). Continuing..."* Git provides safety — `git checkout .`
reverts everything if needed.

---

### Scenario 9: Project Grows to 5,000 Work Items

**Persona:** Maria (Team Lead)
**Story:** As Maria, I want the system to remain performant at scale, so my team isn't
slowed down.

**Project state:** 18 months running. 5,000 work items (3,500 done, 800 archived,
700 active). 50,000+ events across 18 months.

**Directory structure with monthly sharding:**
```
context/work-item/
  2024/09/ (15 files)
  2024/10/ (22 files)
  ...
  2026/03/ (52 files — current month, most active)
```

**Agent queries status:**

1. Agent ALWAYS uses the SQLite index for projects of this size
2. Agent runs or suggests `aibox sync` if index is stale
3. Queries the index: active items, grouped by state and priority
4. Presents the same format as Scenario 3 but sourced from SQL

**Maria asks: "How's our lead time trending?"**

1. Agent uses `metrics-management` skill
2. Reads MET-swift-tern (Lead Time) definition
3. Queries SQLite index (NOT the event files directly):
   ```sql
   SELECT month, AVG(done_at - ready_at) as avg_lead_time
   FROM work_item_events
   WHERE type = 'state-change'
   GROUP BY month
   ORDER BY month
   ```
4. Presents trending data, compares against threshold

**INDEX.md at this scale:** Contains structural information only (see Scenario 3).
Statistics come from the SQLite index. INDEX.md describes the directory's purpose,
schema, subtypes, and related skills — not counts or states.

**Archiving:** Agent periodically suggests: *"You have 2,100 done work items older
than 90 days. Want me to archive them to reduce the active file count?"*

---

### Scenario 10: kaits Spins Up a Simulated Company Project

**Persona:** kaits (Company Simulator)
**Story:** As kaits, I want to programmatically create an aibox project with a full team
structure, so I can simulate a development team.

**Key principle:** kaits orchestrates agents. Those agents use aibox skills to interact
with the context system. kaits doesn't write entity files directly — kaits agents do,
using the same skills any human's agent would use.

**Flow:**

1. kaits runs:
```bash
aibox init simco-webapp --image node --process full-product --non-interactive
```

2. aibox creates the project with all primitives active, SQLite index initialized.

3. kaits assigns agents to the project. Each agent has its own Actor entity.
   The agents use `actor-profile-management` to create their profiles:

```
ACTOR-senior-dev.md      (Senior Developer Agent, model: claude-sonnet)
ACTOR-junior-dev.md      (Junior Developer Agent, model: claude-haiku)
ACTOR-qa-engineer.md     (QA Agent, model: claude-sonnet)
ACTOR-product-owner.md   (PO Agent, model: claude-opus)
ACTOR-tech-lead.md       (Tech Lead Agent, model: claude-opus)
```

4. kaits' orchestration agent uses `role-management` to create roles and assignments:
```
ROLE-developer.md         filled_by: [senior-dev, junior-dev]
ROLE-qa.md                filled_by: [qa-engineer]
ROLE-product-owner.md     filled_by: [product-owner]
ROLE-tech-lead.md         filled_by: [tech-lead]
```

5. kaits' PO agent uses `scope-management` to create the project scope:
```
SCOPE-v1-launch.md        "Simulated Webapp v1.0 Launch"
```

6. kaits' PO agent uses `workitem-management` to create 20 initial work items from
   the product brief. Each is created via the skill, properly formatted, events logged.

7. kaits' tech lead uses `process-management` to review and customize process definitions
   (code-review, release) for this team's conventions.

8. kaits' orchestration agent uses `schedule-management` to set up cadences:
```
SCHED-daily-standup.md     pattern: "daily(weekdays)"
SCHED-weekly-planning.md   pattern: "weekly(monday)"
SCHED-biweekly-retro.md    pattern: "biweekly(friday)"
```

9. Agents begin working — senior-dev picks up a work item, writes code, submits for
   review. QA agent tests. PO agent reviews. All using skills, all logging events.

**kaits monitors progress by:**
- Reading event log files across project repos (JSONL is easy to parse)
- Running `aibox sync` per project to maintain indexes
- Querying SQLite indexes for metrics (lead time, throughput, WIP)
- Aggregating cross-project data in kaits' own database

**CLI requirements for kaits:**
- `--non-interactive` flag on all commands (no prompts)
- `aibox id generate --kind WorkItem --count 20` (batch ID generation)
- `aibox sync --quiet` (machine-parseable output)
- `aibox lint --format json` (structured validation results)

---

## Part 4: Cross-Scenario Summary

### Issues Found and Resolutions

| # | Issue | Severity | Resolution |
|---|---|---|---|
| 1 | Naming inconsistency (gate/checkpoint, scope/project) | High | Standardized: Gate everywhere, SCOPE- for all scopes |
| 2 | Human-readable filenames for actors/roles | High | Pattern A (human-named) for low-volume, Pattern B (word-id+slug) for high-volume |
| 3 | Schedule checking at session start | High | Baked into CLAUDE.md template: "check schedules and recent handover" |
| 4 | Research package missing schedule-management | High | Added to research package |
| 5 | SQLite index should exist from init | High | `aibox init` creates empty index; `aibox sync` keeps it current |
| 6 | INDEX.md should be structural, not statistical | High | INDEX.md describes purpose/schema/subtypes; statistics from SQLite |
| 7 | Human identity resolution | High | 5-level cascade: env var → provider → git config → aibox whoami → ask |
| 8 | Open threads storage across sessions | Medium | Significant threads → sub-work-items; small → parent's Open Questions section |
| 9 | Agent asks "track or discuss?" for ambiguous input | Medium | Skill instruction: "if user says 'idea/maybe', ask whether to track or discuss" |
| 10 | Stale role assignments (Maria leaves) | Medium | role-management skill checks Actor state; aibox lint flags inactive references |
| 11 | Provider-native scheduling integration | Medium | Optional [scheduling] in aibox.toml; session-start check is always the fallback |
| 12 | Multi-human repo privacy | Medium | Actor files contain non-sensitive info; private preferences in ~/.aibox/ |
| 13 | Handover with many documents | Low | Agent reads most recent by date; prompts user if gap >7 days |
| 14 | Large-scale migration (5000 files) | Low | Batch processing with progress reporting; git provides rollback |

### Primitives Exercised Per Scenario

| Primitive | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 |
|---|---|---|---|---|---|---|---|---|---|---|
| Work Item | x | x | x | x | x | | | x | x | x |
| Event | x | x | x | x | x | x | x | x | x | x |
| Decision | | | | x | | x | | | | x |
| Artifact | | | | x | | | | | | |
| Actor | x | | | | x | | x | | | x |
| Role | x | | | | x | | | | | x |
| Process | | | | | x | | | | | x |
| State Machine | | x | x | x | | | | x | | |
| Category | | x | x | | | | | | | x |
| Cross-Reference | | | | x | | x | | | | x |
| Gate | | | | | x | | | | | x |
| Metric | | | | | | | | | x | x |
| Schedule | | | | | | | x | | | x |
| Scope | | | | | | | | | | x |
| Constraint | | | | | x | | | | | |
| Context | x | | | x | | | | x | | |
| Discussion | | x | | | | x | | | | |
