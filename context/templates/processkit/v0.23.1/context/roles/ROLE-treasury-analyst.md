---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-treasury-analyst
  created: 2026-04-22T00:00:00Z
spec:
  name: treasury-analyst
  description: "Manages cash, liquidity, FX, and short-term investments."
  responsibilities:
    - "Forecast and manage cash positions"
    - "Manage banking relationships and facilities"
    - "Execute FX and hedging strategies"
    - "Optimise short-term investments"
  skills_required:
    - "cash-management"
    - "fx-hedging"
    - "liquidity-planning"
    - "banking-ops"
  default_scope: permanent
  default_seniority: senior
  function_group: finance
---
