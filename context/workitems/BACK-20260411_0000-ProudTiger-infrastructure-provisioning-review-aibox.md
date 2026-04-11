---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-ProudTiger-infrastructure-provisioning-review-aibox
  created: '2026-04-10T22:39:38+00:00'
  labels:
    old_id: BACK-104
    area: features
spec:
  title: Infrastructure provisioning review — aibox/DevPod boundary, Hetzner/AWS Tofu
    modules
  state: backlog
  type: task
  priority: medium
  description: 'Review context/research/infrastructure-provisioning-2026-03.md. Key
    boundary decision: does aibox own infrastructure provisioning or stay at the container
    layer? Research recommends staying at container layer + DevPod for IaC. Discuss:
    (1) Document DevPod compatibility as Phase 1. (2) Ship reference Tofu modules
    for Hetzner/AWS as Phase 2, or leave to users? (3) GPU config — aibox concern
    or DevPod concern? (4) Hetzner as cheapest path (EUR 7.49/mo for dev server).
    Old ID: BACK-104.'
---
