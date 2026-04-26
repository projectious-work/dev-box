---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-ai-research-scientist
  created: 2026-04-22T00:00:00Z
spec:
  name: ai-research-scientist
  description: "Researches novel model architectures, training regimes, and alignment techniques."
  responsibilities:
    - "Formulate and run research hypotheses"
    - "Implement experimental training and eval code"
    - "Write up findings for internal and external audiences"
    - "Translate research into productisable signals"
  skills_required:
    - "deep-learning"
    - "scientific-writing"
    - "experiment-design"
    - "ml-theory"
    - "alignment"
  default_scope: permanent
  default_seniority: senior
  function_group: data-ml
---
