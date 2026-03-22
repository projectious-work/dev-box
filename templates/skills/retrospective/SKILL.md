---
name: retrospective
description: Facilitates team or project retrospectives — what worked, what didn't, action items. Use at the end of a sprint, milestone, or project phase.
---

# Retrospective

## When to Use

When the user says "let's do a retro", "what went well?", "lessons learned", "end of sprint review", or wants to reflect on a completed phase of work.

## Instructions

1. **Set the scope:** What time period or milestone are we reflecting on?
2. **Gather input in three categories:**
   - **What worked well:** Practices, tools, decisions that helped
   - **What didn't work:** Pain points, blockers, things that slowed us down
   - **What to try next:** Concrete experiments or changes for the next iteration
3. **Format as a structured document:**
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
4. **Action items must be:**
   - Specific (not "improve testing" but "add integration tests for auth module")
   - Assignable (someone owns it)
   - Time-bound (done by when?)
5. Store the retrospective in context/ or the project's designated location

## Examples

**User:** "Let's do a retro on the v0.3 release"
**Agent:** Asks what went well and what was painful, then structures findings into the retro format with concrete action items. Saves to `context/retros/v0.3.md`.
