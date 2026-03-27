# Self-Hosted AI Models & GPU Providers — Research Report — March 2026

Research for BACK-030 (bring-your-own-model support) and BACK-094 (self-hosting research).
Evaluates GPU infrastructure providers, model serving frameworks, open-weight coding models,
coding agent compatibility, and integration options for aibox. Conducted 2026-03-26.

---

## 1. GPU Infrastructure Providers

### Comparison Table

| Provider | H100 $/hr | A100 80GB $/hr | Min commitment | Docker support | Provisioning API | Best for |
|---|---|---|---|---|---|---|
| **Vast.ai** | ~$1.87 | ~$1.20 | None (spot) | Native Docker | CLI, Python SDK, REST API | Cheapest spot GPU, experimentation |
| **RunPod** | ~$2.49 | ~$1.64 | None | Native Docker, templates | REST API, Python SDK | Serverless inference, quick deploy |
| **Lambda Labs** | ~$2.99 | ~$2.49 | None (on-demand) | SSH + Docker | Web console, limited API | Reliable on-demand, managed feel |
| **Modal** | ~$3.95 | ~$2.50 | None (pay-per-second) | Python-native containers | Python SDK (code-first) | Python developers, burst workloads |
| **Paperspace (DO)** | N/A | ~$3.18 | $39/mo subscription | Gradient notebooks | REST API | Notebooks, ML experimentation |
| **CoreWeave** | ~$4.76 | ~$2.21 | Committed use for discounts | Kubernetes-native | Kubernetes API | Enterprise, large-scale training |
| **Salad** | ~$0.99 (NVL) | ~$0.50 | None | Docker containers | REST API | Batch inference, cost-sensitive |

### Provider Notes

**Vast.ai** — Peer-to-peer GPU marketplace with the lowest prices. Supply varies; machines may
be reclaimed. Best for non-latency-sensitive work. Full Docker support with custom images.
Launched serverless API in December 2025.

**RunPod** — Middle ground between marketplace and managed. Serverless offering with sub-200ms
cold starts (FlashBoot). Bring-your-own Docker image. Strong community and template ecosystem.
On-demand pods are 2-3x cheaper than serverless mode.

**Lambda Labs** — Managed cloud GPUs with predictable availability. Higher prices but reliable.
Good for teams wanting a traditional cloud experience. Offers H100, A100, B200, and GH200
instances.

**Modal** — Python-first serverless compute. No Docker files needed; decorate Python functions
with `@app.function(gpu="H100")`. Per-second billing. $30/month free tier. Excellent developer
experience but opinionated (Python-only).

**Paperspace (DigitalOcean)** — Now part of DigitalOcean. Prices stale since 2023 acquisition.
Requires $39/month subscription for high-end GPUs. Being sunset in favor of DigitalOcean
Gradient GPU Droplets. Not recommended for new projects.

**CoreWeave** — Kubernetes-native GPU cloud. No ingress/egress fees. Best for teams already on
Kubernetes who need large-scale GPU clusters. Up to 60% discounts on committed usage.
Enterprise-oriented.

**Salad** — Distributed network of 450K+ consumer GPUs across 191 countries. Extremely cheap
(RTX 4090 at $0.16/hr). Four priority tiers. Best for batch/async workloads that tolerate
variable latency. Not suitable for interactive coding sessions.

### Recommendation for aibox Users

**RunPod** or **Vast.ai** are the most practical choices for individual developers:
- Both support custom Docker images (critical for devcontainer workflows)
- Both have REST APIs suitable for programmatic provisioning
- Both offer pay-as-you-go with no minimum commitment
- RunPod has better reliability; Vast.ai has lower prices

**Modal** is excellent for Python-heavy ML workflows but its opinionated model (no raw Docker)
makes it less suitable for aibox's container-first approach.

---

## 2. Model Serving Frameworks

### Comparison Table

| Framework | OpenAI API compat | Docker image | Setup ease | Throughput | GPU required | Best for |
|---|---|---|---|---|---|---|
| **vLLM** | Full (drop-in) | Official GHCR image | Medium | 14-24x vs HF Transformers | Yes (CUDA) | Production serving, high throughput |
| **TGI** | Yes (OpenAI-compat) | Official GHCR image | Easy | High (Flash/PagedAttention) | Yes (CUDA) | HuggingFace ecosystem users |
| **Ollama** | Yes (`/v1/chat/completions`) | Official Docker image | Very easy | Moderate | Optional (CPU ok) | Local dev, simplicity |
| **llama.cpp** | Via server mode | Community images | Medium | Good (CPU optimized) | No (CPU-first) | CPU inference, edge, GGUF |
| **SGLang** | Partial | Via pip/Docker | Medium-Hard | Up to 6.4x vs SOTA | Yes (CUDA) | Structured output, max perf |

### Framework Notes

**vLLM** — The production standard for self-hosted LLM serving. PagedAttention serves 2-4x more
concurrent users with the same VRAM. Single H100 running Llama 3.3 70B in FP8 delivers ~400
tokens/sec. Supports continuous batching, streaming, structured outputs (JSON schema, regex,
grammar). V1 engine became default in 2025. vLLM-Omni v0.14.0 (2026) added multimodal support.
Docker: `ghcr.io/vllm-project/vllm-openai`.

**TGI (Hugging Face)** — Production-grade inference engine in Rust+Python. OpenAI-compatible API,
Prometheus metrics, OpenTelemetry tracing. **Entered maintenance mode December 2025** — only
bug fixes accepted. vLLM is being integrated as a TGI backend. Not recommended for new
deployments.

**Ollama** — The simplest path to local model inference. One-command install, one-command model
pull. 100+ models in the library. 52 million monthly downloads in Q1 2026. Supports GPU
acceleration (CUDA, ROCm, Metal) and CPU fallback. OpenAI-compatible API at
`localhost:11434/v1`. Desktop app for macOS/Windows (July 2025). Dynamic batching, multi-GPU
pipeline parallelism. Docker: `ollama/ollama`.

**llama.cpp** — C++ inference engine powering Ollama under the hood. GGUF format enables
quantized models from 2-bit to 8-bit. CPU-first with AVX/AVX2/AVX512 optimizations; also
supports CUDA, ROCm, Vulkan, Metal. A 7B model at 4-bit fits in 4GB VRAM. 1200+ contributors,
~4000 releases. Best for resource-constrained environments.

**SGLang** — Highest raw throughput (6.4x over SOTA on some benchmarks). Compressed finite state
machine enables 3x faster JSON decoding. 3.8x prefill / 4.8x decode on GB200 NVL72 with
DeepSeek. Newer and less battle-tested than vLLM. Best for teams optimizing structured output
at scale.

### Recommendation for aibox

**Ollama** for local/laptop development (simplest setup, Docker image available, CPU fallback).
**vLLM** for cloud GPU serving (production-grade, highest throughput, full OpenAI compatibility).

---

## 3. Best Open-Weight Coding Models (2025-2026)

### Comparison Table

| Model | Params | Active params | VRAM (Q4) | HumanEval | Context | License | FIM support |
|---|---|---|---|---|---|---|---|
| **Qwen3-Coder-Next** | 80B MoE | 3B | ~6 GB | — | 128K | Apache 2.0 | Yes |
| **Qwen 2.5 Coder 32B** | 32B | 32B | ~20 GB | 92.7% | 128K | Apache 2.0 | Yes |
| **Qwen 2.5 Coder 14B** | 14B | 14B | ~9 GB | ~89% | 128K | Apache 2.0 | Yes |
| **Qwen 2.5 Coder 7B** | 7B | 7B | ~5 GB | 88.4% | 128K | Apache 2.0 | Yes |
| **DeepSeek V3** | 671B MoE | 37B | ~320 GB (FP8) | 82.6% | 128K | MIT | No |
| **Codestral 25.01** | 22B | 22B | ~14 GB | 86.6% | 256K | Non-Production* | Yes |
| **CodeLlama 34B** | 34B | 34B | ~20 GB | ~67% | 16K | Llama license | Yes |
| **StarCoder2 15B** | 15B | 15B | ~10 GB | 46.3% | 16K | BigCode OpenRAIL-M | Yes |

*Codestral: free for research/testing; commercial license available on request from Mistral.

### Model Notes

**Qwen 2.5 Coder family** — Dominant in the open-weight coding space. The 32B model fits on a
single 24GB consumer GPU at Q4 quantization and achieves 92.7% HumanEval — competitive with
closed-source models. The 7B variant beats CodeStral-22B and DeepSeek Coder 33B despite being
much smaller. Trained on 5.5T tokens (70% code, 20% text, 10% math). Apache 2.0 license makes
it fully permissive for commercial use.

**Qwen3-Coder-Next** — Newest entrant (early 2026). 80B MoE with only 3B active parameters per
token, making it extremely efficient. SWE-bench Pass@5 of 64.6% reportedly beats Claude Opus
4.6 (58.3%) on real-world coding tasks. Available via Ollama.

**DeepSeek V3/V3.1** — Extremely capable but impractical for self-hosting. Requires 8x H200 GPUs
(141GB each) for full precision. Quantized versions need 4x A100 minimum. Best accessed via API
($0.27/$1.10 per million tokens — much cheaper than hosting).

**Codestral 25.01** — Mistral's dedicated coding model. #1 on LMSys Copilot Arena leaderboard.
Supports 80+ programming languages, 256K context. Fill-in-the-middle optimized. Non-production
license limits commercial self-hosting without contacting Mistral.

**CodeLlama** — Meta's 2023 coding model. Showing its age; the 34B model (~67% HumanEval) is
outperformed by Qwen 2.5 Coder 7B (88.4%). Not recommended for new deployments.

**StarCoder2** — BigCode project (ServiceNow + HuggingFace + NVIDIA). Permissive license. The
15B model (46.3% HumanEval) is significantly behind Qwen 2.5 Coder peers. Primarily of
historical interest.

### Recommendation by VRAM Budget

| VRAM | Best model | HumanEval | Notes |
|---|---|---|---|
| 4-8 GB | Qwen 3.5 4B or Qwen 2.5 Coder 7B (Q4) | ~55-88% | Laptop-viable |
| 8-16 GB | Qwen 2.5 Coder 14B (Q4) | ~89% | Sweet spot for single consumer GPU |
| 16-24 GB | Qwen 2.5 Coder 32B (Q4) | 92.7% | Best single-GPU coding model |
| 24+ GB | Qwen3-Coder-Next (MoE) | frontier | Best open model overall |

---

## 4. Integration with AI Coding Agents

### Compatibility Matrix

| Agent | Custom OpenAI endpoint | Configuration method | Self-hosted viable |
|---|---|---|---|
| **Aider** | Yes | `OPENAI_API_BASE=http://...` + `--model openai/<name>` | Yes — primary target |
| **Continue.dev** | Yes | `config.yaml`: provider "openai", custom apiBase | Yes — primary target |
| **Claude Code** | Proxy only | `ANTHROPIC_BASE_URL` + Bifrost/y-router gateway | Partial — requires Anthropic-format proxy |
| **Gemini CLI** | Limited | `GEMINI_API_ENDPOINT` + `GEMINI_API_KEY` | Limited — expects Gemini API format |
| **OpenAI Codex CLI** | Yes | `OPENAI_API_BASE` | Yes — designed for OpenAI-compat |

### Agent Details

**Aider** — First-class support for custom OpenAI-compatible endpoints. Set two environment
variables and prefix the model name with `openai/`. Supports different models for different
roles (architect, editor, weak-model). This is the most natural fit for self-hosted models in
aibox.

**Continue.dev** — Full support for custom endpoints in `config.yaml`. Dedicated documentation
for self-hosted models. Supports vLLM, Ollama, and any OpenAI-compatible server. Can configure
tool use and image input capabilities per model.

**Claude Code** — Does not natively support non-Anthropic models. However, proxy gateways like
Bifrost (open-source, Go-based) and y-router can translate between Anthropic API format and
OpenAI-compatible endpoints. The `ANTHROPIC_BASE_URL` environment variable enables routing
through these proxies. `ANTHROPIC_CUSTOM_MODEL_OPTION` can add custom model entries.
Functional but fragile — prompt format translation may degrade quality.

**Gemini CLI** — Supports `GEMINI_API_ENDPOINT` for custom base URLs, primarily intended for
enterprise proxies. No documented support for OpenAI-compatible endpoints. Would require a
translation proxy similar to Claude Code.

**OpenRouter** — Unified API layer (500+ models, 60+ providers) that works as a drop-in for any
OpenAI-compatible client. Useful as a fallback/routing layer but adds latency and cost markup.
Does not help with self-hosted models unless you run your own router.

---

## 5. aibox Integration Design Options

### Option A: Document Custom Endpoint Configuration (Lowest effort)

Add documentation showing how to point Aider and Continue at a custom endpoint:

```toml
# aibox.toml
[ai]
providers = ["aider"]

# User sets in their shell or .env:
# OPENAI_API_BASE=http://my-vllm-server:8000/v1
# OPENAI_API_KEY=not-needed
```

**Pros:** No code changes. Works today. Users bring their own infrastructure.
**Cons:** No discoverability. No validation. Users must know the config incantation.

### Option B: New `custom` Provider Type (Medium effort)

Extend the `AiProvider` enum with a `Custom` variant and add endpoint configuration:

```toml
# aibox.toml
[ai]
providers = ["aider", "custom"]

[ai.custom]
name = "my-model"
endpoint = "http://my-vllm-server:8000/v1"
api_key_env = "CUSTOM_LLM_KEY"  # optional
```

The `custom` provider would inject `OPENAI_API_BASE` and `OPENAI_API_KEY` into the container
environment. Aider and Continue would automatically pick these up.

**Pros:** First-class config support. Validates endpoint at `aibox build` time. Discoverable.
**Cons:** Requires CLI changes (new config variant, env injection). Does not provision the model
server.

### Option C: Ollama Sidecar Addon (Higher effort, highest value)

Add an `ai-ollama` addon that runs Ollama as a companion container:

```toml
# aibox.toml
[ai]
providers = ["aider"]

[addons]
ai-ollama = { model = "qwen2.5-coder:32b" }
```

This would:
1. Add an `ollama/ollama` sidecar container in docker-compose.yml
2. Pull the specified model on first start
3. Expose `http://ollama:11434/v1` inside the devcontainer network
4. Set `OPENAI_API_BASE=http://ollama:11434/v1` for Aider/Continue

For GPU passthrough, the sidecar would need:
```yaml
services:
  ollama:
    image: ollama/ollama
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
```

**Prerequisite:** Host must have NVIDIA Container Toolkit installed (`nvidia-ctk`).
Docker `--gpus all` flag or compose `deploy.resources.reservations.devices` section.

**Pros:** Zero-config self-hosted AI. Model specified declaratively. Works with Aider and
Continue out of the box. Ollama handles model download, quantization, and serving.
**Cons:** Requires GPU on host. Large model downloads (20GB+ for 32B models). Adds complexity
to devcontainer setup. Ollama throughput is ~16x lower than vLLM for concurrent use.

### Recommended Phased Approach

1. **Phase 1 (BACK-030, low effort):** Option B — add `[ai.custom]` endpoint config to the
   `AiProvider` enum. Inject `OPENAI_API_BASE` and key into container env. Document usage with
   Aider and Continue.

2. **Phase 2 (future):** Option C — add `ai-ollama` addon for turnkey local model serving.
   Requires the sidecar/companion container infrastructure (related to existing addon system).

3. **Phase 3 (future):** Option A enhanced — add `aibox gpu` commands to provision cloud GPUs
   (RunPod/Vast.ai API integration) and deploy vLLM with a chosen model. This is a significant
   feature and may be out of scope.

---

## 6. Cost Comparison

### Scenario: Typical Coding Agent Session

Assumptions: 100K tokens/hour (mixed input/output), 8-hour workday, single developer.

| Approach | Cost/hour | Cost/day (8h) | Cost/month (22 days) | Notes |
|---|---|---|---|---|
| **Claude Sonnet 4.6 API** | ~$0.90 | ~$7.20 | ~$158 | $3/$15 per M tokens, ~60% output |
| **Claude Opus 4.6 API** | ~$1.50 | ~$12.00 | ~$264 | $5/$25 per M tokens |
| **Claude Code Max (subscription)** | — | — | $100-200 | Flat rate, heavy usage |
| **DeepSeek V3 API** | ~$0.07 | ~$0.56 | ~$12 | $0.27/$1.10 per M tokens |
| **Qwen 2.5 Coder 32B on RunPod** | ~$2.49 | ~$19.92 | ~$438 | H100 on-demand, always on |
| **Qwen 2.5 Coder 32B on Vast.ai** | ~$1.87 | ~$14.96 | ~$329 | H100 spot, may be reclaimed |
| **Qwen 2.5 Coder 32B on Salad** | ~$0.99 | ~$7.92 | ~$174 | H100 NVL, batch tier |
| **Qwen 2.5 Coder 32B local (own GPU)** | ~$0 | ~$0 | ~$0* | *Amortized HW cost ~$1,500-2,000 for RTX 4090 |
| **Ollama on laptop (7B model)** | ~$0 | ~$0 | ~$0 | Lower quality than 32B, but free |

### Key Insights

1. **API access is cheaper than cloud GPU rental for typical individual usage.** At 100K
   tokens/hour, even Claude Sonnet ($0.90/hr) costs less than renting an H100 ($1.87-$2.99/hr).
   Cloud GPU self-hosting only makes economic sense at high utilization (>300K tokens/day
   consistently) or with multiple concurrent users.

2. **DeepSeek V3 API is remarkably cheap** at $0.07/hour for 100K tokens — 13x cheaper than
   Claude Sonnet. However, it is a Chinese-hosted API with potential latency and data residency
   concerns.

3. **Local GPU ownership breaks even quickly.** An RTX 4090 ($1,500-2,000) running Qwen 2.5
   Coder 32B pays for itself in 2-3 months vs cloud GPU rental, and provides unlimited
   inference thereafter. This is the most cost-effective path for developers with compatible
   hardware.

4. **The quality gap is narrowing.** Qwen 2.5 Coder 32B (92.7% HumanEval) is competitive with
   Claude Sonnet on pure code generation benchmarks, though closed models still lead on complex
   agentic tasks (multi-file edits, architectural reasoning).

5. **Hybrid is optimal.** Use Claude/GPT for complex agentic tasks; use self-hosted Qwen for
   autocomplete, simple edits, and high-volume code generation.

---

## 7. Recommendation

### For aibox Users Who Want Self-Hosted Models

**Immediate (no aibox changes needed):**
- Install Ollama locally or on a server
- Run `ollama pull qwen2.5-coder:32b` (if 24GB+ VRAM) or `qwen2.5-coder:7b` (if 8GB)
- Set `OPENAI_API_BASE=http://localhost:11434/v1` in your shell
- Use Aider with `--model openai/qwen2.5-coder:32b`
- Quality is surprisingly good for routine coding tasks

**Short-term (BACK-030 implementation):**
- Add `[ai.custom]` config section to aibox.toml
- Inject endpoint environment variables into the devcontainer
- Document Aider + vLLM/Ollama setup in aibox docs

**Medium-term (new addon):**
- Ship `ai-ollama` addon as a Docker Compose sidecar
- Handle GPU passthrough via NVIDIA Container Toolkit
- Pre-configure model pull and OpenAI-compatible endpoint

**Best model choices:**
- **Budget/laptop:** Qwen 2.5 Coder 7B via Ollama (5GB VRAM, 88.4% HumanEval)
- **Quality/desktop:** Qwen 2.5 Coder 32B via Ollama (20GB VRAM, 92.7% HumanEval)
- **Production serving:** Qwen 2.5 Coder 32B via vLLM on cloud GPU (RunPod/Vast.ai)
- **Frontier:** Qwen3-Coder-Next via vLLM (MoE, only 3B active params, state-of-the-art)

**Avoid:**
- CodeLlama — outdated, outperformed by Qwen models half its size
- StarCoder2 — low benchmark scores compared to current models
- DeepSeek V3 self-hosted — requires 8x H100/H200, impractical for individuals
- TGI — entering maintenance mode, vLLM is the successor

---

## Sources

- [Vast.ai Pricing](https://vast.ai/pricing)
- [RunPod Pricing](https://www.runpod.io/pricing)
- [Lambda Labs Pricing](https://lambda.ai/pricing)
- [Modal Pricing](https://modal.com/pricing)
- [CoreWeave Pricing](https://www.coreweave.com/pricing)
- [Salad GPU Pricing](https://salad.com/pricing)
- [H100 Rental Prices Compared (2026)](https://intuitionlabs.ai/articles/h100-rental-prices-cloud-comparison)
- [Cloud GPU Pricing 2026 (SynpixCloud)](https://www.synpixcloud.com/blog/cloud-gpu-pricing-comparison-2026)
- [vLLM OpenAI-Compatible Server](https://docs.vllm.ai/en/stable/serving/openai_compatible_server/)
- [vLLM Quickstart 2026](https://www.glukhov.org/llm-hosting/vllm/vllm-quickstart/)
- [Ollama Model Library](https://ollama.com/library)
- [Ollama 2025 Updates](https://www.infralovers.com/blog/2025-08-13-ollama-2025-updates/)
- [SGLang GitHub](https://github.com/sgl-project/sglang)
- [TGI GitHub](https://github.com/huggingface/text-generation-inference)
- [llama.cpp GitHub](https://github.com/ggml-org/llama.cpp)
- [Best Local Coding Models 2026 (InsiderLLM)](https://insiderllm.com/guides/best-local-coding-models-2026/)
- [Qwen 2.5 Coder Family](https://qwenlm.github.io/blog/qwen2.5-coder-family/)
- [DeepSeek V3 HuggingFace](https://huggingface.co/deepseek-ai/DeepSeek-V3)
- [Codestral (Mistral)](https://mistral.ai/news/codestral)
- [Aider OpenAI-Compatible APIs](https://aider.chat/docs/llms/openai-compat.html)
- [Continue.dev Self-Host Guide](https://docs.continue.dev/guides/how-to-self-host-a-model)
- [Claude Code Model Configuration](https://code.claude.com/docs/en/model-config)
- [Bifrost AI Gateway (Maxim)](https://www.getmaxim.ai/articles/running-non-anthropic-models-in-claude-code-via-an-enterprise-ai-gateway/)
- [Gemini CLI Custom Endpoint Issue](https://github.com/google-gemini/gemini-cli/issues/1679)
- [OpenRouter](https://openrouter.ai/)
- [Claude API Pricing 2026](https://platform.claude.com/docs/en/about-claude/pricing)
- [Self-Hosted LLM Cost Comparison 2026](https://devtk.ai/en/blog/self-hosting-llm-vs-api-cost-2026/)
- [NVIDIA Container Toolkit Install Guide](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html)
- [Paperspace Pricing (DigitalOcean)](https://docs.digitalocean.com/products/paperspace/pricing/)
