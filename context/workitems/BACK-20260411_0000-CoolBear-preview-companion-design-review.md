---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-CoolBear-preview-companion-design-review
  created: '2026-04-10T22:36:27+00:00'
  labels:
    old_id: BACK-078
    area: features
    project: PROJ-004
spec:
  title: Preview companion design review — in-container vs companion container decision
  state: backlog
  type: task
  priority: high
  description: 'Review context/research/preview-companion-design-2026-03.md. Two architectures
    to reconcile: (A) in-container tools (chafa, timg, mupdf-tools — already in base
    image, character-art quality) vs (B) companion container with web server + browser
    preview (pixel-perfect quality). Decide: keep both (hybrid), or companion-only?
    Discuss excalidraw file-bridge, SSH port forwarding UX, LaTeX edit→preview cycle.
    Decide on MVP scope. Old ID: BACK-078, part of PROJ-004.'
---
