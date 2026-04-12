---
name: estimation-planning
description: |
  Software estimation and planning — story points, velocity, scope negotiation, technical debt budgeting, and statistical forecasting. Use when estimating effort, planning a sprint, negotiating scope with stakeholders, budgeting for technical debt, or forecasting completion dates.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-estimation-planning
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: product
    layer: 3
---

# Estimation and Planning

## Intro

Estimation is uncertainty management, not prediction. Use relative
sizing for stories, track velocity over many sprints, and present
forecasts as ranges. When dates and scope conflict, negotiate one;
never fix both.

## Overview

### Story points vs time estimates

Story points measure relative complexity, accounting for effort,
uncertainty, and risk in one number. Use the Fibonacci sequence
(1, 2, 3, 5, 8, 13) — the gaps reflect rising uncertainty at
larger sizes. A "1" is the simplest well-understood task on the
team; everything is relative to it. Stories of 13+ must be split
before estimation.

Time estimates (hours/days) are appropriate for fixed-scope
contracts, individual task tracking, or teams that prefer them.
Neither approach is universally superior. Never convert story
points to hours — it defeats the purpose of relative sizing.

### Planning poker

Each estimator independently picks a card to prevent anchoring.
Discuss outliers (highest and lowest explain themselves), then
re-vote. Converge within two rounds or take the higher estimate.
Sessions: 60-90 minutes, 15-20 stories max. The person doing the
work does not get extra weight — estimation is a team activity.
For remote teams, run async estimation first, then discuss
outliers live.

### Cone of uncertainty

Early estimates are inherently inaccurate: ~4x range at project
start, ~1.25x at late stages. Communicate as ranges ("3-5
sprints"), not single numbers. Re-estimate as uncertainty
decreases — after design, after a spike, after the first
iteration. Never promise a single date at project start.

### Velocity tracking

Velocity = story points completed per sprint, counting only fully
done stories. Track at least 5-6 sprints before forecasting. Use
the average of the last 3-5 sprints for planning, not the best
sprint. Velocity is a planning tool, not a performance metric;
never compare teams. When velocity drops, investigate (scope
changes, interruptions, debt, team changes) — don't assume it
will recover.

### Scope negotiation

Present scope as a menu with trade-offs ("A+B by March, or A+B+C
by April"). Use MoSCoW: Must-have, Should-have, Could-have,
Won't-have (this time). Fixed date? Negotiate scope. Fixed scope?
Negotiate date. Never fix both. Identify the MVP — the smallest
delivery that provides value. Document descoped items and the
reason, to prevent later scope creep. Revisit scope at sprint
boundaries, not mid-sprint.

### Technical debt budgeting

Allocate 15-20% of each sprint to debt and maintenance. Track
debt items in the same backlog as features so they compete for
priority. Categorize debt as critical (blocks future work),
important (slows development), or minor (cosmetic). Pay critical
debt immediately; schedule important debt each sprint. Frame debt
for stakeholders in business terms ("this refactor will reduce
bug rate by ~30%"). Never let debt accumulate silently.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Single-number estimates at project start.** The cone of uncertainty is ~4x at project start — committing to a single date this early guarantees being wrong. Always present ranges; re-estimate as unknowns resolve.
- **Promising fixed date AND fixed scope simultaneously.** When both are fixed, the only variable left is quality, and quality always loses. When stakeholders want a fixed date, negotiate scope; when they want fixed scope, negotiate the date. Never fix both.
- **Converting story points to hours.** Story points measure relative complexity across a team; hours measure elapsed time for an individual. Converting destroys the relative signal and creates false precision. If hours are needed, use hours-based estimation from the start.
- **Comparing velocity across teams.** Each team calibrates its "1-point" differently. Cross-team velocity comparison is not meaningful and creates perverse pressure to inflate estimates.
- **Padding estimates secretly.** Hidden contingency destroys trust when discovered and eliminates the feedback loop that improves future estimates. Surface the risk explicitly: "This estimate includes 20% buffer for the unknown authentication integration."
- **Re-estimating done work to match actuals.** Retroactively adjusting estimates to make the velocity look better destroys the historical calibration that makes forecasting useful. Leave done work at its original estimate.
- **Treating velocity as a performance metric.** Velocity is a planning tool, not a KPI. Using it to compare or rank engineers causes gaming, inflated estimates, and team dysfunction.

## Full reference

### Three-point estimation (PERT)

For each task, estimate Optimistic (O), Most Likely (M), and
Pessimistic (P).

- **PERT estimate:** `(O + 4*M + P) / 6`
- **Standard deviation:** `(P - O) / 6`

Use for project-level estimates where individual task uncertainty
compounds. Pessimistic should assume things go wrong, but not
catastrophically. Sum the PERT estimates for project total; sum
the variances and take the square root for project uncertainty.

### Monte Carlo simulation

Inputs: historical velocity data (points per sprint) and total
backlog size. Run 1000+ simulations: randomly sample a velocity
for each sprint, count sprints to complete. Output: a probability
distribution of completion dates.

Report as: "80% chance of completing by Sprint 12, 95% chance by
Sprint 14." More credible than single-point estimates because it
uses real historical variation. Re-run as the backlog or velocity
data changes.

### Worked examples

**"Stakeholder wants all 50 stories done by end of quarter."**
Compute based on the team's average velocity (last 5 sprints) and
remaining sprints. If velocity is 25 points/sprint and the 50
stories total 180 points, that's 7.2 sprints — but only 5
remain. Present options: descope to 120 points of must-haves
(fits in 5), extend by 2 sprints, or add capacity. Use MoSCoW to
identify what's actually must-have.

**"Our estimates are always wrong."** Review recent sprints for
patterns: stories consistently under-estimated (8-pointers
spilling across sprints), mid-sprint scope creep, missing tasks.
Recommend: split stories to 5 points or less, include code
review and testing time in estimates, track estimate-vs-actual
for calibration, use planning poker to surface hidden complexity.

**"When will this project be done? We have 400 points in the
backlog."** Run a three-point estimate: with velocity 20-35
points/sprint (O=12, M=16, P=25 sprints), PERT estimate is 17
sprints. Present as a range: "14-20 sprints with 80% confidence."
Offer Monte Carlo for a more precise forecast using actual
velocity history.

### Anti-patterns

- **Single-number estimates at project start** — always a range.
- **Comparing velocity across teams** — different baselines, not
  comparable.
- **Padding estimates secretly** — surface the risk explicitly
  instead.
- **Re-estimating done work to match actuals** — destroys the
  feedback loop that improves future estimates.
- **Treating story points as currency** — they exist to compare
  stories within a team, not to bill against.
