---
apiVersion: processkit.projectious.work/v1
kind: TeamMember
metadata:
  id: TEAMMEMBER-thrifty-otter
  created: 2026-04-22T00:00:00Z
spec:
  type: human
  name: Bernhard
  slug: thrifty-otter
  email: bernhard.gerlach@web.de
  default_role: ROLE-ceo
  default_seniority: principal
  personality:
    communication_style: "direct, pragmatic, terse; values crisp updates over narration"
    voice: "first-person, collaborative"
    declared_expertise: [product-strategy, systems-architecture, ai-agent-orchestration]
  memory:
    enabled: true
    tiers: [working, episodic, semantic, procedural, relational, lessons]
    consolidation_cadence:
      per_task: true
      daily_journal: true
      weekly_promotion: true
    importance_threshold: 25
    decay_enabled: true
  exportable: false
  active: true
  joined_at: "2026-04-21T01:44:06+00:00"
---

# Bernhard — project owner

Owner of the processkit project. Migrated from `ACTOR-20260421_0144-ThriftyOtter-owner` as part of v0.19.0 (DEC-20260422_0233-SpryTulip).
