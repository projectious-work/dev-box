---
name: schedule-management
description: |
  Manage Schedule entities — time-based triggers and recurring cadences (daily standup, weekly review, monthly retro). Use when defining a recurring cadence — daily/weekly/monthly — that triggers a process or reminder.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-schedule-management
    version: "1.0.0"
    created: 2026-04-06T00:00:00Z
    category: processkit
    layer: 3
    uses:
      - skill: event-log
        purpose: Log events to keep the audit trail accurate after every write.
    provides:
      primitives: [Schedule]
      templates: [schedule]
---

# Schedule Management

## Intro

A Schedule is a time-based trigger describing when a process should run or
a reminder should fire — daily standups, weekly retros, monthly ops reviews,
quarterly planning.

## Overview

### Shape

```yaml
---
apiVersion: processkit.projectious.work/v1
kind: Schedule
metadata:
  id: SCHED-daily-standup
  created: 2026-04-06T00:00:00Z
spec:
  name: daily-standup
  description: "Daily async standup at 09:00 local time."
  cadence: daily
  cron: "0 9 * * 1-5"
  timezone: Europe/Berlin
  triggers:
    - process: standup
  active: true
---
```

### Workflow

1. Pick `SCHED-<name>`.
2. Set a human `cadence` (`daily`/`weekly`/`monthly`/`quarterly`/`adhoc`/`custom`).
3. For machine-readable scheduling, set `cron` + `timezone`.
4. Link what it `triggers` — a process name, a reminder text, etc.
5. Use Bindings to scope a schedule to a specific team or project.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Schedule without timezone.** "Daily at 9am" is meaningless
  across DST boundaries and timezone-distributed teams. Always
  specify the timezone explicitly (`UTC`, `Europe/Berlin`, etc.).
  Schedules without timezone drift twice a year.
- **Recurring schedule without an end date or `until`.** Some
  schedules genuinely run forever; others should end with the
  project, the quarter, or the team member's tenure. Make the
  termination condition explicit, even if it's "indefinite" — the
  decision to make it indefinite is itself a decision worth
  recording.
- **Forgetting to handle missed runs.** What happens if the
  schedule fires but the action fails? Does it retry? Skip? Page
  oncall? A Schedule without a missed-run policy silently
  accumulates errors.
- **Schedule fires without an action target.** A Schedule must
  point at a Process, WorkItem template, or other actionable
  artifact. "Trigger something at 9am" with no target is a
  reminder, not a Schedule.
- **Confusing schedule definition vs schedule execution.** The
  Schedule entity defines WHEN something fires; the actual fire
  events should be LogEntries (`schedule.fired`). Don't put
  execution history in the Schedule entity itself.
- **Hand-rolled cron expressions instead of using the schema.**
  Cron strings are infamous for off-by-one errors (every 5
  minutes vs every minute past the 5th). Use the schema's
  structured fields (interval, days, hours) when possible;
  reserve cron strings for cases the schema can't express.
- **Schedule that overlaps with another.** "Daily standup at 9am"
  and "weekly review at 9am Monday" both fire at 9am Monday. Be
  explicit about precedence — does one preempt the other, do
  both run, or does the agent skip one?

## Full reference

### Fields

| Field       | Type         | Notes                                              |
|-------------|--------------|----------------------------------------------------|
| `name`      | string       | Kebab-case identifier                              |
| `description` | string     | Human-readable                                     |
| `cadence`   | enum         | `daily`/`weekly`/`monthly`/`quarterly`/`adhoc`/`custom` |
| `cron`      | string       | Standard cron expression                           |
| `timezone`  | string       | IANA timezone name                                 |
| `triggers`  | list[object] | What fires: processes, reminders, events           |
| `active`    | bool         | `false` = suspended                                |
| `last_run`  | datetime     | Advisory — set by whoever runs the schedule        |
| `next_run`  | datetime     | Advisory                                           |

### Execution

processkit does NOT execute schedules. It describes them. An external
runner (GitHub Actions, cron, a human, an agent on a loop) reads the
Schedule and fires the trigger at the right time. The `triggers` field is
the contract between the Schedule definition and whoever runs it.

### Scoped schedules

A daily standup runs in one scope, not globally. Use a Binding:

```yaml
kind: Binding
spec:
  type: schedule-scope
  subject: SCHED-daily-standup
  target: SCOPE-project-x
```

Then the runner fetches all schedules bound to the scope it's acting on.
