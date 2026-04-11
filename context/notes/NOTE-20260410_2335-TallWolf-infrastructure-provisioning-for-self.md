---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-TallWolf-infrastructure-provisioning-for-self
  created: 2026-04-11
spec:
  title: "Infrastructure Provisioning for Self-Hosted aibox — Research Report"
  type: reference
  state: permanent
  tags: [infrastructure, provisioning, hetzner, tofu, gpu, devpod, remote-dev]
  skill: research-with-confidence
  source_file: infrastructure-provisioning-2026-03.md
---

# Infrastructure Provisioning for Self-Hosted aibox — Research Report

**Date:** 2026-03-26

Research for self-hosted deployment infrastructure. Covers IaC tools, cloud provider comparisons, GPU provisioning, deployment patterns, and where aibox's responsibility boundary should be.

---

## 1. IaC Tools

### OpenTofu / Terraform

OpenTofu is the open-source fork of Terraform (MPL-licensed). For aibox, OpenTofu is the recommended choice due to its open-source license alignment.

**Relevant providers:** Hetzner Cloud (`hetznercloud/hcloud`), AWS (`hashicorp/aws`), GCP (`hashicorp/google`), DigitalOcean, Vultr, OVH, Scaleway.

A minimal Tofu module for a dev server requires ~50-80 lines of HCL: SSH key, firewall, server resource, and cloud-init for Docker installation.

### Ansible for Post-Provisioning

Standard post-provisioning playbook: install Docker → install NVIDIA Container Toolkit (if GPU) → configure SSH → configure ufw firewall → create workspace directory.

**Tofu + Ansible workflow:**
```
tofu apply          →  VM exists, has IP, SSH key installed
ansible-playbook    →  Docker installed, firewall configured, ready for aibox
aibox start --remote user@<ip>  →  Environment running
```

---

## 2. Cloud Provider Comparison — Coding Dev Server (4 vCPU / 8 GB RAM)

Prices as of March 2026. **Note: Hetzner prices increase ~30-35% on April 1, 2026.**

| Provider | Instance | Monthly cost | Notes |
|---|---|---|---|
| **Hetzner Cloud CX32** | 4 vCPU / 8 GB | ~€7.49 | EU only. Cheapest option. |
| **Hetzner CAX21 (ARM)** | 4 vCPU / 8 GB | ~€7.49 | ARM (Ampere). EU only. |
| **Scaleway DEV1-L** | 4 vCPU / 8 GB | ~€12/mo | EU only (Paris, Amsterdam) |
| **DigitalOcean** | 4 vCPU / 8 GB | $63/mo | Per-second billing since Jan 2026 |
| **Vultr** | 4 vCPU / 8 GB | $48/mo | 17 regions globally |
| **AWS t3.xlarge** | 4 vCPU / 16 GB | ~$122/mo | + EBS cost |
| **GCP e2-standard-4** | 4 vCPU / 16 GB | ~$97/mo | + disk cost |

**Verdict:** Hetzner Cloud is 5-15x cheaper than hyperscalers. Recommended default for individual developers. AWS/GCP only justified when users need specific cloud services.

## 3. GPU Pricing

| Provider | GPU | VRAM | Monthly cost | Notes |
|---|---|---|---|---|
| **Hetzner GEX44** | RTX 4000 SFF Ada | 20 GB | €184/mo | Dedicated server, €79 setup fee |
| **vast.ai L40** | L40 | 48 GB | ~$223/mo | Marketplace pricing, variable |
| **RunPod RTX 4090** | RTX 4090 | 24 GB | ~$0.44/hr | Community cloud |
| **AWS g5.xlarge** | A10G | 24 GB | ~$724/mo | Best general-purpose GPU on AWS |

**Verdict for GPU:** Hetzner GEX44 excellent for persistent GPU dev. For on-demand, vast.ai and RunPod offer best price-performance. AWS/GCP are 2-4x more expensive.

---

## 4. Deployment Patterns — Where Does aibox's Responsibility End?

| Pattern | Description | aibox effort | User complexity | Recommendation |
|---|---|---|---|---|
| **A: Docs only** | Documentation for manual VM setup, user runs `aibox start --remote` | Zero | High | Phase 1 baseline |
| **B: Reference Tofu modules** | aibox ships optional Tofu modules for Hetzner/AWS/GCP | Low-Medium | Medium | **Recommended Phase 1** |
| **C: Integrated provisioning** | `aibox provision hetzner` creates VM, installs Docker, deploys | High | Low | **Not recommended** — scope creep |
| **D: DevPod provider** | aibox generates devcontainer.json, DevPod handles provisioning | Zero | Low | **Recommended Phase 1** |
| **E: K8s-only** | User brings K8s cluster, aibox deploys via Helm | Medium | Medium | Covered by BACK-068 |

---

## 5. The Boundary Question — Should aibox Be Opinionated About Infrastructure?

**The "uv for AI" philosophy:** aibox positions itself like "uv for AI work environments." uv does not provision servers — it manages Python environments on whatever machine it runs on.

**aibox's infrastructure scope:**
- **Own:** Container definition, AI tool configuration, workspace setup, addon management
- **Not own:** VM provisioning, cloud API integration, infrastructure lifecycle
- **Enable:** Remote execution via SSH (`--remote`), compatibility with DevPod/Coder/Tofu

**Conclusion: Document and reference, do not build.**

1. Ship reference Tofu modules (Pattern B) for the most common providers
2. Document DevPod compatibility (Pattern D) as the easiest path
3. Provide clear "getting started remotely" documentation
4. **Never integrate cloud SDKs into the CLI binary** (reject Pattern C)

---

## 6. GPU in aibox Containers

NVIDIA Container Toolkit + compose `deploy.resources.reservations` section enables `docker run --gpus all`. vast.ai and RunPod are Docker-ready GPU providers — no post-provisioning needed for GPU access.

**Minimum requirements (GPU):**
- NVIDIA GPU with 16+ GB VRAM (24+ recommended)
- NVIDIA Driver 535+
- NVIDIA Container Toolkit
- RAM: 16 GB minimum (32 recommended)
- Disk: 100 GB+ (7B model weights ~14GB)
