---
name: user-research
description: |
  Structures user research — planning sessions, synthesizing findings, and translating insights into product decisions. Use when planning user interviews, analyzing research data, creating personas or journey maps, or turning qualitative feedback into structured recommendations.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-user-research
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: product
    layer: 2
---

# User Research

## Intro

User research is the practice of studying the people who use (or
might use) a product, to reduce the risk of building the wrong
thing. Good research produces specific, evidence-backed findings —
not vague sentiments. The output of research is always a decision:
what to build, what to change, or what to stop doing.

## Overview

### Research methods at a glance

| Method | Good for | Sample size | Time |
|---|---|---|---|
| **User interviews** | Motivations, mental models, pain points | 5-8 per segment | 45-60 min each |
| **Usability testing** | Identifying friction in an existing flow | 5 per flow | 30-45 min each |
| **Survey** | Measuring prevalence, validating hypotheses | 100-500+ | Async |
| **Diary study** | Longitudinal behavior, context of use | 10-20 | 1-4 weeks |
| **Card sorting** | Information architecture | 20-30 | 20-30 min each |
| **Analytics review** | Where users go, drop off, and struggle | N/A — existing data | 1-4 hours |

For most product questions, **5 user interviews** reveals the
majority of usability issues (Nielsen's law). Add interviews until
you stop hearing new themes.

### Planning a research session

**Research brief** (write before starting):

```markdown
# Research Brief — [Topic]

**Research question:** [The one question this research will answer]
**Why now:** [What decision this informs, and when it's needed]
**Method:** [Interview / usability test / survey / etc.]
**Participants:** [Segment, recruitment criteria, number]
**Timeline:** [Sessions by date, synthesis by date, findings by date]
**Constraints:** [Budget, access, confidentiality]
```

**Discussion guide for user interviews:**
```markdown
# Interview Guide — [Topic]

## Warm-up (5 min)
- Tell me a bit about your role and how you use [product/domain].

## Context exploration (15 min)
- Walk me through the last time you [target activity].
- What were you trying to accomplish?
- What made it hard? What worked well?

## Specific probes (15 min)
- [Probe about specific hypothesis or area of interest]
- Can you show me how you do that today?

## Closing (5 min)
- Is there anything about [topic] I didn't ask that you think is important?
- Who else do you know who deals with this problem?
```

Use open questions, not leading questions. "Walk me through" and
"Tell me about" over "Would you say that..." or "Did you feel...".

### Synthesis: turning data into insights

After sessions, work through these steps:

**1. Raw notes → observations**
For each session, write observations in the format:
`[Participant] [did/said/felt] [specific thing] when [context].`
Observations are factual, not interpreted.

**2. Affinity mapping**
Group observations that express the same underlying experience.
Each cluster becomes a theme.

**3. Themes → insights**
An insight is a non-obvious conclusion drawn from multiple
observations. Format:
`[User type] [wants/needs/struggles with] [specific thing] because [underlying reason].`

**4. Insights → implications**
What should change as a result of each insight?
`Implication: [what the team should do / stop doing / investigate further]`

### Deliverables

**Research findings report** (after synthesis):
```markdown
# Research Findings — [Topic] — [Date]

## What we learned (3-5 key insights)

### Insight 1: [Title — a claim, not a topic]
**Evidence:** [Quotes or observations supporting this]
**Implication:** [What this means for the product]

### Insight 2: [Title]
...

## What we still don't know
- [Open question that emerged]
- [Area needing further research]

## Recommendations
1. [Specific, actionable recommendation with priority]

## Methodology
- [N] participants, [segment description]
- Sessions conducted [date range]
- [Method: interview / usability test / etc.]
```

**Persona (when needed):**
A persona is a composite of research findings — not a made-up character.
Only create a persona when you have enough research to make it
evidence-backed. A persona without research behind it is fiction.

```markdown
# [Persona Name] — [Role]

**Represents:** [User segment from research]

**Goals:** [What they're trying to accomplish — from research]
**Pain points:** [What frustrates them — from research]
**Behaviors:** [How they actually work — from research]
**Context:** [Environment, tools, constraints — from research]

**Representative quote:** "[Verbatim or synthesized quote that captures their view]"
```

**Journey map:**
A journey map shows a user's experience over time — not idealized,
but as it actually happens. Columns: stage, user action, thought,
feeling, pain point, opportunity.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Asking leading questions during interviews.** "Was the checkout process confusing?" primes the participant to say yes. "Walk me through what happened when you tried to check out" surfaces what they actually experienced. Review the discussion guide for leading questions before every session.
- **Synthesizing too quickly into solutions.** Moving from "users are frustrated with the checkout" directly to "we should simplify the checkout flow" skips the why. Stay in the problem space longer — understand what specifically frustrates them, and what underlying need is unmet — before implying a solution.
- **Creating personas from assumptions instead of research.** A persona built on what the team thinks users are like is not a research artifact — it's fiction with a name attached. Only create personas after conducting interviews; each attribute must trace back to a specific observation or pattern of observations.
- **Reporting what participants said without interpreting what it means.** A findings report that is a collection of quotes is not analysis. Each insight must explain WHY the observation matters and WHAT it implies — a finding without an implication is incomplete.
- **Recruiting participants who are too easy to reach.** Recruiting from your own network, your company's Slack, or existing power users skews the sample toward people who are already engaged. Recruit for the segment whose behavior you need to understand, which often means reaching people who don't already know you.
- **Confusing usability findings with feature requests.** When a participant says "I wish it had a bulk export," that's a feature request, not a finding. The underlying finding is "participants are spending significant time exporting records one at a time." Research the behavior and the pain; let the team design the solution.
- **Not closing the loop with the team.** Research that produces a report no one reads has no impact. Present findings in a session where the team can ask questions, share the report where decisions are made, and explicitly link each implication to a backlog item or decision record.

## Full reference

### Recruitment screener template

```markdown
We're looking for [N] participants for a [30/60]-minute [paid/unpaid] research session.

You qualify if you:
- [ ] [Criterion 1 — role, behavior, or experience]
- [ ] [Criterion 2]
- [ ] [Criterion 3]

You do NOT qualify if you:
- [ ] [Disqualifying criterion — e.g., work at a competitor]
```

### Consent and confidentiality

Before recording any session:
- Explain how the recording will be used
- Get explicit verbal or written consent
- Offer to continue without recording if they prefer
- Do not share clips of individual participants without their consent

### Linking to processkit context

Save research artifacts in `context/artifacts/`:
- `context/artifacts/research-brief-[topic].md`
- `context/artifacts/research-findings-[topic]-[date].md`
- `context/artifacts/persona-[name].md`

Create Note entities for quick observations captured during sessions.
Link findings to the WorkItems or DecisionRecords they informed.

### Signal-to-noise in qualitative data

| Type of data | Weight |
|---|---|
| Unprompted behavior (what they did) | Highest |
| Unprompted statement (what they said without being asked) | High |
| Response to open question | Medium |
| Response to leading question | Low |
| What they say they "would" do | Very low |

Weight observations accordingly when synthesizing. A participant
who struggled with a task unprompted is stronger evidence than one
who said "that was kind of confusing" when asked.
