---
name: postmortem-writing
description: |
  Blameless postmortem writing — timeline, root cause analysis, corrective actions. Use when writing an incident postmortem, conducting a post-incident review, building a postmortem template, or coaching a team toward blameless culture.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-postmortem-writing
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
    layer: 3
---

# Postmortem Writing

## Intro

A good postmortem is a learning artifact, not a punishment. It tells
the story of an incident factually, traces the root cause to a
systemic gap (not a person), and produces a small number of owned,
time-bound corrective actions.

## Overview

### Standard structure

Every postmortem should include, in order: title and metadata,
summary, impact, timeline, root cause analysis, contributing
factors, what went well, corrective actions, and lessons learned.
The order matters — facts first, analysis second, actions last.

### Writing the timeline

Use UTC timestamps. Format each line as
`HH:MM UTC — Event description (who/what)`. Include first alert,
escalation points, key diagnostic steps, mitigations applied, and
full resolution. Separate detection time (first signal) from response
time (first human action) and call out delays explicitly. The
timeline is factual, not interpretive — analysis goes in the root
cause section.

### Root cause analysis with 5 Whys

Start with the observable failure and ask "why?" iteratively. Stop
when you reach a systemic or process issue, not a human mistake. If
the chain branches, document all branches. Example:

- Why did the site go down? The database ran out of connections.
- Why? A query held connections open for 30+ seconds.
- Why? It was doing a full table scan on a 50M row table.
- Why no index? The migration that added the column did not include
  one.
- Why was that not caught? There is no review step for migration
  performance impact.

The root cause is "no migration performance review", not "Alice
wrote a slow migration."

### Corrective actions

Each action must be specific, owned, and have a due date. Categorize
as **prevent** (stop it from happening), **detect** (catch it
faster), or **mitigate** (reduce impact). Format:
`[P/D/M] Action — @owner — due YYYY-MM-DD`. Cap at 5–8 actions; more
means none get done. Aim for at least one action per category. Track
completion in the team's issue tracker, not just the document.

### Blameless culture

Focus on systems and processes, not individuals. Use passive voice
for human errors ("the config was deployed without review", not
"Alice deployed without review"). Ask "what about our system made
this mistake easy to make?" Assume everyone acted with the best
information available at the time. If anyone feels blamed, the
postmortem has failed regardless of wording.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Action item bloat.** Twenty action items with no owners mean none get done. Cap at 5–8 actions, assign a named owner and a due date to each, and track them in the team's issue tracker rather than only in the document.
- **Stopping the 5 Whys at the symptom level.** "The query was slow" is a symptom, not a root cause. Keep asking why until you reach a systemic gap: "no migration performance review step existed." The root cause must be something the team can change.
- **Blame disguised as systems language.** "The team should have noticed" is still blame. Rewrite as "no automated check existed for this pattern." If the postmortem names individuals in the cause chain rather than systemic gaps, the culture will drift away from blamelessness.
- **Postmortem written but never referenced.** A document filed and forgotten produces no learning. Link every corrective action to a tracked ticket; schedule a follow-up review at 30 days to check completion.
- **Scheduling the review too late.** Memories of the incident fade rapidly. Schedule the review within 3–5 business days. Waiting for a "convenient time" usually means the review happens weeks later, if at all.
- **Author and facilitator being the same person.** The author is defensive about the incident they were in; the facilitator needs to be neutral. Separate the roles even on small teams.
- **All actions in one category.** A postmortem that produces only "Prevent" actions has no "Detect" or "Mitigate" items, which means the next similar incident will still take too long to detect and contain. Aim for at least one action per category.

## Full reference

### Facilitation tips

- Schedule the review within 3–5 business days of the incident;
  memories fade fast.
- Invite all responders plus 1–2 people who were not involved for
  fresh perspective.
- Time-box to 60 minutes: 10 min timeline walk-through, 20 min root
  cause, 20 min actions, 10 min wrap-up.
- The facilitator should not be the author — separating the roles
  reduces defensiveness.
- Start by reading the timeline aloud as a group to establish a
  shared factual baseline.
- End with: "What is the single most important thing we should
  change?"

### Template

```markdown
# Postmortem: [Incident Title]

**Date:** YYYY-MM-DD
**Severity:** SEV-1 / SEV-2 / SEV-3
**Duration:** HH:MM (from detection to resolution)
**Author:** [Name]
**Reviewers:** [Names]

## Summary
[2-3 sentences: what happened, impact, resolution]

## Impact
- Users affected: [number or percentage]
- Duration of user impact: [time]
- Revenue impact: [estimate if applicable]
- SLA budget consumed: [percentage]

## Timeline (UTC)
- HH:MM — [First signal / alert fired]
- HH:MM — [Escalation / response began]
- HH:MM — [Key diagnostic finding]
- HH:MM — [Mitigation applied]
- HH:MM — [Full resolution confirmed]

## Root Cause Analysis
### 5 Whys
1. Why? ...
2. Why? ...
3. Why? ...
4. Why? ...
5. Why? ...

## Contributing Factors
- [Factor that made it worse or delayed resolution]

## What Went Well
- [Thing that worked during response]

## Corrective Actions
| Type | Action | Owner | Due Date | Status |
|------|--------|-------|----------|--------|
| Prevent | [Action] | @name | YYYY-MM-DD | Open |
| Detect | [Action] | @name | YYYY-MM-DD | Open |
| Mitigate | [Action] | @name | YYYY-MM-DD | Open |

## Lessons Learned
- [Key takeaway]
```

### Common failure modes

- **Action item bloat.** Twenty actions, none done. Cut to the top
  five and defer the rest with explicit justification.
- **Symptom-level root cause.** Stopping at "the query was slow"
  instead of asking why the slow query reached production.
- **Blame disguised as systems language.** "The team should have
  noticed" is still blame. Rewrite as "no automated check existed
  for X."
- **Postmortem as archive.** A document that is written, filed, and
  never referenced. Link corrective actions out to tickets and
  schedule a follow-up review.
