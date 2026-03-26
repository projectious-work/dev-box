# AI Provider Integrations — Addon Candidates Research — March 2026

Research for BACK-043. Evaluates which additional AI coding agents and providers should be
added as aibox addons. Covers installation method, config directory, binary name, maturity
level, license, and a recommendation for each candidate. Conducted 2026-03-26.

---

## Current State

The `addons/ai/` directory already ships four AI provider addons:

| Addon name | Tool | Install method | Status |
|---|---|---|---|
| `ai-claude` | Claude Code CLI | `npm install -g @anthropic-ai/claude-code` | Stable, GA |
| `ai-gemini` | Gemini CLI | `npm install -g @google/gemini-cli` | Stable |
| `ai-aider` | Aider | `uv tool install aider-chat` | Stable |
| `ai-mistral` | mistralai Python SDK | `pip install mistralai` | SDK-only (no CLI) |

The `ai-mistral` addon installs the Python SDK, not an interactive coding CLI — it is qualitatively
different from the others and should be re-evaluated separately (see note at end).

---

## Candidates Evaluated

### 1. OpenAI Codex CLI

**Summary:** OpenAI's official open-source terminal coding agent. Launched April 2025,
now the core of OpenAI's coding platform spanning terminal, IDE extension, cloud environment,
and mobile. Built in Rust.

| Field | Detail |
|---|---|
| Install | `npm install -g @openai/codex` (also: Homebrew `brew install --cask codex`, or binary download) |
| Binary name | `codex` |
| Config directory | `~/.codex/` (config: `~/.codex/config.toml`, rules: `~/.codex/rules/`, sessions: `~/.codex/sessions/`) |
| License | Open source (MIT) |
| Maturity | GA — actively developed, used by millions |
| Platform | macOS, Linux (Windows experimental) |
| Auth | OpenAI account (ChatGPT Plus/Pro/Team/Edu/Enterprise) |
| NPM package | `@openai/codex` |

**Assessment:** Strong candidate. npm-based install fits aibox's existing pattern (same as
`ai-claude`, `ai-gemini`). Open source, actively maintained, large user base. Provider parity
with Claude and Gemini. Requires `OPENAI_API_KEY` or ChatGPT login.

**Recommendation: Add as `ai-openai` addon.**

---

### 2. GitHub Copilot CLI

**Summary:** GitHub's terminal-native coding agent. Went GA February 2026 after a public
preview starting September 2025. Tight GitHub integration (issues, PRs, repos). Ships as
part of the GitHub Codespaces default image and as a Dev Container Feature.

| Field | Detail |
|---|---|
| Install | `npm install -g @github/copilot` (also: Homebrew, install script, binary download) |
| Binary name | `copilot` |
| Config directory | `~/.copilot/` (config: `~/.copilot/config.json`; overridable via `COPILOT_HOME`) |
| License | Proprietary (GitHub Copilot subscription required) |
| Maturity | GA (February 2026) |
| Platform | macOS, Linux, Windows |
| Auth | GitHub account with active Copilot subscription |
| NPM package | `@github/copilot` |

**Assessment:** Strong candidate. npm install matches existing pattern. GA status and large
installed base. The Copilot subscription requirement is a gating factor, but the same applies
to `ai-claude` (requires Anthropic account). GitHub's release of this as a Dev Container
Feature is explicit validation that it belongs in devcontainer-style setups. Requires auth
on first launch — aibox users manage this via environment variables in the standard pattern.

**Recommendation: Add as `ai-copilot` addon.**

---

### 3. Continue.dev CLI (`cn`)

**Summary:** Open-source, modular coding agent CLI. The Continue project (originally a VS Code
extension) launched a standalone terminal CLI (`cn`) built for headless environments, CI/CD,
and Docker containers. Provider-agnostic: works with any model (Anthropic, OpenAI, Gemini,
local models via Ollama).

| Field | Detail |
|---|---|
| Install | `npm install -g @continuedev/cli` (also: `curl` install script) |
| Binary name | `cn` |
| Config directory | `~/.continue/` (standard Continue config location) |
| License | Open source (Apache 2.0) |
| Maturity | Active development; headless/CLI mode relatively new (2025) |
| Platform | macOS, Linux, Windows |
| Auth | API key for configured provider (set via `CONTINUE_API_KEY` for headless) |
| NPM package | `@continuedev/cli` |

**Assessment:** Moderate candidate. Excellent fit for Docker/headless use — headless mode
(`cn -p "prompt"`) was designed explicitly for containers and CI. Provider-agnostic design
aligns with aibox's provider-independence rule. The CLI is newer and less battle-tested than
Codex or Copilot. The binary name `cn` is short and collision-prone.

**Recommendation: Add as `ai-continue` addon — lower priority than Codex/Copilot.**

---

### 4. Kiro CLI (formerly Amazon Q Developer CLI)

**Summary:** AWS's terminal AI coding agent. Rebranded from Amazon Q Developer CLI to Kiro CLI
in November 2025. Positioned as "spec-driven development" — converts prompts to specs, then
to code, docs, and tests. AWS-ecosystem focused (tight integration with AWS services and Bedrock).

| Field | Detail |
|---|---|
| Install | `curl` download + `./install.sh` (zip from AWS CDN); no npm/pip |
| Binary name | `q` (legacy compat) and `kiro` |
| Config directory | `~/.kiro/` (inferred from Kiro docs); legacy `~/.aws/amazonq/` |
| License | Proprietary (AWS account required; free tier available) |
| Maturity | GA; Q CLI was stable before rebrand |
| Platform | macOS, Linux x86_64 + ARM64 (no native Windows) |
| Auth | AWS account (IAM Identity Center or Builder ID) |
| NPM package | None — binary distribution only |

**Assessment:** Weaker candidate for aibox. The curl-pipe-sh install pattern is exactly what
the existing `ai-claude` notes are moving away from (see comment in `ai-claude.yaml` explaining
why npm is used instead of the native installer). No npm/pip install available increases
implementation complexity (must pin a specific release URL or version). AWS-centric audience
overlap with aibox users is lower than OpenAI/GitHub. The rebrand to Kiro adds noise.

**Recommendation: Defer. Document as known candidate. Revisit if Kiro publishes an npm/apt
package. Not worth the curl-install complexity at this stage.**

---

### 5. Cline

**Summary:** Open-source autonomous coding agent for VS Code (and forks — Cursor, Windsurf,
JetBrains). Cline is primarily an IDE extension with Plan/Act modes and MCP integration.
As of March 2026, Cline does not ship a standalone terminal CLI — it requires an IDE host.

| Field | Detail |
|---|---|
| Install | VS Code extension (`cline.bot` marketplace ID) |
| Binary name | No standalone CLI binary |
| Config directory | VS Code extension storage |
| License | Open source (Apache 2.0) |
| Maturity | Stable, widely used |
| Platform | Wherever VS Code runs |

**Assessment:** Not suitable as an aibox addon in its current form. Aibox addons install
CLI tools into containers; Cline's architecture requires a VS Code process as host. If a
standalone Cline CLI emerges, re-evaluate. The VS Code extension itself is available in
the container via VS Code server or as a devcontainer extension — but that is a different
mechanism from addon Dockerfiles and is already supported via `devcontainer.json` extensions
configuration.

**Recommendation: Exclude. Not a CLI tool. Track for future standalone CLI release.**

---

### 6. OpenCode

**Summary:** Open-source terminal AI coding agent built in Go with a full TUI (Bubble Tea).
Provider-agnostic: supports OpenAI, Anthropic, Gemini, AWS Bedrock, Groq, Azure OpenAI,
OpenRouter, and local models. Over 5M monthly users claimed.

| Field | Detail |
|---|---|
| Install | `npm install -g opencode-ai@latest` (also: `curl` install script, Homebrew, bun) |
| Binary name | `opencode` |
| Config directory | `~/.config/opencode/` (Go XDG convention) |
| License | Open source (MIT) |
| Maturity | Active, growing rapidly (2025) |
| Platform | macOS, Linux, Windows |
| Auth | Per-provider API keys; configured on first launch |
| NPM package | `opencode-ai` |

**Assessment:** Interesting candidate. The provider-agnostic model is a strong fit for aibox
users who want flexibility across LLM providers. npm install is simple. However, OpenCode
duplicates much of what `ai-claude`, `ai-gemini`, and `ai-openai` individually provide.
Its primary value-add is the multi-provider TUI in a single tool. Naming is slightly
confusing alongside OpenAI Codex.

**Recommendation: Add as `ai-opencode` addon at lower priority. Good for power users who
want a single multi-provider terminal agent without committing to one vendor.**

---

### 7. Sourcegraph Cody / Amp

**Summary:** Sourcegraph's AI coding tool. As of 2025, the free/Pro self-serve plans were
discontinued; Cody CLI is experimental and enterprise-only (requires a Sourcegraph enterprise
account). The product has been renamed to Amp in some contexts. npm package exists
(`@sourcegraph/cody`).

| Field | Detail |
|---|---|
| Install | npm (`@sourcegraph/cody`) or binary |
| Binary name | `cody` |
| Config directory | Sourcegraph enterprise instance config |
| License | Proprietary (enterprise only) |
| Maturity | Experimental CLI; enterprise-gated |

**Assessment:** Not suitable. The combination of enterprise-only access, experimental CLI
status, and discontinued self-serve plans makes this a poor fit for the aibox addon pattern,
which targets individual developers and small teams.

**Recommendation: Exclude. Revisit if Cody/Amp re-launches a self-serve CLI tier.**

---

## Summary Table

| Candidate | Addon name | Priority | Reason |
|---|---|---|---|
| OpenAI Codex CLI | `ai-openai` | High | GA, npm, open source, provider parity |
| GitHub Copilot CLI | `ai-copilot` | High | GA, npm, Dev Container validated, large user base |
| Continue CLI | `ai-continue` | Medium | Docker-native headless mode, provider-agnostic, Apache 2.0 |
| OpenCode | `ai-opencode` | Low | Multi-provider TUI, npm, open source |
| Kiro CLI | — | Defer | curl-only install, AWS-centric, no npm/apt |
| Cline | — | Exclude | No standalone CLI; IDE extension only |
| Sourcegraph Cody/Amp | — | Exclude | Enterprise-only, experimental CLI |

---

## Implementation Notes

### `ai-openai` — OpenAI Codex CLI

```yaml
name: ai-openai
runtime: |
  # Addon: ai-openai
  # OpenAI Codex CLI: https://github.com/openai/codex
  # npm package: @openai/codex
  USER aibox
  RUN npm install -g @openai/codex
  USER root
```

Config directory: `~/.codex/` — no volume mount needed for basic use; users provide
`OPENAI_API_KEY` as an environment variable.

### `ai-copilot` — GitHub Copilot CLI

```yaml
name: ai-copilot
runtime: |
  # Addon: ai-copilot
  # GitHub Copilot CLI: https://github.com/github/copilot-cli
  # npm package: @github/copilot
  # Requires a GitHub Copilot subscription.
  USER aibox
  RUN npm install -g @github/copilot
  USER root
```

Config directory: `~/.copilot/` — auth is interactive on first launch (`copilot /login`).

### `ai-continue` — Continue CLI

```yaml
name: ai-continue
runtime: |
  # Addon: ai-continue
  # Continue CLI: https://github.com/continuedev/continue
  # npm package: @continuedev/cli
  # Provider-agnostic. Headless mode: cn -p "prompt"
  USER aibox
  RUN npm install -g @continuedev/cli
  USER root
```

Config directory: `~/.continue/` — API key via `CONTINUE_API_KEY` for headless use.

---

## Note on `ai-mistral`

The existing `ai-mistral` addon installs the `mistralai` Python SDK, not an interactive
coding CLI. This is categorically different from all other `ai-*` addons. Options:

1. Keep as-is and add a clarifying comment — it is an SDK addon for programmatic use.
2. Rename to `sdk-mistral` and create a separate SDK addon category.
3. Remove and replace with a future Mistral CLI if one emerges (Le Chat CLI is in development).

This decision is out of scope for BACK-043 and should be tracked separately.

---

## Sources

- [OpenAI Codex CLI — npm](https://www.npmjs.com/package/@openai/codex)
- [OpenAI Codex CLI — GitHub](https://github.com/openai/codex)
- [OpenAI Codex CLI — Config Reference](https://developers.openai.com/codex/config-reference)
- [GitHub Copilot CLI — Generally Available](https://github.blog/changelog/2026-02-25-github-copilot-cli-is-now-generally-available/)
- [GitHub Copilot CLI — Install Docs](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/install-copilot-cli)
- [GitHub Copilot CLI — GitHub](https://github.com/github/copilot-cli)
- [Continue CLI Docs](https://docs.continue.dev/cli/overview)
- [Continue CLI — npm](https://www.npmjs.com/package/@continuedev/cli)
- [Continue CLI — GitHub](https://github.com/continuedev/continue)
- [Kiro CLI — Installation](https://kiro.dev/docs/cli/installation/)
- [Kiro CLI — Migrating from Amazon Q](https://kiro.dev/docs/cli/migrating-from-q/)
- [Cline — GitHub](https://github.com/cline/cline)
- [OpenCode — GitHub](https://github.com/opencode-ai/opencode)
- [OpenCode — npm](https://opencode.ai/download)
- [Sourcegraph Cody CLI — Install](https://sourcegraph.com/docs/cody/clients/install-cli)
