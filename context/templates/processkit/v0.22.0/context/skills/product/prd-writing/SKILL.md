---
name: prd-writing
description: >
  Write a Product Requirements Document structured with problem
  statement, goals, user stories, scope, and success metrics.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-prd-writing
    version: "1.1.0"
    created: 2026-04-08T00:00:00Z
    category: product
    layer: 2
    uses:
      - skill: workitem-management
        purpose: Link requirements to WorkItems in the project backlog.
      - skill: decision-record
        purpose: Reference DecisionRecords that motivated requirements
          or resolved open questions.
    provides:
      primitives: [Artifact]
      mcp_tools: []
      assets: []
      processes: [prd-writing]
---

# PRD Writing

## Intro

A PRD (Product Requirements Document) aligns the team on what to
build, why, and how to know when it's done. It is not a design
spec, not a technical spec, and not a project plan — it is the
bridge between user need and team execution. A good PRD can be read
in 15 minutes and leave no major questions unanswered.

## Overview

### Feature PRD vs. product/vision PRD

processkit distinguishes two PRD variants:

**Feature PRD** — scopes a single feature or change. Audience: the
immediate delivery team. Contains: problem statement, use cases with
acceptance criteria, functional + non-functional requirements,
success metrics. Template: see `### Feature PRD template` below.

**Product/vision PRD** — defines the product itself: what it is, who
it serves, what it must do, and how success is measured. Audience:
contributors, stakeholders, and agents picking up the project cold.
Contains: vision, target users, component map, core requirements,
non-goals, success metrics, milestones, glossary, open questions.
Template: see `### Product PRD template` below.

### Feature PRD template

```markdown
# [Feature Name] — Product Requirements

**Status:** Draft | Review | Approved  
**Owner:** [PM name]  
**Target release:** [Version / Quarter]  
**Last updated:** [Date]

---

## Problem statement

[2-3 sentences: what user problem is this solving? What evidence
confirms it's real? Why now?]

## Goals

**Primary goal:** [The single most important outcome — one sentence]

**Secondary goals:**
- [Additional measurable outcome]
- [Additional measurable outcome]

**Non-goals (explicitly out of scope):**
- [What we are NOT building in this version]
- [What we will address in a future iteration]

## Users and use cases

**Primary user:** [Role / persona, with 1-sentence description]

### Use cases

**Use case 1: [Name]**
> As a [user type], I want to [action] so that [benefit].

- **Current experience:** [How users solve this today, or don't]
- **Desired experience:** [What the new experience looks like]
- **Acceptance criteria:**
  - [ ] [Specific, testable criterion]
  - [ ] [Specific, testable criterion]

**Use case 2: [Name]**
[Same structure]

## Requirements

### Functional requirements

| ID | Requirement | Priority | Notes |
|---|---|---|---|
| REQ-001 | [Capability the product must have] | Must | |
| REQ-002 | [Capability the product must have] | Should | |
| REQ-003 | [Capability nice to have] | Could | |

Use MoSCoW: Must / Should / Could / Won't.

### Non-functional requirements

- **Performance:** [e.g., "p99 response < 500ms under 1000 concurrent users"]
- **Availability:** [e.g., "99.9% uptime"]
- **Security:** [e.g., "data encrypted at rest and in transit"]
- **Accessibility:** [e.g., "WCAG 2.1 AA compliance"]

## Success metrics

| Metric | Baseline | Target | Timeframe | How measured |
|---|---|---|---|---|
| [Primary metric] | [Current value] | [Goal] | [e.g., 90 days post-launch] | [Data source] |
| [Secondary metric] | | | | |

**Counter-metrics** (watch for regressions):
- [Metric that should not worsen]

## Dependencies and constraints

- **Technical dependencies:** [APIs, infrastructure, other teams' work]
- **Design dependency:** [Design work required before development]
- **Timeline constraint:** [Hard deadlines and their reason]
- **Regulatory constraint:** [Compliance requirements]

## Open questions

| Question | Owner | Due by | Status |
|---|---|---|---|
| [Decision needed] | [Name] | [Date] | Open |

## Appendix

[Links to: user research, design mockups, competitive analysis,
analytics data, technical feasibility notes]
```

### Product PRD template

For product-level or vision-level PRDs:

~~~markdown
# [Product Name] — Product Requirements

**Status:** Draft | Review | Approved
**Owner:** [name or team]
**Last updated:** [YYYY-MM-DD]

---

## Vision

[2-4 sentences: what is this product, who is it for, and why does
it exist? Distinct from goals — this is the directional statement.]

## Problem statement

[What problem does this product solve? What evidence confirms it is
real? Why is this the right solution?]

## Goals

**Primary goal:** [The single most important outcome.]

**Secondary goals:**
- [Measurable secondary outcome]

**Non-goals:** See Non-goals section.

## Target users

| User | What they get |
|---|---|
| [Role] | [Value] |

## Component map

[ASCII diagram or description of how major components relate.]

## Core requirements

### R1 — [Requirement name]

[Requirement description. Reference the format spec or schema where
applicable rather than reproducing it inline.]

### R2 — ...

## Non-functional requirements

- **Portability:** ...
- **Offline capability:** ...
- **Backward compatibility:** ...

## Non-goals

- [Explicitly out of scope]

## Success metrics

| Metric | Status | Target |
|---|---|---|
| [Metric] | [Current] | [Goal] |

## Milestones

[What has shipped. What v1.0 / next milestone requires. No date
commitments unless hard deadlines exist.]

## Glossary

| Term | Definition |
|---|---|
| [Term] | [One-line definition] |

## Open questions and constraints

**Technical constraints:**
- [Hard constraint]

**Open design decisions:**
- [Decision not yet made]

## Appendix

- [Link to canonical spec files, related decisions, research]
~~~

### Before writing — gather the inputs

A PRD written without evidence is advocacy, not requirements. Gather
first:

1. **User pain**: interviews, support tickets, NPS feedback, usage
   data showing the gap
2. **Business case**: why this over other options? Revenue impact,
   churn risk, strategic fit
3. **Technical feasibility**: rough signal from engineering that
   the approach is buildable in the target timeframe
4. **Competitive context**: how do alternatives solve this?

### Scoping: non-goals are as important as goals

Every PRD should explicitly list what is NOT in scope. Non-goals:
- Prevent scope creep during development
- Let the team build faster (smaller, clearer target)
- Force the PM to actually choose priorities

If every obvious extension is in-scope, the PRD is not prioritized.

### Writing acceptance criteria

Each use case must have testable acceptance criteria — not "works
well" but "the user can complete [task] without [error]".

**Bad:** "The export works correctly."  
**Good:**
- [ ] User can export up to 10,000 rows as CSV in < 5 seconds
- [ ] Export includes all columns visible on the current filter
- [ ] Filename defaults to `[report-name]-[YYYY-MM-DD].csv`

### Keeping the PRD current

A PRD that is written once and never updated becomes a liability.
Mark decisions made, questions answered, and requirements changed.
A "Status: Approved" PRD with open questions that were resolved
without updating the document causes downstream confusion.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Writing solution first, problem second.** A PRD that opens with "We will build a dashboard" skips the why — it's a solution looking for a problem. Always lead with the user problem and the evidence that it's real. The solution follows from the problem, not the other way around.
- **Writing acceptance criteria that are not testable.** "The UI is intuitive" is not an acceptance criterion — it cannot be verified. "A new user can complete the primary task in under 2 minutes without asking for help" is testable. Each acceptance criterion must have a binary pass/fail.
- **Omitting non-goals.** Without explicit non-goals, the team will build the obvious extensions and the feature will take 3× as long. Non-goals force a prioritization decision and give the team permission to say "that's out of scope" during development.
- **Writing requirements that prescribe implementation instead of behavior.** "Use a dropdown" is an implementation decision; "the user selects a single option from the list" is a requirement. PRDs should describe what the product does, not how it's built — that's the team's decision.
- **Skipping success metrics.** A PRD without success metrics produces a feature you can ship but never evaluate. If you cannot define what "success" looks like before building, you cannot tell after shipping whether the investment was worth it.
- **Leaving open questions open at launch.** Questions that are open at PRD approval must be answered before development starts. A PRD with open architecture questions or unresolved design decisions is not ready for development — it will be decided ad-hoc during implementation, usually badly.
- **Making the PRD too long.** A PRD that takes 2 hours to read will not be read by the team before development starts. Keep it to one focused session (15-30 min) — the full detail belongs in linked design docs, technical specs, and research reports, not in the PRD itself.
- **Writing a PRD without loading this skill first.** The trigger
  phrase "PRD" or "product requirements" should load this skill
  before any writing begins. Writing from general knowledge produces
  a document that may look correct but misses processkit-specific
  conventions (Artifact storage, workitem linking, feature vs.
  product PRD distinction).

## Full reference

### PRD vs. technical spec vs. design spec

| Document | Owner | Answers | When written |
|---|---|---|---|
| PRD | Product manager | What and why | Before design and eng start |
| Design spec | Designer | How it looks and feels | After PRD approval |
| Technical spec | Engineering | How it's built | After design spec |

The PRD is the source of truth for requirements. Design and technical
specs are downstream — they should be consistent with the PRD, and
contradictions should trigger a PRD update, not a silent departure.

### MoSCoW prioritization

| Priority | Meaning | Implication |
|---|---|---|
| **Must** | The feature cannot ship without this | Block release if missing |
| **Should** | High value but can be worked around temporarily | Include if time allows; don't block |
| **Could** | Nice to have, low risk if cut | Cuts first when scope must shrink |
| **Won't** | Explicitly out of scope for this version | Documents a conscious choice |

### Linking to processkit context

When the project uses processkit, link the PRD to:
- The WorkItem(s) it spawns (list BACK-NNN IDs in the appendix)
- Relevant decisions (`context/decisions/` or DECISIONS.md)
- Research notes (Note entities with `type: reference`)

Save the PRD as `context/artifacts/prd-[feature-name]-[date].md` or
link it as an Artifact entity in the index.

The PRD itself should be stored as an `Artifact` entity under
`context/artifacts/prd-<product-or-feature>-<date>.md` so it is
indexed and queryable. Create the entity with:

    spec:
      name: <descriptive name>
      kind: document
      location: context/artifacts/prd-<name>-<date>.md
      format: markdown
      tags: [prd]
