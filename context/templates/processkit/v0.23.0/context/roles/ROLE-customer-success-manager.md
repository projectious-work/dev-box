---
apiVersion: processkit.projectious.work/v1
kind: Role
metadata:
  id: ROLE-customer-success-manager
  created: 2026-04-22T00:00:00Z
spec:
  name: customer-success-manager
  description: "Drives adoption, retention, and expansion post-sale."
  responsibilities:
    - "Own onboarding and time-to-value for accounts"
    - "Run regular health checks and business reviews"
    - "Identify expansion and reduce churn"
    - "Advocate customer voice internally"
  skills_required:
    - "account-management"
    - "onboarding"
    - "business-reviews"
    - "churn-prevention"
  default_scope: permanent
  default_seniority: senior
  function_group: sales-customer
---
