---
name: incident-response
description: Guides production incident handling — triage, communicate, fix, postmortem. Use when something is broken in production.
---

# Incident Response

## When to Use

When the user reports a production issue, outage, or says "production is down", "users are affected", "we have an incident", or describes urgent production problems.

## Instructions

1. **Triage (first 5 minutes):**
   - What is the user impact? (total outage, degraded, cosmetic)
   - What changed recently? (deploy, config change, dependency update)
   - Is there a quick rollback option?
2. **Communicate:**
   - Notify stakeholders immediately with: what's broken, who's affected, ETA to fix
   - Update status page or team channel
3. **Mitigate first, fix later:**
   - Rollback if safe and quick
   - Apply a temporary workaround to restore service
   - Don't debug in production if you can reproduce elsewhere
4. **Fix:**
   - Identify root cause
   - Apply and test the fix
   - Deploy with extra monitoring
5. **Postmortem (within 48 hours):**
   - Timeline of events
   - Root cause analysis
   - What went well, what didn't
   - Action items to prevent recurrence
   - No blame — focus on systems and processes

## Examples

**User:** "Our API is returning 500 errors after the last deploy"
**Agent:** Checks the deploy diff, identifies the breaking change, recommends immediate rollback while preparing a fix, drafts a stakeholder update.
