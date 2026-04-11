---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-BoldCrow-ai-provider-audit-logging
  created: 2026-04-11
spec:
  title: "AI Provider Audit Logging Capabilities"
  type: reference
  state: permanent
  tags: [audit-logging, ai-providers, hooks, opentelemetry, event-log, hybrid-model]
  skill: research-with-confidence
  source_file: ai-provider-audit-logging-2026-03.md
---

# AI Provider Audit Logging Capabilities

**Date:** 2026-03-28
**Relates to:** DISC-001, event-log-management skill design
**Purpose:** Determine whether AI coding assistants provide deterministic audit logging that aibox could integrate with for hybrid (deterministic + probabilistic) event capture.

---

## Key Finding

Most major AI coding providers offer some form of audit logging, but capabilities vary dramatically. Two clear patterns emerge:

1. **Hooks/OTel providers** (Claude Code, Gemini CLI, Codex CLI): Full event capture including prompts, responses, tool calls, file modifications. Can push to external destinations via webhooks or OpenTelemetry backends.

2. **Enterprise-only providers** (GitHub Copilot, Cursor, Windsurf, Amazon Q): Audit logging gated behind enterprise tiers. Often exclude prompt/response content.

---

## Comparison Table

| Provider | Audit Logging | Webhook/External | Captures Prompts | Tool Calls | All Tiers? |
|---|---|---|---|---|---|
| **Claude Code** | Yes (hooks + JSONL) | Yes (HTTP hooks) | Yes | Yes (21 events) | Yes |
| **Gemini CLI** | Yes (OTel + Cloud Audit) | Yes (any OTel backend) | Yes | Yes | Yes |
| **Codex CLI** | Yes (OTel) | Yes (any OTel backend) | Yes | Yes | Yes |
| **Amazon Q Dev** | Yes (CloudTrail) | Yes (EventBridge) | No (hidden) | Yes | Pro only |
| **GitHub Copilot** | Yes (enterprise) | No native webhook | No (excluded) | Yes (agent sessions) | Enterprise only |
| **Windsurf** | Yes (enterprise DB) | Enterprise self-hosted | Yes | Unknown | Enterprise only |
| **Aider** | Partial (file-based) | No | Yes (.md history) | Partial (git commits) | Yes (OSS) |
| **Cursor** | Limited (enterprise) | No | No | No | Enterprise only |
| **Continue.dev** | Partial (PostHog) | No | Yes (internal logs) | No | Enterprise policies |

---

## Implication for aibox

**Three logging channels available:**

1. **Provider hooks (deterministic):** Claude Code hooks, Gemini OTel, Codex OTel can capture every interaction automatically. Always logs if configured.

2. **Agent event-log skill (probabilistic):** Agent uses event-log-management skill to record process events (state changes, decisions, gate checks). Best-effort.

3. **aibox sync infrastructure events (deterministic):** `aibox sync` and `aibox lint` record infrastructure events.

**Recommended `[audit]` section in `aibox.toml`:**

```toml
[audit]
provider_hooks = true
provider_destination = "context/audit/"
event_log = "context/events/"
sync_log = "context/events/"
```

When `provider_hooks = true`, `aibox init` configures the active provider's hooks to capture tool calls and file modifications.

**The hybrid model:**
- Provider hooks capture **WHAT** happened (every tool call, every file edit) — deterministic
- Agent event-log captures **WHY** it happened (state changes, decisions, rationale) — probabilistic
- Together: "what" is guaranteed, "why" is best-effort

---

## Emerging Standards

- **OpenTelemetry GenAI semantic conventions** — emerging standard for AI agent telemetry. Gemini CLI and Codex CLI already use it. Could become the universal format.
- No universal standard yet for AI coding assistant audit log format.
- **Third-party governance middleware** (MintMCP Gateway, Oasis) emerging as tool-agnostic audit layers.
