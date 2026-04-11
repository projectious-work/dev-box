---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-SharpQuail-ai-provider-integrations-addon
  created: 2026-04-11
spec:
  title: "AI Provider Integrations — Addon Candidates Research"
  type: reference
  state: permanent
  tags: [ai-providers, addons, codex, copilot, continue, opencode, research]
  skill: research-with-confidence
  source_file: ai-provider-integrations-2026-03.md
---

# AI Provider Integrations — Addon Candidates Research — March 2026

**Task:** BACK-043
**Date:** 2026-03-26

Research evaluating which additional AI coding agents and providers should be added as aibox addons. Covers installation method, config directory, binary name, maturity level, license, and recommendation for each candidate.

---

## Current State

The `addons/ai/` directory already ships four AI provider addons:

| Addon name | Tool | Install method | Status |
|---|---|---|---|
| `ai-claude` | Claude Code CLI | `npm install -g @anthropic-ai/claude-code` | Stable, GA |
| `ai-gemini` | Gemini CLI | `npm install -g @google/gemini-cli` | Stable |
| `ai-aider` | Aider | `uv tool install aider-chat` | Stable |
| `ai-mistral` | mistralai Python SDK | `pip install mistralai` | SDK-only (no CLI) |

Note: `ai-mistral` installs a Python SDK, not an interactive coding CLI — qualitatively different from others.

---

## Summary of Candidates

| Candidate | Addon name | Priority | Reason |
|---|---|---|---|
| OpenAI Codex CLI | `ai-openai` | **High** | GA, npm, open source, provider parity |
| GitHub Copilot CLI | `ai-copilot` | **High** | GA, npm, Dev Container validated, large user base |
| Continue CLI | `ai-continue` | Medium | Docker-native headless mode, provider-agnostic, Apache 2.0 |
| OpenCode | `ai-opencode` | Low | Multi-provider TUI, npm, open source |
| Kiro CLI | — | Defer | curl-only install, AWS-centric, no npm/apt |
| Cline | — | Exclude | No standalone CLI; IDE extension only |
| Sourcegraph Cody/Amp | — | Exclude | Enterprise-only, experimental CLI |

---

## Detailed Evaluations

### 1. OpenAI Codex CLI → `ai-openai` (HIGH)

- **Install:** `npm install -g @openai/codex`; also Homebrew, binary download
- **Binary:** `codex`; **Config:** `~/.codex/` (config.toml, rules/, sessions/)
- **License:** MIT; **Auth:** OpenAI account (ChatGPT Plus/Pro/Team/Enterprise)
- **Assessment:** Strong candidate. npm-based install fits aibox pattern. Open source, actively maintained, large user base. Requires `OPENAI_API_KEY` or ChatGPT login.

### 2. GitHub Copilot CLI → `ai-copilot` (HIGH)

- **Install:** `npm install -g @github/copilot`; also Homebrew, install script
- **Binary:** `copilot`; **Config:** `~/.copilot/config.json` (overridable via `COPILOT_HOME`)
- **License:** Proprietary; **Auth:** GitHub account with Copilot subscription; **GA:** February 2026
- **Assessment:** GA, large installed base. GitHub released this as a Dev Container Feature — explicit validation for devcontainer-style setups. Interactive auth on first launch.

### 3. Continue CLI → `ai-continue` (Medium)

- **Install:** `npm install -g @continuedev/cli`; also curl install script
- **Binary:** `cn`; **Config:** `~/.continue/`
- **License:** Apache 2.0; **Auth:** Per-provider API key via `CONTINUE_API_KEY`
- **Assessment:** Excellent fit for Docker/headless use (headless mode designed explicitly for containers and CI). Provider-agnostic aligns with aibox's provider-independence rule. CLI is newer and less battle-tested. Binary name `cn` is collision-prone.

### 4. OpenCode → `ai-opencode` (Low)

- **Install:** `npm install -g opencode-ai@latest`; also curl, Homebrew, bun
- **Binary:** `opencode`; **Config:** `~/.config/opencode/` (XDG)
- **License:** MIT; **Auth:** Per-provider API keys; configured on first launch
- **Assessment:** Provider-agnostic multi-provider TUI is a strong fit. Duplicates individual provider addons but provides single multi-provider interface. Good for power users who want flexibility.

### 5. Kiro CLI (Defer)

Formerly Amazon Q Developer CLI. curl-only install (no npm/pip), AWS-centric audience, rebrand noise. Revisit if Kiro publishes npm/apt package.

### 6. Cline (Exclude)

VS Code extension only — no standalone CLI binary. Not suitable for aibox addon pattern. Track for future standalone CLI release.

### 7. Sourcegraph Cody/Amp (Exclude)

Enterprise-only, experimental CLI, discontinued self-serve plans. Poor fit for individual developers and small teams.

---

## Implementation Sketch

```yaml
# ai-openai
RUN npm install -g @openai/codex

# ai-copilot
RUN npm install -g @github/copilot

# ai-continue
RUN npm install -g @continuedev/cli
```

## Note on `ai-mistral`

The existing addon installs a Python SDK, not an interactive coding CLI. Options: keep with clarifying comment, rename to `sdk-mistral`, or remove and replace with future Mistral CLI (Le Chat CLI in development). Out of scope for BACK-043.
