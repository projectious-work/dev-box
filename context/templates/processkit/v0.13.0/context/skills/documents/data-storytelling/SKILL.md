---
name: data-storytelling
description: |
  Translates data analysis into clear narratives and recommendations that non-technical audiences can act on. Use when presenting analysis results, writing data-driven reports, creating executive summaries from dashboards, or when data needs to be communicated as a decision rather than as numbers.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-data-storytelling
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: documents
    layer: 2
---

# Data Storytelling

## Intro

Data storytelling is the discipline of turning analysis into
decisions. Numbers alone rarely change behavior — context, narrative,
and a clear recommendation do. A data story answers: what happened,
why it matters, and what to do next. The analyst's job is not to
present data; it is to reduce the decision cost for the audience.

## Overview

### The data story structure

Every data communication follows this arc:

```
Situation → Complication → Insight → Recommendation → Action
```

- **Situation** — what the world looks like (baseline, context)
- **Complication** — what changed or what problem emerged
- **Insight** — what the data reveals about why, or what it implies
- **Recommendation** — what should happen as a result
- **Action** — who does what by when

This is the Minto Pyramid adapted for data: answer first, support
second. The most common mistake is reversing the order — building
to the conclusion instead of leading with it.

### Lead with the finding, not the method

**Bad (method-first):**
> "I pulled the last 90 days of session data from the analytics
> platform, joined it with the conversion events table, and
> filtered for users in the onboarding cohort. After controlling
> for device type, I found..."

**Good (finding-first):**
> "Mobile onboarding conversions dropped 23% in Q1. The root cause
> is a single screen — the account verification step — where 61%
> of mobile users abandon. Here's what we should do."

The method belongs in an appendix, not in the lede. Executives
and stakeholders need the finding; analysts need the method.

### Choosing the right visual

| Data relationship | Best chart | Avoid |
|---|---|---|
| Trend over time | Line chart | Bar chart for many points |
| Category comparison | Bar chart (horizontal for long labels) | Pie chart |
| Part-to-whole | Stacked bar, treemap | Pie with >4 slices |
| Distribution | Histogram, box plot | Line chart |
| Correlation | Scatter plot | Line chart (implies causation) |
| Single big number | Bold number + context | Chart (overly complex) |

**The "so what" test:** every chart should have a title that states
the finding, not the topic. "Revenue by Region" is a topic;
"APAC Revenue Is Growing 3× Faster Than Other Regions" is a finding.

### Writing the narrative

**Executive summary format:**
```
**Bottom line:** [One sentence — the most important finding]

**Evidence:**
- [Finding 1] — [significance/magnitude]
- [Finding 2] — [significance/magnitude]

**Recommendation:**
- [Specific action] — **owner:** [name], **by:** [date]

**Context:** [1-2 sentences on methodology and data source, for
credibility — not detail]
```

**Slide narrative format** (one insight per slide):
- Title: the finding (claim, not topic)
- Chart: the evidence
- Annotation: the key number called out directly on the chart
- Caption: the implication ("This suggests we should...")
- Footer: data source and date range

**Report format** (for longer analysis):
```markdown
# [Report Title] — [Date Range] — [Prepared by]

## Summary (read this first)
[3-5 bullets, each a finding + implication]

## Key findings

### Finding 1: [Title — a claim]
[Chart or table]
[2-3 sentences interpreting the chart]
[Implication]

### Finding 2: [Title]
...

## Recommendations
| Recommendation | Expected impact | Owner | Timeline |
|---|---|---|---|
| [Action] | [Metric change] | [Name] | [Date] |

## Methodology
[Source, date range, definitions, caveats]
```

### Making numbers meaningful

Context transforms numbers:

**Raw number:** "Users completed 4,200 tasks last month."  
**With comparison:** "Users completed 4,200 tasks last month, up 18% from the prior month."  
**With benchmark:** "Users completed 4,200 tasks — roughly 3.5 tasks per active user, vs. our target of 5."  
**With implication:** "At the current completion rate, we'll hit our Q2 target of 20K tasks if growth holds."

Always answer: compared to what? Is this good or bad? What does
this mean for the decision at hand?

### Handling uncertainty

Data stories must acknowledge their limits:
- Name the confidence level for projections: "Based on the last 3 months, we estimate..."
- Surface data quality issues: "This metric excludes mobile app events, which represent ~15% of sessions"
- Distinguish correlation from causation: "This is associated with higher retention — we have not established causation"

Acknowledging uncertainty builds credibility; hiding it destroys it
when discovered.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Building to the conclusion instead of leading with it.** Audiences who have not yet heard the finding cannot follow a long setup. Lead with the recommendation or the most important finding, then provide the supporting evidence. "Revenue is at risk if we don't fix onboarding — here's why" beats three paragraphs of context before the punchline.
- **Using chart titles that describe the chart instead of stating the finding.** "Revenue by Quarter" tells the reader what to look at; "Revenue Growth Stalled in Q3 for the First Time in Two Years" tells them what to conclude. Every chart title should be a complete sentence that states the finding — not a label.
- **Presenting percentages without the base.** "Conversion improved by 50%" means nothing without the base — going from 2% to 3% and from 20% to 30% are very different outcomes. Always include the denominator: "from 2% to 3% (N=12,000 sessions)."
- **Using precision to signal confidence that isn't there.** Reporting "23.47% conversion rate" instead of "~23%" implies a precision the data probably does not warrant. Match significant figures to the actual certainty of the measurement; over-precision undermines credibility with sophisticated audiences.
- **Presenting data without a recommendation.** A report that describes what happened but doesn't say what to do leaves all the decision work to the audience. The analyst who understands the data is better positioned to recommend the next action than the executive who just received it. Always end with a specific recommendation and owner.
- **Treating association as causation.** Showing that users who use feature X retain better does not prove that feature X causes retention. Correlation is common; causation is hard to establish. Distinguish "users who do X have 20% higher retention" from "X drives retention." Use causal language only when the analysis actually establishes it.
- **Overloading a single slide or page with multiple findings.** A slide with three charts and six bullets gives the audience nothing to focus on. One finding per slide, one key number called out, one implication. If more findings need to be presented, use more slides — not more density per slide.

## Full reference

### Annotation patterns for charts

- **Highlight the key data point** with a contrasting color or bold label
- **Add a reference line** for targets, averages, or prior periods
- **Annotate inflection points** with what happened: "Price increase announced"
- **Call out the key number** in large text near the most important data point
- Keep all other chart elements muted (light gray grid, thin axes)

### Handling missing data

When data is incomplete:
1. State what is missing explicitly
2. Explain the impact on the finding (minor / may affect conclusion / invalidates analysis)
3. Recommend what data collection would fill the gap

Never silently exclude data that would change the conclusion.

### Statistical significance in business contexts

Business audiences rarely need p-values. Translate:
- "This is statistically significant at p < 0.05 with n=10,000" →
  "We're highly confident this is a real effect, not random noise, given the large sample"
- "Confidence interval: 18-26%" →
  "We estimate the true value is between 18% and 26% — our best point estimate is 22%"

### Linking to processkit context

Save data stories in `context/artifacts/`:
- `context/artifacts/analysis-[topic]-[date].md`

Link to the WorkItem or Decision that this analysis informed. If the
analysis produced a recommendation, create a WorkItem for the action
and link the analysis as a reference.
