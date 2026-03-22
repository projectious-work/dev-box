# Process Skills

Skills for managing project workflows, team coordination, and operational processes.

---

### backlog-context

> Manages project backlog as a BACKLOG.md file in the context directory. Creates, prioritizes, and tracks work items in Markdown format.

**Triggers:** When the user asks to add, update, or review backlog items.
**Tools:** None
**References:** None

Key capabilities:

- Read and update `context/BACKLOG.md`
- Work items in checkbox format: `- [ ] **Title** -- Description`
- Group items by priority: Next Up, Planned, Ideas
- Reference GitHub issues where they exist (`#N`)
- Mark completed items with `[x]`

??? example "Example usage"
    User asks to add a backlog item. The agent reads `context/BACKLOG.md`, adds the new item under the appropriate priority group with checkbox format and a GitHub issue reference if one exists.

---

### decisions-adr

> Manages architectural decision records in context/DECISIONS.md. Records decisions with rationale, alternatives considered, and implications.

**Triggers:** When the user makes a significant technical or process decision that should be recorded.
**Tools:** None
**References:** None

Key capabilities:

- Read and update `context/DECISIONS.md`
- Assign next `DEC-NNN` number in inverse chronological order
- Each entry includes decision, rationale, and alternatives considered
- Date added in parentheses after the title
- New decisions placed at the top, below the header

??? example "Example usage"
    User decides to use SQLite instead of PostgreSQL for the local dev environment. The agent assigns the next DEC number, records the decision with rationale and alternatives, and places it at the top of DECISIONS.md.

---

### standup-context

> Manages session standup notes in context/STANDUPS.md. Records what was done, what is planned, and any blockers at the start or end of work sessions.

**Triggers:** At the start of a new session, or when the user asks to record progress.
**Tools:** None
**References:** None

Key capabilities:

- Read and update `context/STANDUPS.md`
- Add entries with today's date as heading
- Each entry includes Done, Next, and Blockers sections
- New entries placed at the top, below the header
- Keep entries concise -- one line per item

??? example "Example usage"
    At the start of a session, the agent reads STANDUPS.md, creates a new entry with today's date, and fills in what was completed last session, what is planned now, and any blockers.

---

### release-semver

> Manages semantic versioning releases -- version bumps, changelogs, tags, publishing. Use when preparing a new release.

**Triggers:** When the user says "prepare a release", "bump the version", "publish", "cut a release", or asks about versioning.
**Tools:** None
**References:** None

Key capabilities:

- Determine version bump type (patch, minor, major) based on changes
- Pre-release checklist: tests pass, no uncommitted changes, dependencies up to date
- Prepare changelog grouped by Added, Changed, Fixed, Removed
- Bump version in all relevant files (Cargo.toml, pyproject.toml, package.json, etc.)
- Commit with `chore: bump version to vX.Y.Z` and tag `vX.Y.Z`
- Publish according to the project's distribution channel

??? example "Example usage"
    User says "Let's release -- we fixed two bugs and added a feature." The agent recommends a minor bump (new feature), drafts changelog entries, updates version files, and creates the tagged commit.

---

### incident-response

> Guides production incident handling -- triage, communicate, fix, postmortem. Use when something is broken in production.

**Triggers:** When the user reports a production issue, outage, or says "production is down", "users are affected", "we have an incident".
**Tools:** None
**References:** None

Key capabilities:

- Triage within first 5 minutes: assess user impact, identify recent changes, evaluate rollback options
- Communicate to stakeholders with status, impact, and ETA
- Mitigate first, fix later: rollback or temporary workaround to restore service
- Fix: identify root cause, apply and test the fix, deploy with extra monitoring
- Postmortem within 48 hours: timeline, root cause analysis, action items, blameless approach

??? example "Example usage"
    User reports "Our API is returning 500 errors after the last deploy." The agent checks the deploy diff, identifies the breaking change, recommends immediate rollback while preparing a fix, and drafts a stakeholder update.

---

### retrospective

> Facilitates team or project retrospectives -- what worked, what didn't, action items. Use at the end of a sprint, milestone, or project phase.

**Triggers:** When the user says "let's do a retro", "what went well?", "lessons learned", or "end of sprint review".
**Tools:** None
**References:** None

Key capabilities:

- Set the scope of the retrospective (time period or milestone)
- Gather input in three categories: What Worked, What Didn't Work, What to Try Next
- Format as a structured Markdown document
- Action items must be specific, assignable, and time-bound
- Store the retrospective in `context/` or the project's designated location

??? example "Example usage"
    User says "Let's do a retro on the v0.3 release." The agent asks what went well and what was painful, then structures findings into the retro format with concrete action items and saves to `context/retros/v0.3.md`.

---

### agent-management

> Multi-agent workflow coordination including task delegation, context sharing, handoff protocols, and quality gates. Use when orchestrating multiple AI agents, designing agent pipelines, or managing agent-to-agent communication.

**Triggers:** When coordinating multiple AI agents on a shared task, decomposing large tasks into agent-assignable subtasks, designing agent pipelines, or setting up handoff protocols.
**Tools:** None
**References:** `references/coordination-patterns.md`

Key capabilities:

- Task decomposition into agent-sized units with clear scope, input/output specs, and success criteria
- Agent role definitions: Planner, Researcher, Coder, Reviewer
- Context sharing via file-based handoffs, structured messages, and shared memory (project files)
- Structured handoff protocol with status, artifacts, verification, blockers, and next action
- Quality gates between stages (planning, research, coding, review)
- Error recovery and fallback strategies for failed agent steps
- Progress tracking via checklists updated after each step

??? example "Example usage"
    User asks to add authentication to an API using multiple agents. The agent creates a task plan: (1) Researcher reads existing route handlers and middleware patterns, writes findings to a scratch file. (2) Coder implements auth middleware and JWT validation based on findings. (3) Reviewer checks against OWASP guidelines and verifies tests pass. Each handoff includes structured status, artifacts, and next action.

---

### estimation-planning

> Software estimation and planning including story points, velocity tracking, scope negotiation, and technical debt budgeting. Use when estimating work, planning sprints, or negotiating project scope.

**Triggers:** When estimating work effort, planning sprints, negotiating scope, budgeting for technical debt, or asking "how should I estimate this project?".
**Tools:** None
**References:** None

Key capabilities:

- Story points vs time estimates: Fibonacci sizing, relative complexity, when to use each
- Planning poker with anchoring bias prevention and async estimation for remote teams
- Cone of uncertainty: communicate estimates as ranges with confidence levels
- Velocity tracking over 5-6 sprints for reliable forecasting
- Scope negotiation using MoSCoW prioritization and MVP identification
- Technical debt budgeting: allocate 15-20% of each sprint, categorize as critical/important/minor
- Three-point estimation with PERT formula and standard deviation
- Monte Carlo simulation basics for probabilistic completion forecasts

??? example "Example usage"
    Stakeholder wants all 50 stories done by end of quarter. The agent calculates based on average velocity and remaining sprints, finds the scope exceeds capacity, and presents options: descope to must-haves, extend the deadline, or add capacity. Uses MoSCoW to identify which stories are must-have vs could-have.

---

### postmortem-writing

> Blameless postmortem writing with timeline, root cause analysis, and corrective actions. Use when writing incident postmortems, conducting post-incident reviews, or creating postmortem templates.

**Triggers:** When writing an incident postmortem, conducting a post-incident review, creating a postmortem template, or asking "how do I write a blameless postmortem?".
**Tools:** None
**References:** None

Key capabilities:

- Structured postmortem template: title/metadata, summary, impact, timeline, root cause, contributing factors, what went well, corrective actions, lessons learned
- Timeline writing with UTC timestamps, separating detection time from response time
- Root cause analysis using 5 Whys technique, drilling to systemic/process issues
- Corrective actions categorized as prevent, detect, or mitigate -- each with owner and due date
- Blameless culture practices: systems-focused language, passive voice for human errors
- Facilitation tips: schedule within 3-5 days, time-box to 60 minutes, separate facilitator from author

??? example "Example usage"
    User reports a production outage and needs help writing the postmortem. The agent asks for the timeline of events, collects facts, structures the postmortem using the template, guides the 5 Whys analysis from symptom to systemic root cause, and proposes corrective actions in prevent/detect/mitigate categories.
