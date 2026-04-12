---
name: incident-response
description: |
  Production incident handling — triage, communicate, mitigate, fix, postmortem. Use when production is broken, users are affected, an outage is in progress, or a deploy has caused regressions and the priority is restoring service.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-incident-response
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
    layer: 3
---

# Incident Response

## Intro

When production breaks, restore service first and understand it
later. Triage user impact, communicate early, mitigate with the
fastest safe action (often a rollback), then fix the root cause and
write a blameless postmortem within 48 hours.

## Overview

### Triage in the first five minutes

Answer three questions before doing anything else: what is the user
impact (total outage, degraded, cosmetic), what changed recently
(deploy, config flip, dependency update), and is there a quick
rollback option. The answers decide whether you mitigate by reverting
or by patching forward.

### Communicate early and often

Notify stakeholders immediately with what is broken, who is affected,
and an ETA to next update — not an ETA to fix. Post to the status
page or incident channel. Re-post on a regular cadence even if there
is no progress; silence is worse than bad news.

### Mitigate before fixing

Mitigation restores service; the fix comes later. Roll back if it is
safe and quick. Apply a workaround (feature flag off, traffic shed,
cache bypass) to stop the bleeding. Do not debug in production if you
can reproduce the issue elsewhere.

### Fix and verify

Once service is stable, identify the root cause, apply the fix in a
normal change-management path, and deploy with extra monitoring.
Resist the urge to ship the fix straight to prod under incident
adrenaline — that is how you create the next incident.

### Postmortem within 48 hours

Write a blameless postmortem covering the timeline, root cause
analysis, what went well, what did not, and concrete corrective
actions. Focus on systems and processes, never individuals. See the
`postmortem-writing` skill for the full template.

### Example

User reports the API is returning 500s after the latest deploy. The
agent checks the deploy diff, identifies the breaking change,
recommends an immediate rollback while preparing a forward fix,
drafts a stakeholder update, and schedules the postmortem.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Debugging instead of mitigating when a rollback is available.** When a deploy caused the incident and a rollback would restore service in minutes, spending time reproducing and root-causing the failure while users are affected is the wrong priority order. Mitigate first — roll back, toggle the feature flag, or shed traffic — then diagnose once service is restored.
- **Going silent on stakeholders while investigating.** A status page or incident channel that hasn't been updated in 30 minutes signals to stakeholders that either nothing is happening or the team forgot them. Post an update every 15–30 minutes even if it is "still investigating, no change in scope" — silence is more alarming than bad news delivered regularly.
- **Combining the incident commander role with hands-on debugging.** An engineer simultaneously writing code and running the incident response makes poor decisions in both roles. The commander's job is to coordinate, gather context, and make calls. The technical lead's job is to dig into the system. These must be separate people or the investigation becomes uncoordinated.
- **Shipping the fix to production under incident adrenaline without review.** During an incident, the pressure to fix quickly leads to skipping code review, testing, and staging deployment. The incident adrenaline fix that goes straight to production is one of the most common causes of a second, worse incident. Once service is mitigated, apply the permanent fix through normal change-management paths.
- **Skipping the postmortem because "we know what happened."** The postmortem serves multiple purposes: capturing the full timeline while memory is fresh, identifying contributing factors beyond the surface cause, generating systemic improvements, and sharing learning with the team. Skipping it because the immediate cause is clear means missing contributing factors and losing the opportunity to improve.
- **Corrective actions that live only in the postmortem document.** Action items that are tracked only in a postmortem document and not moved to the team's task tracker will not be completed. Within 48 hours of the postmortem, every action item must have an owner, a priority, and a ticket in the team's project management system.
- **Naming individuals in the timeline or root cause.** "Alice pushed a bad config" shifts blame to a person rather than examining the systems that allowed the bad config to reach production without detection. Blameless postmortems focus on process failures, tooling gaps, and systemic weaknesses. People make mistakes; postmortems find the conditions that made the mistake consequential.

## Full reference

### Severity levels

| Severity | Definition | Response |
|---|---|---|
| SEV-1 | Total outage or data loss | All-hands, immediate |
| SEV-2 | Major feature down or large user subset affected | On-call + manager |
| SEV-3 | Degraded performance, workaround exists | On-call only |
| SEV-4 | Cosmetic or low-impact bug | Normal triage |

### Roles during an incident

- **Incident commander** — owns coordination and decisions, does
  not type fixes.
- **Communications lead** — owns external/stakeholder updates and
  the status page.
- **Technical lead** — owns diagnosis and mitigation.
- **Scribe** — captures the timeline in real time for the postmortem.

For small teams, one person may wear multiple hats, but the
commander role should never be combined with hands-on debugging.

### Anti-patterns

- Debugging in production when a rollback would restore service
- Going silent on stakeholders while you investigate
- Skipping the postmortem because "we know what happened"
- Shipping the fix straight to prod without normal review
- Naming individuals in the timeline or root cause
- Letting corrective actions live only in the postmortem document
  instead of the team's tracker
