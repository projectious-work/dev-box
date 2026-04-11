---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-JollyRobin-aibox-doctor-skill-consistency
  created: '2026-04-10T22:37:35+00:00'
  labels:
    old_id: BACK-020
    area: cli
spec:
  title: aibox doctor skill consistency — check installed vs declared skills
  state: backlog
  type: task
  priority: low
  description: 'Extend `aibox doctor` to check that installed processkit skills match
    the effective skill set computed from `[context].packages` + `[skills].include`/`exclude`.
    Warn on drift. Old ID: BACK-020.'
---
