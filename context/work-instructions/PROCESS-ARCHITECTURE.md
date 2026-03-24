# Process Definition Frameworks Research — BACK-057

**Status:** Research spike — informs process library architecture and BACK-055 implementation
**Date:** 2026-03-18
**Decision refs:** DEC-020 (company simulation), BACK-053 (lifecycle hooks), BACK-055 (process templates)
**Author:** Researcher agent

---

## 1. SAFe (Scaled Agile Framework) 6.0

### 1.1 Full Hierarchy

SAFe 6.0 organizes work delivery across four levels, each with distinct artifacts, ceremonies, and roles:

**Portfolio Level** — the strategic apex. Connects business strategy to execution through Lean Portfolio Management (LPM). Three integrated domains: Strategy and Investment Funding, Agile Portfolio Operations, and Lean Governance. Work items at this level are **Epics** — large initiatives that flow through a Portfolio Kanban board (states: Funnel → Reviewing → Analyzing → Portfolio Backlog → Implementing → Done). Each epic requires a **Lean Business Case** before approval.

**Large Solution Level** — coordinates multiple Agile Release Trains (ARTs) to deliver complex solutions. Uses a **Solution Train** to synchronize ARTs. Work items: **Capabilities** (large solution-level features that span ARTs). This level exists only when a single ART cannot deliver the full solution.

**ART (Program) Level** — the primary value delivery mechanism. An ART is a long-lived team-of-teams (50-125 people, 8-12 agile teams) aligned to a single value stream. Work items: **Features** (user-facing value) and **Enablers** (technical infrastructure that supports future features). The ART operates on a fixed cadence of **Program Increments (PIs)** — typically 8-12 weeks comprising 4-5 iterations plus an Innovation and Planning (IP) iteration.

**Team Level** — cross-functional Agile teams (5-11 people) using Scrum, Kanban, or hybrid. Work items: **Stories** (user stories and enabler stories). Teams operate in 2-week iterations within the PI cadence. Each team has a Scrum Master and Product Owner.

### 1.2 Process Artifacts at Each Level

| Level | Work Items | Key Artifacts |
|---|---|---|
| Portfolio | Epics, Enabler Epics | Lean Business Case, Portfolio Canvas, Strategic Themes |
| Large Solution | Capabilities, Enabler Capabilities | Solution Intent, Solution Context, Solution Roadmap |
| ART | Features, Enabler Features | Program Backlog, PI Objectives, Feature Acceptance Criteria |
| Team | Stories, Enabler Stories | Team Backlog, Iteration Goals, Story Acceptance Criteria |

### 1.3 PI Planning, Inspect & Adapt, Innovation Sprints

**PI Planning** (now called Planning Interval Planning) is a two-day face-to-face event at the start of each PI where all ART members align on a shared mission and plan. Outputs: PI Objectives per team, Program Board (dependency map), ROAM risk board. PI Planning defines a delivery model using teams-of-teams operating in fixed-width timeboxes.

**Inspect & Adapt (I&A)** — a full-day event at the end of each PI. Three parts:
1. **PI System Demo** — integrated demo of all features delivered
2. **Quantitative Measurement** — teams review metrics they have agreed to collect, discuss data and trends
3. **Problem-Solving Workshop** — root cause analysis (fishbone/5 whys) on the biggest impediment, producing improvement stories for the next PI

**Innovation and Planning (IP) Iteration** — the final iteration of each PI, reserved for innovation, exploration, hackathons, infrastructure work, and PI Planning prep. Not carrying committed stories.

### 1.4 Lean Portfolio Management

LPM aligns strategy and execution through three integrated domains:

- **Strategy and Investment Funding:** Establishes Strategic Themes and Outcomes that guide investment decisions. Allocates budgets to value streams rather than projects. Epics flow through the Portfolio Kanban.
- **Agile Portfolio Operations:** Coordinates across ARTs. Manages the Portfolio Kanban for visualizing and managing epic flow. The mechanism that bridges strategy to approved work is the Lean Business Case — when an epic moves from funnel through analysis, the Epic Owner develops a case articulating the hypothesis, expected business outcomes, and minimum viable approach.
- **Lean Governance:** Replaces traditional phase-gate governance with dynamic budgeting, epic-level guardrails, and continuous compliance through built-in quality.

Epic Owners focus on business Epics; Enterprise Architects guide Enabler Epics (technical enablers).

### 1.5 Built-in Quality and Definition of Done

SAFe mandates built-in quality at every level:
- **Story DoD:** Code complete, unit tested, code reviewed, meets acceptance criteria
- **Feature DoD:** All stories done, feature tested end-to-end, documentation updated, PO accepted
- **PI DoD:** All committed features done, system demo passed, I&A completed, metrics reviewed
- **Release DoD:** Regression passed, performance validated, release notes complete

### 1.6 Metrics — Flow Metrics and Business Agility

SAFe 6.0 introduces six **Flow Metrics** (replacing output-focused velocity metrics):

1. **Flow Velocity** — number of completed items in a given timeframe
2. **Flow Time** — total time from start to finish for an item
3. **Flow Efficiency** — ratio of active work time to total elapsed time
4. **Flow Load** — total number of work items in progress (WIP)
5. **Flow Distribution** — proportion of work types (features, enablers, defects, debt)
6. **Flow Predictability** — consistency between planned and actual delivery

SAFe 6.0 also introduces four new flow articles — Team Flow, ART Flow, Solution Train Flow, and Portfolio Flow — describing how to apply the eight flow accelerators from Principle #6 at each level.

**Business Agility Assessment** — SAFe's maturity model. Measures organizational competence across seven competencies: Team and Technical Agility, Agile Product Delivery, Enterprise Solution Delivery, Lean Portfolio Management, Organizational Agility, Continuous Learning Culture, and Lean-Agile Leadership. The "Measure and Grow" concept (replacing the older "Metrics" icon in SAFe 6.0) emphasizes OKRs alongside flow metrics.

### 1.7 What kaits Adopts from SAFe

| SAFe Concept | kaits Mapping | Implementation |
|---|---|---|
| PI cadence | Heartbeat cycle | Agent heartbeat already on fixed cadence (BACK-046); treat a "PI" as N heartbeat cycles |
| Flow Metrics | Event-sourced metrics | Flow Time = task created→completed from event log; Flow Load = active tasks; Flow Distribution = task tags |
| Portfolio Kanban | Project lifecycle | `ProjectDefinition` states (BACK-052) map to Kanban columns |
| ART coordination | Department-level sync | Department heads coordinate their agents (BACK-054) |
| I&A ceremony | Process Watchdog review | Watchdog agent (BACK-053) runs periodic retrospective analysis |
| Lean Business Case | Task briefing | PM agent creates task briefings with scope, context, acceptance criteria |
| Built-in quality DoD | Quality gate trigger | `triggers/quality-gate.yaml` already enforces quality checks |

---

## 2. PMBOK 7th Edition

### 2.1 Process Groups vs. Performance Domains

The PMBOK 7th Edition (2021) represents a fundamental paradigm shift. Previous editions (6th and earlier) were organized around **5 Process Groups** (Initiating, Planning, Executing, Monitoring & Controlling, Closing) and **49 processes** with prescriptive inputs/tools/outputs. The 7th Edition abandons this prescriptive approach in favor of **principles and performance domains** — recognizing that no single approach fits all projects.

The shift: from "follow these 49 processes" to "attend to these 8 areas and apply these 12 principles, tailored to your context."

Note: PMBOK 8th Edition is scheduled for release in early 2026, retaining the principles and performance domains foundation but simplifying to seven performance domains.

### 2.2 The 8 Performance Domains

The eight performance domains are interactive, interrelated, and interdependent areas of focus that work together throughout the project:

1. **Stakeholders** — identifying, understanding, and engaging the stakeholder community. Seeking good working relationships, agreement with project objectives, and support for achieving goals. Maps to: who cares about the work and how to keep them informed.

2. **Team** — creating a high-performing team that takes shared ownership. Covers team composition, leadership styles, team development, and conflict management. Maps to: how agents collaborate and who leads.

3. **Development Approach and Life Cycle** — decisions about methodology (predictive, adaptive, hybrid) and delivery cadence. Maps to: which process template to apply to a project.

4. **Planning** — ongoing and evolving organization and coordination required for successful delivery. Encompasses scope, schedule, cost, resource, quality, and communications planning. Maps to: how work is structured and sequenced.

5. **Project Work** — establishing project processes, managing physical resources, and managing procurement. The actual execution of planned work. Maps to: task execution and resource allocation.

6. **Delivery** — focused on delivering the intended value and meeting quality requirements. Maps to: ensuring outputs match acceptance criteria.

7. **Measurement** — monitoring and controlling delivery through metrics, dashboards, and earned value. Maps to: event-sourced metrics and progress tracking.

8. **Uncertainty** — risk management — identifying, analyzing, and responding to risks, ambiguity, and complexity. Maps to: how agents handle unknown situations and escalate.

The activities within domains overlap throughout the project, as opposed to being performed linearly.

### 2.3 Tailoring

PMBOK 7th Edition makes tailoring a first-class concept. The tailoring process:

1. **Select the development approach** — predictive, adaptive, or hybrid
2. **Tailor for the organization** — adapt to organizational culture, governance, and existing processes
3. **Tailor for the project** — adapt to project size, complexity, risk profile, and stakeholder needs
4. **Implement and improve** — continuously adjust throughout the project based on outcomes

Tailoring is the key concept for kaits: different project types (research spike, feature development, bug fix) need different process templates, and the PM agent should select the appropriate template based on context.

### 2.4 Earned Value Management (EVM)

EVM integrates scope, schedule, and cost to measure project performance. Core variables:

- **Planned Value (PV):** Budgeted cost of work scheduled. `PV = Budget × Planned % Complete`
- **Earned Value (EV):** Budgeted cost of work actually performed. `EV = Budget × Actual % Complete`
- **Actual Cost (AC):** Actual cost incurred for work performed.

Key indices:
- **Schedule Performance Index (SPI):** `SPI = EV / PV` — >1 means ahead of schedule, <1 behind
- **Cost Performance Index (CPI):** `CPI = EV / AC` — >1 means under budget, <1 over budget
- **Schedule Variance (SV):** `SV = EV - PV`
- **Cost Variance (CV):** `CV = EV - AC`

### 2.5 What kaits Adopts from PMBOK

| PMBOK Concept | kaits Mapping | Implementation |
|---|---|---|
| Performance domains | Process template categories | Each template covers relevant domains (not all 8 for every project) |
| Tailoring | PM agent process selection | PM agent selects process template based on task type, complexity, risk |
| EVM (adapted) | Token-cost tracking | `PV` = estimated token budget; `EV` = work completed; `AC` = actual tokens spent. SPI/CPI computed per task from event log |
| Measurement domain | Materializer metrics | Event log materializer (BACK-050) computes flow metrics and EVM analogues |
| Uncertainty domain | Escalation protocol | Risk events trigger escalation to manager or human (org chart routing) |
| Stakeholder domain | Notification system | Attention badges (BACK-039) surface important events to human stakeholders |

---

## 3. IPMA ICB4

### 3.1 Competence Model — Three Domains

The IPMA Individual Competence Baseline version 4 (ICB4) defines competence across three interconnected domains:

**Perspective (5 competence elements):**
- Strategy — understanding organizational strategy and aligning project goals
- Governance, structures and processes — organizational context for projects
- Compliance, standards and regulations — legal and regulatory requirements
- Power and interest — stakeholder influence dynamics
- Culture and values — organizational and societal context

**People (10 competence elements):**
- Self-reflection and self-management
- Personal integrity and reliability
- Personal communication
- Relations and engagement
- Leadership
- Teamwork
- Conflict and crisis
- Resourcefulness
- Negotiation
- Results orientation

**Practice (14 competence elements):**
- Project design — defining how the project will be managed
- Requirements and objectives
- Scope — what's in and what's out
- Time — scheduling and time management
- Organisation, information and documentation
- Quality
- Finance
- Resources
- Procurement
- Plan and control
- Risk and opportunity
- Stakeholders
- Change and transformation
- Select and balance (portfolio management)

ICB4 is **competence-based, not process-based** — it describes what practitioners should be able to do, not prescriptive steps to follow. This is a fundamental difference from PMBOK's historical process orientation.

### 3.2 IPMA Delta — Organizational Competence Assessment

IPMA Delta is a holistic assessment of organizational competence in managing projects, using three modules:

1. **Individual assessment (I)** — evaluates competence of selected individuals using ICB4
2. **Project assessment (P)** — evaluates selected projects/programmes using the IPMA Project Excellence Baseline (PEB)
3. **Organizational assessment (O)** — evaluates the organization's approach to managing projects using the IPMA Organisational Competence Baseline (OCB)

The OCB describes 18 organisational competence elements in 5 groups. Through the assessment, an organisation gets insights regarding the current maturity and the Delta to a desired target state.

IPMA Delta classification levels:
- **Class 1:** Initial — ad-hoc project management
- **Class 2:** Defined — some standardization exists
- **Class 3:** Standardized — consistent processes across the organization
- **Class 4:** Managed — quantitative management and continuous improvement
- **Class 5:** Optimizing — innovation-driven, best-in-class

### 3.3 IPMA Certification Levels

| IPMA Level | Title | Description | kaits Agent Level Mapping |
|---|---|---|---|
| Level D | Certified Project Management Associate | Has project management knowledge | Junior agent — follows instructions, limited autonomy |
| Level C | Certified Project Manager | Can manage projects of moderate complexity | Mid-level agent — manages tasks independently |
| Level B | Certified Senior Project Manager | Can manage complex projects/programmes | Senior agent — coordinates others, makes decisions |
| Level A | Certified Projects Director | Can manage portfolios/programmes of strategic importance | Principal agent — strategic direction, mentoring |

### 3.4 What kaits Adopts from IPMA

| IPMA Concept | kaits Mapping | Implementation |
|---|---|---|
| ICB4 three domains | Skill tree branches | Perspective/People/Practice → three skill branches per agent role (BACK-026) |
| Competence levels D→A | Agent leveling (DEC-019) | Junior→Mid→Senior→Principal maps to IPMA D→C→B→A |
| Delta assessment | Process Watchdog evaluation | Watchdog agent assesses organizational competence periodically |
| OCB elements | Department maturity metrics | Track competence per department across the 18 OCB elements |
| Competence-based (not process-based) | Agent capability model | Agents have skills (competences) that determine what tasks they can handle; processes describe the flow, competences describe the capability |

---

## 4. Process Definition Structure

### 4.1 What Makes a Good Process Document

A process document must answer seven questions:

1. **Why** — purpose, business justification, when to use this process
2. **What** — inputs required, outputs produced, deliverables
3. **Who** — roles involved, RACI matrix, approval authority
4. **How** — step-by-step instructions with decision points and quality criteria
5. **When** — trigger conditions, cadence, duration expectations
6. **Measure** — metrics for process health, KPIs, SLAs
7. **Improve** — feedback mechanism, retrospective cadence, owner responsible for updates

### 4.2 Template with YAML Frontmatter and Stable Section IDs

kaits process documents use Markdown with YAML frontmatter. Sections have **stable IDs** (HTML anchors) separate from heading text, so headings can evolve without breaking `process_ref` pointers.

```markdown
---
id: proc-release
version: "1.2"
title: Release Process
owner: engineering
triggers:
  - release
  - quality-gate
roles:
  - developer
  - qa
  - pm
maturity: defined
last_reviewed: 2026-03-15
supersedes: null
tags: [delivery, quality]
---

# Release Process

<section id="purpose">

## Purpose and Scope

Define the steps to produce a production-ready release with validated quality.

</section>

<section id="prerequisites">

## Prerequisites

- All feature branches merged to main
- CI pipeline green
- CHANGELOG.md updated

</section>

<section id="steps">

## Steps

### Step 1: Test Suite {#step-test}

Run the full test suite. All tests must pass with zero failures.

**Role:** Developer
**Quality criteria:** Zero test failures, coverage ≥ threshold
**Output:** Test report artifact

### Step 2: Lint Check {#step-lint}

Run linter with zero-error policy.

**Role:** Developer
**Quality criteria:** Zero lint errors
**Output:** Lint report

### Step 3: Documentation Review {#step-docs}

Ensure documentation reflects current state.

**Role:** PM or Developer
**Quality criteria:** All public APIs documented, README current
**Output:** Documentation diff

### Step 4: Create Release {#step-release}

Create changelog entry and git tag with semantic version.

**Role:** Developer (with PM approval)
**Quality criteria:** Semantic versioning followed, changelog complete
**Output:** Git tag, changelog entry

</section>

<section id="dod">

## Definition of Done

- Zero test failures
- Zero lint errors
- CHANGELOG.md updated with version entry
- Git tag created and pushed
- All artifacts generated

</section>

<section id="metrics">

## Metrics

- Release cycle time (first commit → tag)
- Defect escape rate (bugs found after release)
- Rollback frequency

</section>
```

### 4.3 Stable Section IDs for process_ref

The `process_ref` format uses stable IDs, not heading text:

```yaml
# In a trigger definition
process_ref: "context/processes/release.md#step-test"
```

This points to the `<section id="step-test">` anchor, not the heading text "Step 1: Test Suite". If the heading is renamed to "Step 1: Execute Test Suite", the `process_ref` still resolves correctly.

Rules for stable IDs:
- Use lowercase kebab-case: `step-test`, `quality-gate`, `dod`
- Never rename an ID once published — deprecate and add a new one instead
- IDs are unique within a document
- Document the ID in the section tag: `<section id="...">`

### 4.4 Versioning Strategy

Process documents use semantic versioning in the frontmatter:
- **Major version** (1.x → 2.x): Breaking changes — steps removed, roles changed, section IDs removed
- **Minor version** (1.1 → 1.2): Additive changes — new steps, new quality criteria, new metrics
- **Patch version** (1.1.1 → 1.1.2): Clarifications, typo fixes, no structural changes

Version history is tracked via git commits on the process document file. The frontmatter `version` field is the human-readable version; git SHA provides the immutable reference.

### 4.5 Process Hierarchy

Four levels of process abstraction:

1. **Meta-processes** — how to create, monitor, and improve processes (see §6)
2. **Standard processes** — organization-wide process templates (e.g., "Release Process", "Feature Development")
3. **Tailored processes** — standard processes adapted for a specific project or team (e.g., "Release Process for kaits" adds specific test commands)
4. **Process instances** — a specific execution of a tailored process for a specific task (e.g., "Release v0.5.0")

This hierarchy maps to the CMMI concept of "Standard Process" → "Defined Process" → "Process Instance."

---

## 5. Process Maturity Models

### 5.1 CMMI v3.0

CMMI (Capability Maturity Model Integration) v3.0, released April 2023, integrates Development, Services, and Supplier Management into a single framework. It defines five maturity levels:

| Level | Name | Description | Key Characteristics |
|---|---|---|---|
| 1 | Initial | Unpredictable, reactive | Ad-hoc processes, success depends on individual heroics |
| 2 | Managed | Basic project management | Projects are planned and executed per policy; basic metrics collected |
| 3 | Defined | Standardized and documented | Organization-wide standard processes; projects tailor from the standard |
| 4 | Quantitatively Managed | Metrics-driven control | Statistical process control; quantitative quality and performance objectives |
| 5 | Optimizing | Continuous improvement | Innovation-driven; systematic root cause analysis and process optimization |

Key change in CMMI v3.0: Maturity Level 2 is now defined as all Practice Areas at Capability Level 2 — a major philosophical shift from previous versions where ML2 focused on project management. In v3.0, maturity is the improvement of all processes in parallel.

### 5.2 IPMA Delta Levels

(See §3.2 above for the 5-class system: Initial → Defined → Standardized → Managed → Optimizing)

### 5.3 OPM3 (Organizational Project Management Maturity Model)

PMI's OPM3 differs from CMMI in using a **continuum** rather than discrete levels. It maps best practices by both domain (Project, Program, Portfolio) and stage:

1. **Standardize** — processes are documented and consistently applied
2. **Measure** — metrics are collected on process performance
3. **Control** — processes are managed to stay within acceptable bounds
4. **Continuously Improve** — processes are systematically improved based on data

Three implementation phases: Knowledge (understand maturity), Assessment (measure current state), Improvement (close the gap).

### 5.4 Process Maturity for AI Agents — Adapted Model

Traditional maturity models describe human organizations. For AI agents, we need an adapted model:

| Level | Name | Agent Behavior | kaits Indicators |
|---|---|---|---|
| 1 | Ad-hoc | Agent acts on best-effort basis, no consistent process | No process_ref in triggers; no DoD checks; no event logging of process steps |
| 2 | Guided | Agent follows process when explicitly instructed | process_ref exists; agent reads process doc; steps are logged but not enforced |
| 3 | Defined | Agent consistently follows defined process without prompting | Agent autonomously reads process_ref; creates events for each step; Watchdog verifies compliance |
| 4 | Measured | Agent tracks process metrics and reports deviations | Agent computes flow metrics per process execution; SPI/CPI tracked; deviations trigger alerts |
| 5 | Optimizing | Agent proposes process improvements based on data | Agent analyzes process metrics over time; suggests process doc updates; implements approved improvements |

**Maturity progression in kaits:**
- Level 1→2: Add process_ref to triggers, ensure agents read process docs
- Level 2→3: Watchdog agent enforces process compliance on heartbeat
- Level 3→4: Materializer computes process-level metrics from event log
- Level 4→5: Agent-driven retrospectives that produce process improvement proposals

---

## 6. Meta-Processes

### 6.1 Process for Creating New Processes

A meta-process that governs how new standard processes are created:

```
1. **Identify need** — PM or team member identifies a recurring workflow without a defined process
2. **Draft process document** — Author creates document using the template (§4.2)
   - Define purpose, steps, roles, DoD, metrics
   - Assign stable section IDs
   - Set initial version to "0.1" (draft)
3. **Review** — At least one other role reviews for completeness and clarity
4. **Trial** — Execute the process for 2-3 instances, collect feedback
5. **Approve** — PM approves; version set to "1.0"
6. **Register** — Add to process library index; create trigger definitions if applicable
7. **Communicate** — Inform affected agents via event log
```

### 6.2 Process for Monitoring Process Adherence

How the Process Watchdog agent evaluates whether agents follow defined processes:

```
1. **Heartbeat trigger** — Watchdog runs on its standing task heartbeat (BACK-051)
2. **Scan active tasks** — Find tasks with associated process_ref
3. **Check step completion** — For each task, verify that expected process steps have corresponding events
4. **Identify deviations** — Steps skipped, steps out of order, DoD criteria not met
5. **Generate report** — Create adherence report as event log entry
6. **Escalate** — If critical deviation found, create attention badge (BACK-039) for PM or human
7. **Track trends** — Materializer aggregates adherence scores over time
```

### 6.3 Process for Improving Processes (PDCA)

Continuous improvement using the Deming cycle, adapted for AI agents:

**Plan:** Analyze process metrics (flow time, adherence rate, defect escape rate). Identify the process with the worst performance. Propose specific improvement (e.g., add a review step, remove a bottleneck step, clarify quality criteria).

**Do:** Create a modified version of the process document (minor version bump). Apply the modified process to the next instance.

**Check:** Compare metrics between the old and new process versions. Did flow time improve? Did quality improve? Did adherence remain high?

**Act:** If improvement validated, promote the new version. If not, revert and try a different approach. Document the experiment outcome in the process document's changelog.

### 6.4 Feedback Loops

```
Process Execution → Event Log → Materializer → Metrics
       ↑                                           ↓
Process Update ← Improvement Proposal ← Watchdog Analysis
```

The key insight: all process execution data flows through the event log (BACK-050), which the materializer aggregates into metrics. The Watchdog agent reads these metrics on its heartbeat and proposes improvements. The PM agent approves improvements and updates process documents. This creates a closed feedback loop where processes evolve based on empirical data.

---

## 7. Standard Artifact Types

### 7.1 Decision Records (ADR Format)

kaits already uses ADRs in `context/DECISIONS.md`. Enhanced format for process library:

```markdown
## DEC-NNN: [Title]

**Status:** Proposed | Accepted | Deprecated | Superseded by DEC-XXX
**Date:** YYYY-MM-DD
**Context:** What is the situation that requires a decision?
**Decision:** What was decided and why?
**Consequences:** What are the implications — positive, negative, and risks?
**Alternatives considered:** What was rejected and why?
```

### 7.2 Process Definitions

See §4.2 for the full template. Process definitions are the core artifact of the process library.

### 7.3 Checklists / Definition of Done Templates

Reusable checklists that can be referenced from process steps:

```markdown
---
id: checklist-code-review
version: "1.0"
title: Code Review Checklist
applies_to: [developer, qa]
---

# Code Review Checklist

- [ ] All tests pass
- [ ] No linter errors
- [ ] No hardcoded secrets
- [ ] Public functions have docstrings
- [ ] Complex logic has inline comments
- [ ] No TODO comments without linked task ID
- [ ] Error handling covers edge cases
- [ ] Performance: no N+1 queries, no unnecessary loops
```

### 7.4 Handoff Templates

Structured information that flows between process steps (implemented in BACK-023):

```markdown
---
id: handoff-dev-to-qa
version: "1.0"
title: Developer → QA Handoff
---

# Developer → QA Handoff Template

**Summary of work completed:** [What was built/changed]
**How to test:** [Steps to verify the work]
**Known limitations:** [What is not covered, edge cases]
**Artifacts:**
- Code changes: [file list or PR link]
- Test results: [test output or link]
- Documentation: [updated docs]
**Acceptance criteria status:** [Which criteria are met]
```

### 7.5 Quality Gate Definitions

Quality gates are checkpoints that must be passed before work progresses. They are referenced from trigger definitions:

```markdown
---
id: gate-pre-merge
version: "1.0"
title: Pre-Merge Quality Gate
severity: blocking
---

# Pre-Merge Quality Gate

## Automated Checks
- [ ] All unit tests pass (zero failures)
- [ ] Lint check passes (zero errors)
- [ ] Type check passes (if applicable)
- [ ] Test coverage ≥ threshold

## Manual Checks
- [ ] Code review approved by at least one reviewer
- [ ] Acceptance criteria verified
- [ ] No unresolved TODO comments

## Documentation
- [ ] CHANGELOG updated (if user-facing change)
- [ ] API documentation updated (if public API changed)
```

### 7.6 Retrospective / Lessons Learned Templates

Templates for structured reflection after process execution:

```markdown
---
id: retro-pi
version: "1.0"
title: PI Retrospective Template
cadence: per-PI
---

# PI Retrospective

**Period:** [PI start date] — [PI end date]
**Participants:** [Agent names]

## Metrics Review
- Flow Velocity: [value vs. previous PI]
- Flow Time (avg): [value vs. previous PI]
- Adherence Rate: [% of process steps followed]
- Defect Escape Rate: [bugs found after completion]

## What Went Well
1. [Item]

## What Didn't Go Well
1. [Item]

## Action Items
| Item | Owner | Due Date | Status |
|---|---|---|---|
| [Action] | [Agent] | [Date] | Todo |
```

---

## 8. Architectural Implications for kaits

### 8.1 Process Storage Format: Markdown with Frontmatter

**Decision: Use Markdown files with YAML frontmatter** (not database, not pure YAML).

Rationale:
- Agents already read and write Markdown fluently
- Git provides version history, diffs, and blame
- YAML frontmatter provides structured metadata for programmatic access
- Stable section IDs (`<section id="...">`) enable precise `process_ref` pointers
- Human-readable without tooling
- Aligns with kaits's existing pattern (context files, process docs are already Markdown)

Alternative rejected: Pure YAML process definitions. Too rigid, poor readability for complex multi-step processes, hard for agents to author naturally.

Alternative rejected: Database-stored processes. Adds persistence complexity, loses git history, harder for humans to review and edit.

### 8.2 How process_ref Works with Stable IDs

The `process_ref` field in trigger definitions (BACK-053) resolves to a specific section:

```yaml
# triggers/release.yaml
name: "/release"
process_ref: "context/processes/release.md#step-test"
```

Resolution logic:
1. Parse the path: `context/processes/release.md` → file path
2. Parse the fragment: `#step-test` → section ID
3. Read the file, find `<section id="step-test">` or heading with `{#step-test}`
4. Extract the section content (up to the next section boundary)
5. Provide the section content to the agent as process instructions

If no fragment is specified, the entire document is provided.

### 8.3 Event-Sourced Maturity Tracking

Process maturity per agent and per department is tracked through the event log:

```python
# Event types for process tracking
"process_step_started"   # Agent began a process step
"process_step_completed" # Agent completed a process step
"process_step_skipped"   # Agent skipped a step (deviation)
"process_dod_checked"    # Agent verified DoD criteria
"process_dod_failed"     # DoD check failed
"process_adherence_report" # Watchdog adherence report
```

Maturity score computation:
- **Adherence rate** = completed steps / expected steps (per process instance)
- **DoD compliance** = passed DoD checks / total DoD checks
- **Process coverage** = tasks with process_ref / total tasks
- **Improvement rate** = process improvements proposed / PIs elapsed

These metrics are computed by the materializer (BACK-050) and stored as materialized views for dashboard display.

### 8.4 Meta-Process Enforcement

The Process Watchdog agent (BACK-053) enforces meta-processes:

1. **On heartbeat:** Scan for active tasks with process_ref; check event log for step completion
2. **On task completion:** Verify DoD before allowing status transition
3. **On PI boundary (periodic):** Generate adherence report; compare current PI to previous
4. **On process document change:** Validate the change follows the meta-process for updating processes

The Watchdog operates autonomously on its standing task heartbeat but escalates to PM or human for critical deviations.

### 8.5 Framework-to-kaits Mapping Table

| Framework Concept | kaits Primitive | Status |
|---|---|---|
| SAFe PI | Heartbeat cycle × N | Implemented (BACK-046) |
| SAFe Flow Metrics | Event log + materializer | Implemented (BACK-050) |
| SAFe ART | Department | Planned (BACK-054) |
| SAFe Portfolio Kanban | Project lifecycle | Implemented (BACK-052) |
| PMBOK Performance Domains | Process template categories | This spike |
| PMBOK Tailoring | PM agent process selection | Planned (BACK-055) |
| PMBOK EVM | Token-cost tracking | Planned |
| IPMA ICB4 Competences | Skill tree branches | Implemented (BACK-026) |
| IPMA Levels D→A | Agent leveling | Implemented (DEC-019) |
| IPMA Delta | Watchdog assessment | Implemented (BACK-053) |
| CMMI Maturity Levels | Agent maturity model (§5.4) | This spike |
| CMMI Standard Process | Process templates | Planned (BACK-055) |
| OPM3 SMCI cycle | Watchdog + materializer | Partially implemented |

### 8.6 Recommended Build Order

Based on this research, the recommended implementation sequence for process library features:

1. **Process document template** — Define the standard Markdown+frontmatter template (§4.2). Convert existing process docs (`release.md`, `quality-gate.md`) to the new format with stable section IDs. No code changes needed.

2. **process_ref resolver** — Implement the logic to parse `process_ref` with fragment IDs and extract section content. Integrate into trigger activation (BACK-053).

3. **Process step events** — Add process-specific event types to the event log. Agents emit `process_step_started/completed` events as they execute process steps.

4. **Process templates (BACK-055)** — Create standard process templates for common workflows (feature development, research spike, bug fix, release). PM agent selects template when structuring work.

5. **Watchdog adherence checking** — Extend the Process Watchdog to compare event log against process definitions and generate adherence reports.

6. **Maturity metrics** — Extend the materializer to compute process maturity metrics (adherence rate, DoD compliance, process coverage). Surface on dashboard.

7. **Tailoring framework** — Enable PM agent to create tailored processes from standard templates based on project characteristics (size, risk, complexity).

8. **Process improvement loop** — Enable Watchdog to propose process improvements based on metric trends. PM agent reviews and approves changes.

---

## Sources

### SAFe 6.0
- [SAFe Framework](https://framework.scaledagile.com/)
- [What's New in SAFe 6.0](https://framework.scaledagile.com/whats-new-in-safe-6-0/)
- [SAFe Hierarchy Explained](https://www.enov8.com/blog/the-hierarchy-of-safe-scaled-agile-framework-explained/)
- [SAFe Lean Portfolio Management](https://agility-at-scale.com/safe/lpm/)
- [SAFe PI Planning](https://agility-at-scale.com/safe/team-technical-agility/pi-planning/)
- [SAFe Inspect and Adapt](https://agility-at-scale.com/safe/team-technical-agility/inspect-and-adapt/)
- [SAFe Flow Metrics](https://agileseekers.com/blog/using-flow-metrics-to-prioritize-features-in-safe)

### PMBOK 7th Edition
- [PMI PMBOK Guide](https://www.pmi.org/standards/pmbok)
- [8 Performance Domains in PMBOK 7th Edition](https://mpug.com/8-planning-and-delivery-performance-domains)
- [PMBOK 7 vs PMBOK 8](https://projectmanagementacademy.net/resources/blog/pmbok-7-vs-pmbok-8-differences/)
- [Earned Value Management Fundamentals](https://aliresources.hexagon.com/enterprise-project-performance/earned-value-management)
- [EVM Formulas](https://www.parallelprojecttraining.com/blog/earned-value-equations/)

### IPMA
- [ICB4 Standard](https://ipma.world/ipma-standards-development-programme/icb4/)
- [IPMA OCB](https://ipma.world/ipma-standards-development-programme/ocb/)
- [IPMA Delta and OCB Research](https://www.researchgate.net/publication/263480419_IPMA_Delta_and_IPMA_Organisational_Competence_Baseline_OCB_New_approaches_in_the_field_of_project_management_maturity)
- [ICB4 Competence Framework](https://umbrex.com/resources/frameworks/project-management-frameworks/ipma-competence-baseline-icb/)

### CMMI
- [CMMI v3.0 Changes](https://www.prowisesystems.com/cmmi-v3-0-what-changed-and-how-to-prepare/)
- [CMMI Levels](https://cmmiinstitute.com/learning/appraisals/levels)
- [CMMI v3 Update Explained](https://www.thecoresolution.com/cmmi-v3-update-explained)

### OPM3
- [PMI OPM3](https://www.pmi.org/learning/library/pmi-organizational-maturity-model-7666)
- [OPM3 Primer](https://www.pmi.org/learning/library/grow-up-already-opm3-primer-8108)
- [Project Management Maturity Models Overview](https://www.epicflow.com/blog/project-management-maturity-models-a-basis-for-reaching-your-organizations-business-success/)
