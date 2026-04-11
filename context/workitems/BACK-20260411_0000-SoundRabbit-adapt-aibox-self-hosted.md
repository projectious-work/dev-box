---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-SoundRabbit-adapt-aibox-self-hosted
  created: '2026-04-10T22:36:10+00:00'
  labels:
    old_id: BACK-122
    area: cli
spec:
  title: Adapt aibox + self-hosted devcontainer after processkit v0.8.0 overhaul
  state: backlog
  type: task
  priority: critical
  description: 'processkit has undergone a major structural overhaul (v0.8.0). Two
    sub-tasks: (a) audit aibox CLI and scaffolding for required adaptations — content
    install pipeline, three-way diff, lock schema, addon/skill references, and any
    processkit API surface changes; (b) migrate this repo''s own devcontainer (.devcontainer/)
    and context/ to the new processkit setup, since both are managed by aibox + processkit.
    Track processkit v0.8.0 changes before actioning either sub-task. Coordinate with
    the pre-install three-way diff work item (BACK-120) which may interact with the
    new install contract. THIS IS THE PRIMARY CURRENT TASK — the processkit reset
    was just performed and aibox CLI needs to be brought into alignment. Old ID: BACK-122.'
---
