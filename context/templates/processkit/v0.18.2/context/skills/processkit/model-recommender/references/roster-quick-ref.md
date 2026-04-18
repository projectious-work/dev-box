# Roster Quick Reference

> Auto-generated view of `mcp/model_scores.json`. For live filtered output,
> call `list_models()`. For full sub-dimension profiles, call `get_profile(id, scope="full")`.
> Estimated models (marked †) need Workflow C validation before production use.

| Model | R | E | S | B | L | G | ctx K | $/1M out | Provider |
|-------|---|---|---|---|---|---|-------|-----------|----------|
| Claude Opus 4.6 | 5 | 5 | 2 | 4 | 5 | 5 | 200 | $75.00 | Anthropic |
| Claude Sonnet 4.6 | 4 | 5 | 3 | 4 | 4 | 5 | 200 | $15.00 | Anthropic |
| Claude Haiku 4.5 | 3 | 3 | 5 | 3 | 4 | 5 | 200 | $1.25 | Anthropic |
| GPT-4o | 4 | 4 | 3 | 5 | 4 | 2 | 128 | $10.00 | OpenAI |
| o3 | 5 | 5 | 1 | 3 | 4 | 2 | 128 | $40.00 | OpenAI |
| o4-mini | 4 | 4 | 3 | 3 | 4 | 2 | 128 | $4.40 | OpenAI |
| Gemini 2.5 Pro | 5 | 5 | 3 | 5 | 4 | 2 | 1000 | $10.00 | Google |
| Gemini Flash 2.0 | 3 | 3 | 5 | 5 | 3 | 2 | 1000 | $0.30 | Google |
| Llama 3.3 70B | 3 | 3 | 4 | 3 | 3 | 5 | 128 | self-hosted | Meta |
| Mistral Large 3 | 3 | 4 | 4 | 3 | 3 | 4 | 128 | $6.00 | Mistral |
| DeepSeek V3 | 4 | 4 | 4 | 3 | 3 | 1 | 64 | $0.28 | DeepSeek |
| DeepSeek R1 | 5 | 4 | 3 | 3 | 3 | 1 | 64 | $2.19 | DeepSeek |
| MiniMax Text-01 | 3 | 3 | 3 | 5 | 3 | 2 | 1000 | $1.10 | MiniMax |
| Qwen 2.5 72B | 4 | 4 | 4 | 3 | 3 | 4 | 128 | $1.20 | Alibaba / Open |
| Qwen 2.5 Coder 32B | 3 | 5 | 4 | 3 | 3 | 4 | 128 | $0.60 | Alibaba / Open |
| Phi-4 | 4 | 3 | 5 | 2 | 3 | 4 | 16 | $0.14 | Microsoft |
| Gemma 3 27B | 3 | 3 | 4 | 4 | 3 | 4 | 128 | $0.45 | Google / Open |
| Grok 3 | 5 | 4 | 3 | 4 | 4 | 2 | 128 | $15.00 | xAI |
| Command R+ | 3 | 3 | 4 | 3 | 4 | 3 | 128 | $10.00 | Cohere |
| Gemini 2.5 Flash | 3 | 3 | 5 | 5 | 3 | 2 | 1000 | $0.30 | Google |
| Codestral | 3 | 5 | 4 | 3 | 3 | 4 | 256 | $0.90 | Mistral |
| Mistral Medium 3 | 3 | 3 | 4 | 3 | 3 | 4 | 128 | $2.00 | Mistral |
| Mistral Small 3 | 2 | 3 | 5 | 2 | 3 | 4 | 32 | $0.30 | Mistral |
| Mistral Deep Think | 4 | 4 | 2 | 3 | 4 | 4 | 128 | $6.00 | Mistral |
| Qwen3 235B (MoE) | 5 | 5 | 3 | 3 | 4 | 4 | 128 | self-hosted | Alibaba / Open |
| GPT-5† | 5 | 5 | 3 | 5 | 5 | 2 | 256 | $20.00 | OpenAI |
| GPT-5.4 Pro† | 5 | 5 | 2 | 5 | 5 | 2 | 256 | $60.00 | OpenAI |
| Grok 3.5† | 5 | 5 | 3 | 4 | 4 | 2 | 128 | $15.00 | xAI |
| Grok 4† | 5 | 5 | 3 | 4 | 5 | 2 | 256 | $20.00 | xAI |
| Grok 4.1† | 5 | 5 | 3 | 5 | 5 | 2 | 256 | $20.00 | xAI |
| Grok 4 Heavy† | 5 | 5 | 1 | 4 | 5 | 2 | 256 | $40.00 | xAI |
| Gemini 3 Flash† | 4 | 3 | 5 | 5 | 3 | 2 | 2000 | $0.20 | Google |
| Gemini 3.1 Pro† | 5 | 5 | 3 | 5 | 5 | 2 | 2000 | $8.00 | Google |
| MiniMax M2.5† | 4 | 3 | 3 | 5 | 3 | 2 | 1000 | $1.10 | MiniMax |

† Estimated scores — extrapolated from predecessor models. Run Workflow C (Roster Refresh) to validate.

## Governance footnotes

- **G:1** — Chinese jurisdiction (DeepSeek, MiniMax API, Alibaba Cloud API): never use for sensitive data
- **G:2** — US commercial (OpenAI, Google, xAI): data may be used for improvement without enterprise DPA
- **G:3** — Mid-tier commercial (Cohere): SOC 2 available, HIPAA BAA available for enterprise
- **G:4** — EU-based or open weights (Mistral, Phi, Gemma, Qwen/Llama self-hosted): strong regional sovereignty, limited compliance certs
- **G:5** — Best-in-class (Anthropic, Llama self-hosted): no training on data, full audit, HIPAA BAA available

For open-weight models (Llama, Qwen, Gemma, Phi, Codestral): G score shown is for **self-hosted** deployment.
Via third-party API: G drops to 2–3. Via Chinese-provider API: G drops to 1.
