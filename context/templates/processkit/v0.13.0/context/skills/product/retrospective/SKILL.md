---
name: retrospective
description: |
  Facilitates team or project retrospectives — what worked, what didn't, action items. Use at the end of a sprint, milestone, or project phase to reflect on what went well, what didn't, and what to try next.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-retrospective
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: product
    layer: 3
---

# Retrospective

## Intro

A retrospective is a short, structured reflection on a completed
phase of work. Capture what worked, what didn't, and a small set
of concrete experiments to try next — then act on them.

## Overview

### Set the scope

Be explicit about the period under review: a sprint, a milestone,
a release, a project phase. A vague scope produces vague feedback;
a tight scope produces actionable items.

### Gather input in three categories

- **What worked well** — practices, tools, decisions that helped.
- **What didn't work** — pain points, blockers, things that slowed
  the team down.
- **What to try next** — concrete experiments or changes for the
  next iteration.

### Format as a structured document

```markdown
## Retrospective — [date/milestone]

### What Worked
- Item 1
- Item 2

### What Didn't Work
- Item 1
- Item 2

### Action Items
- [ ] Specific, assignable action 1
- [ ] Specific, assignable action 2
```

### Action items must be specific, owned, and time-bound

Not "improve testing" but "add integration tests for the auth
module by end of next sprint, owner @alice." If you cannot name
the owner and the date, the item will not happen.

### Store it where the team will find it

Save the retro in `context/` or the project's designated location.
Link to it from the next sprint's kickoff so the action items
actually feed into planning.

### Example

User says "let's do a retro on the v0.3 release". The agent asks
what went well and what was painful, structures the findings into
the retro format with concrete action items, and saves it to
`context/retros/v0.3.md`.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Action items with no owner or due date.** An action item that says "improve testing" with no owner and no date will not happen. Every action must name who is responsible and when it is due — otherwise it is a wish, not a commitment.
- **Discussing the same blockers retro after retro.** If the same pain point appears in three consecutive retrospectives with no change, the problem is not the retro — it is that action items are not being followed through. Audit completion before the next retro starts.
- **Letting the loudest voices dominate.** Verbal-first retros favor extroverts and senior voices. Use silent individual input gathering (sticky notes, async docs) before opening group discussion to surface the full range of experience.
- **Skipping the retro because "we are too busy".** Busyness is the symptom that makes the retro most valuable. A team that never reflects never improves its throughput. Protect the retro time even at the cost of other meetings.
- **Treating the retro as a complaint session.** The "what didn't work" section creates energy only if it leads to "what to try next." If the output is a list of grievances with no action items, the retro has failed.
- **Storing retros where nobody looks at them again.** A retrospective document in a folder no one revisits produces no learning. Link to the retro from the next sprint's kickoff so action items feed into planning.
- **Vague scope for the retrospective.** "Let's reflect on the project" produces generic feedback. "Let's reflect on the v0.5 release, specifically the last two weeks" produces specific, actionable items.

## Full reference

### Facilitation patterns

Common formats besides the three-column default:

- **Start / Stop / Continue** — what should we start doing, stop
  doing, continue doing.
- **Mad / Sad / Glad** — emotion-led, useful when team morale is
  the topic.
- **4 Ls** — Liked / Learned / Lacked / Longed for.
- **Sailboat** — wind (helpers), anchors (blockers), rocks
  (risks), island (goal).

Pick a format that matches the question you actually want to
answer. Rotate formats so retros do not become routine.

### Anti-patterns

- Action items with no owner or due date
- Discussing the same blockers retro after retro with no change
- Treating the retro as a complaint session with no follow-through
- Letting the loudest voices dominate; use silent input gathering
  (sticky notes, async docs) before discussion
- Skipping the retro "because we are too busy" — that is usually
  exactly when you need it
- Storing retros where nobody looks at them again
