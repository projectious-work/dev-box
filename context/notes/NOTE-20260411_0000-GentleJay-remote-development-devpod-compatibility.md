---
id: NOTE-20260411_0000-GentleJay-remote-development-devpod-compatibility
title: "Remote Development — DevPod Compatibility and aibox start --remote"
type: reference
status: permanent
created: 2026-04-11T00:00:00Z
tags: [remote-dev, devpod, ssh, cloud]
skill: research-with-confidence
source_file: remote-development-2026-03.md
---

# Remote Development Patterns — Research Report — March 2026

Research for BACK-011. Investigates how aibox could support running environments on
remote hosts with a local thin client. Covers existing solutions, architecture patterns,
integration options, and a recommendation. Conducted 2026-03-26.

---

## 1. Existing Remote Development Solutions (2025-2026 State)

### 1.1 VS Code Remote SSH / Remote Containers / Dev Containers on Remote Hosts

VS Code's Remote - SSH extension installs a VS Code Server on the remote host over SSH.
Once connected, users can invoke Dev Containers: Reopen in Container to run a
devcontainer on the remote Docker/Podman daemon — no local Docker required.

| Aspect | Detail |
|---|---|
| Architecture | Local VS Code thin client + remote VS Code Server over SSH tunnel |
| Container support | Remote - SSH and Dev Containers extensions compose: SSH into host, then open devcontainer |
| Protocol | SSH tunnel, with extension host running server-side |
| Strengths | Native VS Code experience, mature, well-documented |
| Weaknesses | Proprietary server binary, VS Code-only, extensions must be installed remotely |

This is the closest analog to what aibox would do with `--remote`: the user's project
is on a remote machine, Docker runs there, and the IDE connects over SSH.

### 1.2 GitHub Codespaces

Microsoft's managed cloud development environment service built on the devcontainer spec.

| Aspect | Detail |
|---|---|
| Architecture | Managed VMs running Docker with devcontainer, accessed via browser or VS Code |
| Compute | 2-core ($0.18/hr) to 32-core ($2.88/hr) VMs, auto-suspend after inactivity |
| Storage | $0.07/GiB/month, persists across suspend/resume |
| Prebuilds | Pre-built images cached per branch for instant startup |
| Strengths | Zero setup, deep GitHub integration, prebuild system |
| Weaknesses | Vendor lock-in (GitHub only), expensive at scale, no GPU, no self-hosting |

Codespaces validates that devcontainer-on-remote-VM is a viable architecture. The
prebuild model (build image on commit, start from snapshot) is worth studying.

### 1.3 Gitpod (now Ona)

Originally a browser-based CDE with SaaS and self-hosted options. Underwent significant
changes in 2025: rebranded to Ona, sunset SaaS pay-as-you-go (October 2025), and pivoted
all offerings to self-hosted only (Gitpod Flex, Gitpod Enterprise).

| Aspect | Detail |
|---|---|
| Architecture | Kubernetes-based, runs workspace pods with custom OCI images |
| Config | `.gitpod.yml` (not devcontainer.json — proprietary spec) |
| Self-hosted | All tiers now self-hosted only; requires Kubernetes cluster |
| Current focus | Pivoting to "mission control for software engineering agents" |
| Strengths | Prebuild system, workspace snapshots, Kubernetes-native |
| Weaknesses | SaaS sunset, proprietary config format, heavy K8s dependency, uncertain direction |

The Gitpod pivot away from SaaS and toward agent orchestration signals that the CDE
market is consolidating. Not a strong integration target for aibox.

### 1.4 JetBrains Gateway

JetBrains' remote development solution using a thin local client (Gateway) that connects
to a backend IDE running on a remote machine.

| Aspect | Detail |
|---|---|
| Architecture | Local Gateway (thin client) + remote IDE backend, connected via RD protocol |
| Protocol | End-to-end TLS 1.3 over SSH tunnel, Projector-based GUI rendering |
| 2026.1 update | Debugger redesigned for remote — tool window renders locally, session runs remotely |
| Integrations | Connects to Codespaces, Gitpod, Coder, Google Cloud, CodeCatalyst |
| Strengths | Full JetBrains IDE experience remotely, excellent latency handling |
| Weaknesses | JetBrains subscription required, does not use devcontainer spec natively |

Relevant because aibox users who prefer JetBrains IDEs would need Gateway support.
The key insight: Gateway connects via SSH, so any solution that exposes SSH to a
container workspace is Gateway-compatible.

### 1.5 Coder (coder.com)

Open-source (AGPL) self-hosted remote development platform. Templates are Terraform
definitions that provision workspaces on any infrastructure.

| Aspect | Detail |
|---|---|
| Architecture | Central Coder server + Terraform templates + coder_agent in workspace |
| Templates | Written in Terraform (HCL), define infrastructure (Docker, K8s, VMs, cloud) |
| Editions | Community (free, open source) and Premium (paid, audit/RBAC/autoscaling) |
| IDE support | VS Code (SSH), JetBrains Gateway, code-server (browser), any SSH client |
| Dev Containers | Supports devcontainer.json via @devcontainers/cli integration |
| Strengths | Infrastructure-agnostic via Terraform, devcontainer support, active community |
| Weaknesses | Requires running a Coder server, Terraform complexity, AGPL license |

Coder is a strong integration target. An aibox Coder template could provision a
workspace that runs `aibox sync && aibox start` inside a VM or container.

### 1.6 DevPod (loft.sh)

Open-source (Apache 2.0) client-only tool for creating devcontainer-based environments
on any backend. No server component required.

| Aspect | Detail |
|---|---|
| Architecture | Client-only CLI/desktop app, deploys agent to remote, runs devcontainer there |
| Config | Uses standard `devcontainer.json` — full spec compatibility |
| Providers | Pluggable: Docker, SSH, Kubernetes, AWS, GCP, Azure, DigitalOcean, and custom |
| Agent model | DevPod binary deployed to remote, starts SSH server on stdio tunnel for IDE |
| Credential sync | SSH agent forwarding, git credential forwarding, docker credential forwarding |
| Cost | 5-10x cheaper than Codespaces (bare VMs, auto-shutdown) |
| Strengths | No server, devcontainer-native, provider-agnostic, Apache 2.0 license |
| Weaknesses | Less mature than Coder for enterprise, no built-in dashboard/team management |

**DevPod is the most architecturally aligned solution with aibox.** Both are client-only
tools that generate devcontainer artifacts. DevPod's provider system is the natural
extension point: an aibox DevPod provider would let DevPod manage the remote
infrastructure while aibox manages the environment definition.

### 1.7 Hocus

Self-hosted Gitpod alternative using Firecracker VMs instead of containers.

| Aspect | Detail |
|---|---|
| Architecture | Self-hosted, Firecracker micro-VMs, Dockerfile-based env definition |
| Status | **Discontinued** — repository archived, maintainers no longer active |
| Strengths | Full VM isolation, no K8s dependency |
| Weaknesses | Dead project, no community, not viable for integration |

Not a viable target. Mentioned for completeness only.

---

## 2. Architecture Patterns

### 2.1 Pattern Comparison

| Pattern | How it works | Latency | Complexity | IDE support |
|---|---|---|---|---|
| **SSH-based** | User SSHes into remote host, runs tools there directly | Low (terminal) | Low | Any (terminal, VS Code, JetBrains) |
| **Agent-based** | Local IDE connects to remote agent via WebSocket/gRPC/SSH | Medium | Medium | IDE-specific (VS Code, JetBrains) |
| **Browser-based** | Web IDE (code-server, Theia) runs in container, accessed via browser | Medium-High | Medium | Browser only |
| **K8s orchestration** | Platform provisions workspace pods on Kubernetes cluster | Variable | High | Any (via SSH/browser) |

### 2.2 SSH-Based (Simplest)

The user SSHes into a remote machine that has Docker/Podman installed. They run `aibox`
commands there directly. The IDE either runs in the terminal (vim + zellij — aibox's
default) or connects remotely (VS Code Remote SSH, JetBrains Gateway).

```
Local machine              Remote host
+------------------+       +---------------------------+
| Terminal / IDE   | SSH   | Docker/Podman             |
| (thin client)    |------>| aibox CLI                 |
|                  |       | devcontainer (zellij+vim) |
+------------------+       +---------------------------+
```

**Pros:** Simplest to implement, works with aibox's existing zellij-centric model,
no new protocol/agent needed, all tools run natively on the remote.

**Cons:** Requires aibox CLI installed on remote, user manages remote host, no
automatic provisioning.

### 2.3 Agent-Based (IDE-Centric)

A local IDE connects to a lightweight agent running inside the remote container.
The agent handles file operations, terminal multiplexing, and port forwarding.

```
Local machine              Remote host
+------------------+       +---------------------------+
| VS Code / Gate   | SSH   | Agent (VS Code Server /   |
| way / DevPod     |<----->| JetBrains backend /       |
|                  |       | DevPod agent)             |
+------------------+       | devcontainer              |
                           +---------------------------+
```

This is how VS Code Remote, JetBrains Gateway, and DevPod work. DevPod's model is
particularly relevant: it deploys its own agent binary to the remote, which then
manages the devcontainer lifecycle and provides an SSH server for IDE connection.

### 2.4 Browser-Based (Zero-Install Client)

A web-based IDE runs inside the container and is accessed via browser. Options:

| Tool | Description | VS Code compat | License |
|---|---|---|---|
| **code-server** | VS Code in the browser (by Coder) | High (fork of VS Code) | MIT |
| **Eclipse Theia** | Extensible cloud IDE platform | Partial (VS Code extension compat) | EPL 2.0 |
| **VS Code Web** | Official VS Code for the Web | Full (limited extension support) | Proprietary |

**Pros:** Zero local setup, works from any device with a browser.

**Cons:** Worse latency than native IDE, limited extension support, does not align with
aibox's terminal-first (zellij + vim) philosophy.

### 2.5 Kubernetes Orchestration

Platforms like Coder, Gitpod, and DevPod (K8s provider) run workspace containers as
Kubernetes pods. This enables team-wide provisioning, resource limits, and auto-scaling.

**Pros:** Multi-tenant, scalable, integrates with existing K8s infrastructure.

**Cons:** Requires K8s cluster, significant operational complexity, overkill for
individual developers.

---

## 3. How aibox Could Support Remote Development

### Option A: `aibox start --remote user@host`

**Concept:** aibox CLI runs locally, SSHes into the remote host, and executes
Docker/Podman commands there. The generated devcontainer files are synced to the
remote, built remotely, and the user attaches via SSH.

**Implementation sketch:**
1. `aibox sync` generates `.devcontainer/` locally as today
2. `aibox start --remote user@host` does:
   - `rsync .devcontainer/ user@host:~/project/.devcontainer/`
   - `ssh user@host "cd ~/project && docker compose -f .devcontainer/docker-compose.yml up -d"`
   - `ssh -t user@host "docker exec -it <container> zellij attach --create main"`

**Effort:** Medium. Requires SSH command execution layer, rsync of generated files,
and remote container attach logic.

**Strengths:**
- Minimal architecture change — aibox stays client-only
- Works with aibox's existing zellij-centric workflow
- User gets full terminal experience over SSH
- No new agent or server component

**Weaknesses:**
- Requires aibox.toml + project source on remote (or sync both)
- SSH latency affects terminal responsiveness
- No automatic VM provisioning

### Option B: `aibox deploy` to Kubernetes

**Concept:** Generate Kubernetes manifests (Helm chart) from aibox.toml and deploy the
devcontainer as a pod. Relates to BACK-068 (Helm chart scaffolding).

**Implementation sketch:**
1. `aibox deploy --target k8s` generates Helm chart from config
2. `helm install <name> ./deploy/helm` creates the workspace pod
3. User connects via `kubectl exec`, VS Code, or JetBrains Gateway

**Effort:** High. Requires Helm chart generation, K8s API interaction, ingress/service
setup, PVC management.

**Strengths:**
- Scalable to teams
- Integrates with enterprise K8s infrastructure
- GPU access via K8s device plugins

**Weaknesses:**
- Heavy operational requirement (K8s cluster)
- Significant new code surface
- Zellij layout/attach over kubectl exec is awkward

### Option C: Integration with Existing Platforms

**Concept:** Instead of building remote infrastructure management, integrate with
platforms that already handle it. Two concrete paths:

#### C1: DevPod Provider

Write a DevPod provider that uses aibox to define the environment. DevPod handles
provisioning (SSH, AWS, K8s, etc.) and aibox handles environment configuration.

```yaml
# provider.yaml (aibox DevPod provider)
name: aibox
version: 0.1.0
description: "Use aibox environments with DevPod"
exec:
  command: |-
    # DevPod calls this to run commands in the workspace
    ssh ${DEVPOD_MACHINE_HOST} "docker exec ${WORKSPACE_ID} $@"
```

The devcontainer.json that aibox already generates is directly consumable by DevPod.
No provider is even strictly needed — users can point DevPod at a project with
aibox-generated `.devcontainer/` and it works out of the box.

**Effort:** Low for basic support (aibox already generates devcontainer.json).
Medium for a dedicated provider with aibox-specific optimizations.

#### C2: Coder Template

Write a Coder template (Terraform) that provisions a workspace, installs aibox, and
runs `aibox sync && aibox start` inside it.

```hcl
# main.tf (aibox Coder template)
resource "docker_container" "workspace" {
  image = "ghcr.io/projectious-work/aibox/base:latest"
  ...
}

resource "coder_agent" "main" {
  os   = "linux"
  arch = "amd64"
  startup_script = <<-EOT
    aibox sync && aibox start
  EOT
}
```

**Effort:** Medium. Requires maintaining Terraform template, understanding Coder
agent lifecycle.

### Option D: Cloud Provider Integration

**Concept:** `aibox start --cloud aws` provisions a VM on a cloud provider, syncs
the project, builds the container, and provides SSH access.

**Implementation sketch:**
1. User configures cloud credentials in `aibox.toml` or environment
2. `aibox start --cloud aws` uses cloud SDK to create an EC2 instance
3. Instance bootstrap: install Docker, pull/build image, start container
4. SSH config entry written locally, user connects

**Effort:** Very high. Each cloud provider requires SDK integration, VM lifecycle
management, networking setup, cost management.

**Strengths:**
- Fully integrated experience
- GPU instances available (p-series on AWS, A2/G2 on GCP)
- aibox controls full lifecycle

**Weaknesses:**
- Massive scope — essentially building a mini-cloud platform
- Cloud SDK dependencies bloat the CLI binary
- Credential management complexity
- DevPod already does this better with its provider system

---

## 4. Key Design Decisions

### 4.1 Where Does the aibox CLI Run?

| Scenario | CLI location | Docker location | Notes |
|---|---|---|---|
| Current (local) | Local machine | Local machine | Today's model |
| Option A (SSH) | Local machine | Remote host | CLI orchestrates remotely via SSH |
| Option A' (remote CLI) | Remote host | Remote host | CLI installed on remote, user SSHes in |
| Option C (DevPod) | Local machine | Managed by DevPod | DevPod handles provisioning |

**Recommendation:** Support both "CLI on local, Docker on remote" (Option A) and
"CLI on remote" (Option A'). The latter is simpler — users SSH into a remote machine
and use aibox normally. The former adds convenience but more complexity.

### 4.2 How Are Files Synced?

| Strategy | Mechanism | Latency | Bidirectional | Complexity |
|---|---|---|---|---|
| **Git-based** | Push locally, pull remotely | High (commit cycle) | No | Low |
| **rsync** | One-shot delta sync | Medium | No (push only) | Low |
| **Mutagen** | Real-time bidirectional sync | Low | Yes | Medium |
| **SSH mount (SSHFS)** | FUSE filesystem over SSH | High | Yes | Low |
| **No sync** | Code lives on remote only | N/A | N/A | Lowest |

**Recommendation:** For the initial implementation, support two modes:
1. **No sync** (default for `--remote`): project already exists on remote (cloned via git).
   This is the simplest and most reliable approach.
2. **rsync** (opt-in): `aibox start --remote user@host --sync` pushes local
   `.devcontainer/` and optionally project files to the remote.

Mutagen is worth considering later for real-time bidirectional sync, but adds a
dependency and complexity that is not justified for v1.

### 4.3 How Does the User Connect?

| Method | Experience | aibox alignment | Setup |
|---|---|---|---|
| **SSH + zellij** | Terminal multiplexer over SSH | Perfect | Lowest |
| **VS Code Remote SSH** | Full VS Code GUI | Good | Needs VS Code |
| **JetBrains Gateway** | Full JetBrains GUI | Good | Needs Gateway |
| **Browser (code-server)** | VS Code in browser | Fair | Needs code-server in image |

**Recommendation:** SSH + zellij as the primary path. This is aibox's core experience
and requires zero additional components. VS Code Remote SSH and JetBrains Gateway work
automatically if the container exposes SSH — document this as a secondary path. Do not
invest in browser-based IDE support.

### 4.4 How Are Secrets Managed Remotely?

| Secret type | Local mechanism | Remote mechanism |
|---|---|---|
| SSH keys | Volume mount (`~/.ssh:ro`) | SSH agent forwarding (`ssh -A`) |
| AI provider API keys | Environment variables in compose | `SendEnv`/`AcceptEnv` in SSH, or `.env` on remote |
| Git credentials | Volume mount (`~/.config/git`) | Git credential helper over SSH, or agent forwarding |
| Cloud credentials | Environment variables | Forward via SSH or use instance roles (AWS IAM, GCP SA) |

**Recommendation:** Use SSH agent forwarding for SSH keys (already standard).
For API keys and environment variables, support two approaches:
1. **Forward from local:** `aibox start --remote` reads local `.env` and passes
   values via `ssh -o SendEnv=...` or injects them into the remote compose file.
2. **Manage on remote:** secrets stored on the remote host (e.g., in a `.env` file
   or a secrets manager). This is the simpler and more secure default for persistent
   remote environments.

### 4.5 GPU Access for AI Workloads

| Platform | GPU mechanism | Cost (H100) | Notes |
|---|---|---|---|
| AWS EC2 | `p5.xlarge` instances, NVIDIA Container Toolkit | ~$3.00/hr | Instance roles, EBS |
| GCP | A3/G2 VMs, Container-Optimized OS | ~$3.20/hr | Preemptible for cost savings |
| Runpod | Serverless GPU, Docker-native | ~$2.10/hr | Pay-per-second billing |
| Vast.ai | Marketplace GPU rentals | ~$1.50/hr | Variable availability |
| K8s | `nvidia.com/gpu` resource requests | Depends on cluster | Device plugin required |

**Recommendation:** GPU support is a natural extension of `--remote` but should not be
a v1 requirement. The compose file already supports `deploy.resources.reservations` for
GPU passthrough. For remote, this means:
1. Ensure the remote host has NVIDIA Container Toolkit installed (document requirement)
2. Add `[container.gpu]` config section to aibox.toml (optional, generates GPU resource
   reservation in compose)
3. Cloud-specific GPU provisioning belongs in Option C (DevPod/Coder) or Option D, not
   in the core `--remote` flag

---

## 5. Comparison: Integration Options Summary

| | Option A: `--remote` SSH | Option B: K8s deploy | Option C1: DevPod | Option C2: Coder | Option D: Cloud SDK |
|---|---|---|---|---|---|
| **Effort** | Medium | High | Low-Medium | Medium | Very High |
| **Server required** | No | K8s cluster | No | Coder server | No |
| **aibox changes** | SSH execution layer | Helm generation | Minimal/none | Template only | Cloud SDK integration |
| **Zellij support** | Native | Awkward | Via SSH | Via SSH | Via SSH |
| **GPU support** | Manual setup | K8s device plugin | Via provider | Via template | Cloud API |
| **Team scaling** | Manual | Good | Manual per user | Good | Manual |
| **Aligns with aibox** | Strong | Moderate | Strong | Moderate | Weak |

---

## 6. Recommendation

### Phase 1: DevPod compatibility (zero effort) + documentation

aibox already generates standard `devcontainer.json` and `docker-compose.yml` files.
DevPod can consume these directly. The immediate action is:

1. **Document** that DevPod works with aibox projects out of the box
2. **Test** the workflow: `devpod up . --provider ssh --host user@remote`
3. **Add a docs page** explaining how to use aibox with DevPod for remote development
4. **Verify** that the zellij-centric experience works when DevPod connects via SSH

This gives aibox users remote development capability with zero CLI changes.

### Phase 2: `aibox start --remote user@host` (medium effort)

For users who do not want to install DevPod, add native SSH remote support:

1. **`aibox start --remote user@host`** — SSH into remote, check for Docker, sync
   `.devcontainer/` via rsync, build and start the container remotely
2. **`aibox attach --remote user@host`** — SSH into the running container's zellij
   session
3. **`aibox stop --remote user@host`** — stop the remote container
4. Assume project code is already on the remote (cloned via git) — do not build a
   full file sync system

Implementation requires an SSH command execution module (~300-500 lines of Rust using
the `openssh` or `ssh2` crate) and modifications to `container.rs` to dispatch
compose commands over SSH instead of locally.

### Phase 3: GPU and cloud (later, via DevPod or BACK-068)

- GPU passthrough config in aibox.toml (`[container.gpu]`) that generates the
  appropriate compose `deploy` section
- Kubernetes deployment via BACK-068 (Helm chart scaffolding)
- Cloud VM provisioning is best left to DevPod providers — do not duplicate this
  in the aibox CLI

### What NOT to Build

- **A server component** — aibox's strength is being client-only, like DevPod.
  Adding a server (dashboard, team management) puts aibox in competition with Coder
  and Gitpod without clear differentiation.
- **Browser-based IDE** — aibox's identity is zellij + vim in the terminal.
  code-server/Theia integration adds complexity for a use case that DevPod and Coder
  already serve.
- **Cloud SDK integration** — provisioning VMs on AWS/GCP/Azure is DevPod's job.
  Building this into aibox would be a massive scope expansion with high maintenance
  burden.
- **Proprietary remote protocol** — use SSH. It is universal, secure, and every
  IDE (VS Code, JetBrains, terminal) already knows how to use it.

### Decision Summary

| Decision | Choice | Rationale |
|---|---|---|
| Primary remote pattern | SSH-based | Lowest complexity, aligns with zellij-centric model |
| File sync strategy | Git-based (code on remote) + rsync (config files) | Simple, reliable, no new dependencies |
| User connection method | SSH + zellij (primary), VS Code/JetBrains via SSH (secondary) | Zero additional components for primary path |
| Platform integration | DevPod (Phase 1), native SSH (Phase 2) | DevPod compatibility is free; native SSH adds convenience |
| Secrets | SSH agent forwarding + remote .env files | Standard, secure, no custom protocol |
| GPU | Compose `deploy` section, document requirements | Keep it simple, defer cloud provisioning |
| K8s deployment | Separate effort (BACK-068) | Different use case, different timeline |

---

## Sources

- [VS Code Remote Development](https://code.visualstudio.com/docs/remote/remote-overview)
- [VS Code Dev Containers on Remote Host](https://code.visualstudio.com/remote/advancedcontainers/develop-remote-host)
- [GitHub Codespaces Billing](https://docs.github.com/billing/managing-billing-for-github-codespaces/about-billing-for-github-codespaces)
- [GitHub Codespaces Features](https://github.com/features/codespaces)
- [Gitpod GitHub Repository](https://github.com/gitpod-io/gitpod)
- [Gitpod SaaS Deprecation](https://www.harness.io/blog/gitpod-saas-is-being-deprecated-what-are-your-options)
- [JetBrains Gateway](https://www.jetbrains.com/remote-development/gateway/)
- [JetBrains Debugger Architecture Redesign for 2026.1](https://blog.jetbrains.com/platform/2026/01/platform-debugger-architecture-redesign-for-remote-development-in-2026-1/)
- [Coder Open Source](https://github.com/coder/coder)
- [Coder Templates Documentation](https://coder.com/docs/admin/templates)
- [Coder Dev Containers Support](https://coder.com/docs/user-guides/devcontainers)
- [DevPod GitHub Repository](https://github.com/loft-sh/devpod)
- [DevPod How It Works](https://devpod.sh/docs/how-it-works/overview)
- [DevPod Provider Development](https://devpod.sh/docs/developing-providers/quickstart)
- [DevPod Agent Architecture](https://devpod.sh/docs/developing-providers/agent)
- [DevPod SSH Devcontainers](https://fabiorehm.com/blog/2025/11/11/devpod-ssh-devcontainers/)
- [Hocus GitHub Repository (archived)](https://github.com/hocus-dev/hocus)
- [Eclipse Theia vs VS Code Comparison](https://markaicode.com/eclipse-theia-vs-vscode-self-hosted-comparison/)
- [Mutagen File Synchronization](https://github.com/mutagen-io/mutagen)
- [Remote Development Platforms 2025](https://diploi.com/blog/remote_development_platforms)
- [Gitpod vs Codespaces vs Coder vs DevPod Comparison](https://www.vcluster.com/blog/comparing-coder-vs-codespaces-vs-gitpod-vs-devpod)
- [Cloud GPU Providers 2026](https://www.runpod.io/articles/guides/top-cloud-gpu-providers)
- [State of Cloud GPUs 2025](https://dstack.ai/blog/state-of-cloud-gpu-2025/)
