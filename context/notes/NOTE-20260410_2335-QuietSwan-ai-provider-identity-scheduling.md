---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-QuietSwan-ai-provider-identity-scheduling
  created: 2026-04-11
spec:
  title: "AI Provider Identity, Scheduling, and Multi-User Collaboration"
  type: reference
  state: permanent
  tags: [identity, scheduling, multi-user, kubernetes, collaboration, ai-providers]
  skill: research-with-confidence
  source_file: ai-provider-identity-scheduling-2026-03.md
---

# AI Provider Identity, Scheduling, and Multi-User Collaboration

**Date:** 2026-03-28
**Relates to:** DISC-001 §2.48 (identity resolution), §2.49 (scenario 5, 7)

---

## 1. Identity Mechanisms by Provider

| Tool | Identity Source | Auth Method | Extractable? |
|------|----------------|-------------|--------------|
| Claude Code | Anthropic account | OAuth + keychain | No (locked down) |
| Gemini CLI | Google account | OAuth / API key / ADC | Yes |
| Copilot CLI | GitHub account | OAuth device flow | Yes (`gh api user`) |
| Aider | None (provider key) | API keys only | No identity exists |
| Codex CLI | OpenAI/ChatGPT | Browser OAuth / API key | Partially |
| Self-hosted | None by default | None / reverse proxy | N/A |

---

## 2. Kubernetes Identity Patterns (applicable to aibox)

- **kubeconfig** → `~/.aibox/identity.toml` (local, never committed)
- **`kubectl auth whoami`** → `aibox auth whoami` (shows identity + provider)
- **Separation of authn/authz** → identity (who) separate from RBAC (what can you do)
- **Service accounts** → for CI/CD and scheduled tasks, distinct from human identity

---

## 3. Scheduling Capabilities

| Tool | Native Scheduling | Mechanism | Recurring? |
|------|-------------------|-----------|-----------|
| Claude Code | Yes | `/loop`, desktop schedules, cloud | Yes |
| Gemini CLI | Via cron | Headless + system cron | Yes |
| Gemini Enterprise | Yes | Agent Designer schedules | Yes |
| Copilot Agent | No (event-driven) | Issue assignment | No |
| Codex App | Yes | Automations (web) | Yes |
| Codex CLI | Planned | Not yet | -- |
| Aider | No | -- | -- |

---

## 4. Multi-Human Repo — Recommended Pattern

**External identity file** (`~/.aibox/identity.toml`) — analogous to `~/.gitconfig`.
Contains name, email, preferences. Never committed to repos.

Actor files in the repo contain non-sensitive shared info (name, expertise, roles).
Private preferences stay in `~/.aibox/`. This aligns with the Kubernetes kubeconfig
pattern and avoids all privacy concerns.
