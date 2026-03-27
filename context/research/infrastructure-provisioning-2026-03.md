# Infrastructure Provisioning for Self-Hosted aibox — Research Report — March 2026

Research for self-hosted deployment infrastructure. Investigates how users go from
"I have a cloud account" to "my aibox environment is running remotely." Covers IaC
tools, cloud provider comparisons, GPU provisioning, deployment patterns, and where
aibox's responsibility boundary should be. Conducted 2026-03-26.

Related research:
- Remote development patterns (BACK-011): `remote-development-2026-03.md`
- Kubernetes deployment (BACK-068): `kubernetes-deployment-2026-03.md`
- Self-hosted AI models (BACK-094): in progress

---

## 1. Infrastructure-as-Code Tools for Dev Environment Provisioning

### 1.1 OpenTofu / Terraform

OpenTofu is the open-source fork of Terraform (MPL-licensed), created after
HashiCorp changed Terraform to the Business Source License (BUSL) in 2023. Both
use HCL (HashiCorp Configuration Language) and share the same provider ecosystem.
For aibox, OpenTofu is the recommended choice due to its open-source license
alignment.

| Aspect | OpenTofu | Terraform |
|---|---|---|
| License | MPL 2.0 (open source) | BUSL 1.1 (source-available) |
| Provider compatibility | Same providers — uses Terraform plugin protocol v6 | Same providers |
| State management | Local, S3, GCS, Hetzner S3, etc. | Same + HCP Terraform (paid) |
| Testing | `tofu test` (GA) | `terraform test` (GA since 1.6) |
| Community | Linux Foundation, growing | HashiCorp, mature |
| Key difference for aibox | No licensing concerns for bundling/shipping | BUSL restricts competing products |

**Relevant providers for aibox users:**

| Provider | Registry name | Maturity | Notes |
|---|---|---|---|
| Hetzner Cloud | `hetznercloud/hcloud` | Stable (v1.49+) | Official, well-maintained |
| AWS | `hashicorp/aws` | Mature | Most-used provider in ecosystem |
| GCP | `hashicorp/google` | Mature | Full Compute Engine support |
| DigitalOcean | `digitalocean/digitalocean` | Stable | Straightforward API |
| Vultr | `vultr/vultr` | Stable | Good value provider |
| OVH | `ovh/ovh` | Stable | EU-focused |
| Scaleway | `scaleway/scaleway` | Stable | EU-focused, GPU instances |

A minimal Tofu module for a dev server requires approximately 50-80 lines of HCL:
SSH key, firewall, server resource, and cloud-init for Docker installation.

### 1.2 Ansible for Post-Provisioning

Ansible handles what Tofu cannot: software installation and configuration after
the VM exists. The typical post-provisioning playbook for an aibox-ready server:

1. Install Docker (via `apt` + official Docker GPG key + repository)
2. Install NVIDIA Container Toolkit (if GPU server)
3. Configure SSH (disable password auth, set authorized keys)
4. Configure firewall (`ufw` — allow SSH, deny all else)
5. Create workspace directory with correct permissions
6. Pull aibox base image (optional pre-warm step)

A well-known reference: DigitalOcean's "Use Ansible to Install Docker on Ubuntu"
tutorial provides a production-quality playbook pattern. For security hardening,
community playbooks demonstrate disabling SSH password authentication, moving SSH
off port 22, and configuring fail2ban.

**Tofu + Ansible workflow:**

```
tofu apply          →  VM exists, has IP, SSH key installed
ansible-playbook    →  Docker installed, firewall configured, ready for aibox
aibox start --remote user@<ip>  →  Environment running
```

Ansible's `community.docker` collection provides native Docker integration for
tasks like pre-pulling images.

### 1.3 Pulumi as an Alternative to Tofu

Pulumi uses real programming languages (TypeScript, Python, Go, Java) instead of
HCL. This appeals to developers who already know these languages and want loops,
conditionals, and unit testing in their infrastructure code.

| Aspect | OpenTofu/Terraform | Pulumi |
|---|---|---|
| Language | HCL (DSL) | TypeScript, Python, Go, Java, .NET |
| Learning curve | New language for most devs | Familiar if you know the language |
| Testing | `tofu test` (limited) | Jest, pytest, Go testing (full) |
| Provider ecosystem | 3,000+ providers | Uses Terraform providers via bridge |
| State management | File or remote backend | Pulumi Cloud (free tier) or self-managed |
| Community size | Larger (Terraform legacy) | Smaller but growing |
| Typing/IDE support | HCL LSP (decent) | Full IDE support via language tooling |

**Assessment for aibox:** If aibox ships reference IaC modules, HCL/Tofu is the
safer choice due to broader adoption and simpler mental model for infrastructure
(declarative, no imperative logic needed for a single VM). Pulumi would be a
better fit if aibox were building a complex multi-resource orchestration system,
which it should not.

### 1.4 How Existing Platforms Handle Infrastructure Provisioning

| Platform | Provisioning approach | User experience |
|---|---|---|
| **Coder** | Terraform templates define workspaces. Templates bundled with Coder or custom. Supports Docker, K8s, AWS, GCP, Azure. | Admin writes Terraform template, users click "Create Workspace" |
| **Gitpod (Ona)** | CloudFormation template for AWS. Creates VPC, subnets, ALB, EC2 ASGs, RDS, IAM. K8s clusters underneath. | Customer runs CloudFormation, Gitpod manages scaling |
| **DevPod** | Provider plugins (Go binaries). Machine providers create VMs via cloud APIs. No Terraform — direct API calls. | User selects provider, DevPod creates VM and deploys agent |
| **GitHub Codespaces** | Fully managed — user never sees infrastructure. Microsoft provisions VMs internally. | Zero infrastructure awareness |

**Key insight:** Coder uses Terraform as its provisioning layer — exactly the
pattern aibox could adopt for reference modules. DevPod uses direct API calls
via provider plugins, which is simpler for users but requires maintaining
provider code for each cloud. Gitpod's CloudFormation approach is heavy
(designed for enterprise with K8s clusters, RDS, etc.) and not applicable
to aibox's single-VM use case.

---

## 2. Cloud Provider Comparison — Minimum Viable Dev Server

### 2.1 Requirements

**Coding-only (no GPU):**
- 4 vCPU, 8 GB RAM, 50 GB SSD (minimum)
- Linux (Ubuntu 24.04 LTS preferred)
- Docker/Podman installed
- SSH access with key-based auth
- Firewall: SSH only (port 22), optionally HTTP for preview companion

**AI workloads (GPU):**
- All of the above, plus:
- NVIDIA GPU with 16+ GB VRAM
- NVIDIA drivers + Container Toolkit
- `docker run --gpus all` functional

### 2.2 Coding-Only Dev Server Pricing (4 vCPU / 8 GB RAM)

Prices as of March 2026. Hetzner prices increase ~30-35% on April 1, 2026.

| Provider | Instance type | vCPU | RAM | Disk | Monthly cost | Hourly cost | Notes |
|---|---|---|---|---|---|---|---|
| **Hetzner Cloud** | CX32 (shared) | 4 | 8 GB | 80 GB | ~€7.49 | €0.011 | EU only. Cheapest option. Pre-April pricing. |
| **Hetzner Cloud** | CAX21 (ARM) | 4 | 8 GB | 80 GB | ~€7.49 | €0.011 | ARM (Ampere). Even more efficient. EU only. |
| **Hetzner Cloud** | CPX31 (shared) | 4 | 8 GB | 160 GB | ~€12.99 | €0.019 | AMD EPYC, more disk. EU + US. |
| **Vultr** | Regular Cloud | 4 | 8 GB | 100 GB | $48/mo | $0.071 | 17 regions globally |
| **DigitalOcean** | General Purpose | 4 | 8 GB | 25 GB | $63/mo | ~$0.094 | Per-second billing since Jan 2026 |
| **Scaleway** | DEV1-L | 4 | 8 GB | 80 GB | ~€12/mo | €0.018 | EU only (Paris, Amsterdam) |
| **OVH** | B2-15 | 4 | 15 GB | 100 GB | ~€26/mo | €0.039 | EU, CA, APAC regions |
| **AWS** | t3.xlarge | 4 | 16 GB | 50 GB EBS | ~$122/mo | $0.1664 | + EBS cost ($5/mo for 50GB gp3) |
| **AWS** | t3.xlarge (Spot) | 4 | 16 GB | 50 GB EBS | ~$37-49/mo | ~$0.05-0.067 | 60-70% savings, can be interrupted |
| **GCP** | e2-standard-4 | 4 | 16 GB | 50 GB PD | ~$97/mo | $0.134 | + disk cost |

**Verdict:** Hetzner Cloud is 5-15x cheaper than hyperscalers for a coding-only
dev server. For aibox's target audience (individual developers, small teams),
Hetzner is the recommended default. Scaleway is a good EU alternative. AWS/GCP
are only justified when users need specific cloud services or have existing
accounts with credits.

### 2.3 GPU Server Pricing

| Provider | GPU | VRAM | vCPU | RAM | Monthly cost | Hourly cost | Notes |
|---|---|---|---|---|---|---|---|
| **Hetzner** | RTX 4000 SFF Ada | 20 GB | 8 | 32 GB | €184/mo | N/A (dedicated) | GEX44. €79 setup fee. Dedicated server, not cloud VM. |
| **AWS** | T4 (g4dn.xlarge) | 16 GB | 4 | 16 GB | ~$380/mo | $0.526 | On-demand. Spot ~$0.16-0.21/hr |
| **AWS** | A10G (g5.xlarge) | 24 GB | 4 | 16 GB | ~$724/mo | $1.006 | Best general-purpose GPU on AWS |
| **GCP** | T4 | 16 GB | 4 | 15 GB | ~$360/mo | $0.35 (GPU) + compute | Preemptible available for savings |
| **GCP** | A100 80GB | 80 GB | 12 | 85 GB | ~$2,500/mo | ~$3.67 | A2-ultragpu-1g |
| **vast.ai** | A100 40GB | 40 GB | varies | varies | ~$374/mo | $0.52 | Marketplace pricing, variable |
| **vast.ai** | L40 | 48 GB | varies | varies | ~$223/mo | $0.31 | Marketplace pricing, variable |
| **RunPod** | A100 80GB | 80 GB | varies | varies | varies | ~$1.64 | On-demand, Docker-native |
| **RunPod** | RTX 4090 | 24 GB | varies | varies | varies | ~$0.44 | Community cloud pricing |

**Verdict for GPU:** Hetzner's GEX44 (€184/mo) is excellent value for persistent
GPU dev work but is a dedicated server (not cloud VM — longer provisioning time,
setup fee). For on-demand GPU, vast.ai and RunPod offer the best price-performance.
AWS/GCP are 2-4x more expensive but offer better reliability and integration with
cloud ecosystems.

### 2.4 Provider-Specific Setup Details

#### Hetzner Cloud (Recommended Default)

**CLI tool:** `hcloud` (Go binary, installable via brew/apt)

```bash
# Minimal setup via CLI
hcloud ssh-key create --name mykey --public-key-from-file ~/.ssh/id_ed25519.pub
hcloud firewall create --name aibox-fw \
  --rules-file <(echo '[{"direction":"in","protocol":"tcp","port":"22","source_ips":["0.0.0.0/0","::/0"]}]')
hcloud server create \
  --name aibox-dev \
  --type cx32 \
  --image ubuntu-24.04 \
  --ssh-key mykey \
  --firewall aibox-fw \
  --user-data-from-file cloud-init.yml
```

**OpenTofu module (~60 lines):**

```hcl
terraform {
  required_providers {
    hcloud = { source = "hetznercloud/hcloud", version = "~> 1.49" }
  }
}

variable "hcloud_token" { sensitive = true }
variable "ssh_public_key" {}

provider "hcloud" { token = var.hcloud_token }

resource "hcloud_ssh_key" "default" {
  name       = "aibox"
  public_key = var.ssh_public_key
}

resource "hcloud_firewall" "default" {
  name = "aibox"
  rule {
    direction  = "in"
    protocol   = "tcp"
    port       = "22"
    source_ips = ["0.0.0.0/0", "::/0"]
  }
}

resource "hcloud_server" "dev" {
  name        = "aibox-dev"
  server_type = "cx32"
  image       = "ubuntu-24.04"
  location    = "fsn1"
  ssh_keys    = [hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.default.id]

  user_data = file("cloud-init.yml")
}

output "ip" { value = hcloud_server.dev.ipv4_address }
```

**cloud-init.yml** (installs Docker, hardens SSH):

```yaml
#cloud-config
package_update: true
packages: [docker.io, docker-compose-plugin, ufw, fail2ban]
runcmd:
  - systemctl enable --now docker
  - ufw default deny incoming
  - ufw default allow outgoing
  - ufw allow 22/tcp
  - ufw --force enable
  - usermod -aG docker ubuntu
ssh_pwauth: false
```

**Estimated cost:** CX32 at €7.49/mo = ~€90/year for a full-time dev server.
This is exceptionally cheap compared to GitHub Codespaces (~$40/mo for similar
specs) or AWS (~$120/mo).

#### AWS

**Minimum viable setup:** EC2 instance + security group + SSH key pair + EBS volume.

```hcl
resource "aws_instance" "dev" {
  ami           = "ami-0c55b159cbfafe1f0"  # Ubuntu 24.04
  instance_type = "t3.xlarge"
  key_name      = aws_key_pair.dev.key_name

  vpc_security_group_ids = [aws_security_group.dev.id]

  root_block_device {
    volume_size = 50
    volume_type = "gp3"
  }

  user_data = file("cloud-init.yml")
}

resource "aws_security_group" "dev" {
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
```

**Spot instances** can save 60-70% but may be interrupted. Suitable for
non-persistent dev work. Not recommended for aibox's primary use case
(persistent development environment) unless combined with EBS persistence.

**GPU:** `g5.xlarge` (A10G, 24GB VRAM) at ~$1.006/hr on-demand, ~$0.30-0.40/hr
spot. Requires AMI with NVIDIA drivers pre-installed (AWS Deep Learning AMI) or
cloud-init installation of NVIDIA Container Toolkit.

#### GCP

Similar pattern to AWS. Compute Engine instance + firewall rule + SSH key.
Preemptible/Spot VMs offer 60-91% discount with 24-hour max runtime.

**GPU:** Attach T4 ($0.35/hr) or A100 to a VM. GCP's Container-Optimized OS
comes with Docker pre-installed, reducing post-provisioning steps.

#### DigitalOcean

Simplest API/CLI of all providers. `doctl compute droplet create` with a few
flags. No VPC/subnet complexity. GPU Droplets are enterprise-only with custom
pricing — not suitable for individual aibox users.

---

## 3. Infrastructure Requirements for aibox

### 3.1 Minimum Requirements (Coding-Only)

| Requirement | Specification | Why |
|---|---|---|
| OS | Linux (Ubuntu 22.04+ or Debian 12+) | Container runtime compatibility |
| Docker/Podman | Docker 24+ or Podman 4+ | aibox generates compose files |
| CPU | 2+ vCPU (4 recommended) | Compilation, language servers, AI tools |
| RAM | 4 GB minimum (8 recommended) | Container overhead + workspace tools |
| Disk | 20 GB minimum (50 recommended) | Base image (~2-4GB) + workspace + packages |
| SSH | Port 22, key-based auth only | Remote access, IDE connection |
| Firewall | Deny all except SSH | Security baseline |

### 3.2 GPU Requirements (AI Workloads)

| Requirement | Specification | Why |
|---|---|---|
| GPU | NVIDIA (CUDA-capable) | Industry standard for AI/ML |
| VRAM | 16 GB minimum (24+ recommended) | LLM inference, fine-tuning |
| NVIDIA Driver | 535+ | Container Toolkit compatibility |
| NVIDIA Container Toolkit | Latest | `docker run --gpus all` support |
| RAM | 16 GB minimum (32 recommended) | GPU workloads need host RAM too |
| Disk | 100 GB+ | Model weights are large (7B model ~14GB) |

### 3.3 NVIDIA Container Toolkit Setup

The toolkit enables GPU access inside Docker containers. Installation steps:

```bash
# Add NVIDIA repository
curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | \
  sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \
  sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
  sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list

# Install
sudo apt-get update && sudo apt-get install -y nvidia-container-toolkit

# Configure Docker runtime
sudo nvidia-ctk runtime configure --runtime=docker
sudo systemctl restart docker

# Verify
docker run --rm --gpus all nvidia/cuda:12.3.1-base-ubuntu22.04 nvidia-smi
```

For aibox compose files, GPU access is declared via:

```yaml
services:
  workspace:
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
```

### 3.4 Networking

| Port | Purpose | Required? |
|---|---|---|
| 22 (TCP) | SSH access | Yes |
| 8080-8099 (TCP) | Preview companion (PROJ-004) | Optional |
| 443 (TCP) | HTTPS for web services | Optional |

For preview companion ports, the firewall rule can be added when needed rather
than opened by default. Security principle: minimal open ports.

### 3.5 Persistent Storage

The workspace volume must survive VM restarts. Two approaches:

1. **Local disk** (simplest): workspace lives on the VM's root disk or an
   attached volume. Survives restarts but lost if VM is deleted.
2. **Separate block storage**: Hetzner Volumes, AWS EBS, GCP Persistent Disk.
   Can be detached and reattached to new VMs. More resilient but adds cost
   and complexity.

For most aibox users, local disk is sufficient — the workspace is a git
repository, so the authoritative copy is on the remote origin.

---

## 4. Deployment Patterns — Where Does aibox's Responsibility End?

### 4.1 Pattern Comparison

| Pattern | Description | Effort | User complexity | aibox scope |
|---|---|---|---|---|
| **A: Docs only** | aibox documents manual VM setup, user runs `aibox start --remote` | Zero | High (user provisions manually) | Container layer only |
| **B: Reference Tofu modules** | aibox ships optional Tofu modules. User runs `tofu apply` then `aibox start --remote` | Low-Medium | Medium (user runs IaC) | Container + reference infra |
| **C: Integrated provisioning** | `aibox provision hetzner` creates VM, installs Docker, deploys env | High | Low (aibox handles everything) | Full lifecycle |
| **D: DevPod provider** | aibox generates devcontainer.json, DevPod handles provisioning | Zero-Low | Low (DevPod handles infra) | Container layer only |
| **E: K8s-only** | User brings K8s cluster, aibox deploys via Helm (BACK-068) | Medium | Medium (user manages K8s) | Container + Helm |

### 4.2 Pattern A: Docs Only

**How it works:** aibox documentation includes a guide: "Set up a remote dev
server on Hetzner/AWS/GCP." User follows the guide manually (or uses their own
IaC), ends up with a VM that has Docker + SSH. Then runs `aibox start --remote
user@host` (per BACK-011).

**Pros:**
- Zero code to maintain
- No cloud SDK dependencies in aibox binary
- Users are not locked into aibox's infrastructure opinions
- Works with any provider, including ones aibox has never heard of

**Cons:**
- High friction for users who are not familiar with cloud provisioning
- No reproducibility — each user's setup is different
- Cannot automate VM lifecycle (start/stop/delete)

**Best for:** Experienced developers who already have cloud infrastructure.

### 4.3 Pattern B: Reference Tofu Modules

**How it works:** aibox ships a `deploy/tofu/` directory (in the aibox repo or a
separate `aibox-infra` repo) with OpenTofu modules for each supported provider.
Users clone, set variables, and `tofu apply`. The output is an IP address they
pass to `aibox start --remote`.

**Example structure:**

```
deploy/tofu/
  hetzner/
    main.tf
    variables.tf
    outputs.tf
    cloud-init.yml
  aws/
    main.tf
    variables.tf
    outputs.tf
    cloud-init.yml
  gcp/
    main.tf
    ...
```

**Pros:**
- Reproducible infrastructure with minimal user effort
- Tofu modules are well-understood, versionable, testable
- aibox does not need cloud SDK dependencies — Tofu is a separate tool
- Users can customize modules (add VPN, change instance type, etc.)
- Ansible post-provisioning can be bundled for Docker/toolkit setup

**Cons:**
- Requires users to install OpenTofu (or Terraform)
- Requires cloud API tokens configured outside aibox
- More code to maintain (one module per provider)
- Two-step workflow: `tofu apply` then `aibox start --remote`

**Best for:** The sweet spot between simplicity and automation. Most aibox
users can follow a `tofu apply` workflow.

### 4.4 Pattern C: Integrated Provisioning

**How it works:** `aibox provision hetzner --type cx32 --region fsn1` creates
a VM, waits for it to boot, installs Docker via SSH, and returns the IP. Then
`aibox start --remote` (or auto-chained) deploys the environment.

**Implementation requirements:**
- Cloud SDK/API client for each provider (Hetzner: `hcloud` crate or HTTP API)
- SSH client for post-provisioning (install Docker, configure firewall)
- VM lifecycle management (start, stop, delete, list)
- Credential management (API tokens stored securely)
- State tracking (which VMs were created, their IPs, status)

**Pros:**
- Single-command experience: `aibox up --cloud hetzner`
- Full lifecycle management from one tool
- Can optimize for aibox-specific needs (pre-pull images, configure GPU)

**Cons:**
- Massive scope expansion — essentially building a mini-cloud orchestrator
- Each provider requires a separate API integration
- Cloud SDK dependencies bloat the CLI binary
- Credential management is a security-sensitive surface
- Duplicates what DevPod already does well
- State management for provisioned VMs adds complexity

**Best for:** A future where aibox has a large team and wants to compete with
DevPod/Coder on integrated experience. Not recommended for current stage.

### 4.5 Pattern D: DevPod Provider

**How it works:** Users install DevPod, add a cloud provider (Hetzner, AWS,
GCP, DO — all have existing DevPod providers), and point DevPod at a project
with aibox-generated `.devcontainer/`. DevPod creates the VM, deploys its
agent, builds the container, and provides SSH access.

**DevPod providers already exist for:**
- AWS (official, uses EC2)
- GCP (official, uses Compute Engine)
- Azure (official)
- DigitalOcean (official)
- Hetzner (community, `mrsimonemms/devpod-provider-hetzner`)
- Kubernetes (official)
- SSH (official — bring your own VM)

**How DevPod machine providers work:**
1. User runs `devpod up . --provider hetzner`
2. DevPod provider creates VM via Hetzner API
3. DevPod deploys its agent binary to the VM via SSH
4. Agent installs Docker, builds the devcontainer image
5. Agent starts the container, sets up SSH tunnel
6. User connects via `devpod ssh` or IDE

**Pros:**
- Zero aibox code needed — devcontainer.json already works
- DevPod handles VM lifecycle, SSH tunneling, IDE integration
- Providers for all major clouds already exist
- Apache 2.0 license, actively maintained
- Auto-shutdown on inactivity (saves money)

**Cons:**
- Requires users to install DevPod (additional tool)
- Hetzner provider is community-maintained (not official)
- DevPod's zellij integration is untested (DevPod expects IDE connection)
- aibox loses control of the provisioning experience

**Best for:** Users who want cloud provisioning without aibox building it.
This is the recommended path for Phase 1.

### 4.6 Pattern E: K8s-Only

Covered by BACK-068 (Kubernetes deployment research). User brings a K8s cluster,
aibox generates Helm charts. No VM provisioning — the cluster is pre-existing.

---

## 5. The Boundary Question — Should aibox Be Opinionated About Infrastructure?

### 5.1 How Comparable Tools Handle This

| Tool | Infrastructure stance | Analogy |
|---|---|---|
| **uv** (Python) | Zero — installs packages, does not provision servers | Package manager |
| **Docker/Podman** | Zero — runs containers on whatever host exists | Container runtime |
| **DevPod** | Full — provisions VMs via provider plugins | Container runtime + IaC |
| **Coder** | Full — Terraform templates provision workspaces | Platform |
| **Gitpod** | Full — CloudFormation creates AWS infrastructure | Managed platform |
| **Nix** | Zero — defines environments, does not provision hosts | Environment manager |

### 5.2 The "uv for AI" Philosophy

aibox positions itself as "uv for AI work environments." uv does not provision
servers — it manages Python environments on whatever machine it runs on. By this
analogy, aibox should:

- **Own:** Container definition (devcontainer.json, compose), AI tool configuration,
  workspace setup, addon management
- **Not own:** VM provisioning, cloud API integration, infrastructure lifecycle
- **Enable:** Remote execution via SSH (`--remote`), compatibility with tools that
  do handle infrastructure (DevPod, Coder, Tofu)

This means aibox's infrastructure responsibility ends at the Docker socket. If
there is a Linux machine with Docker and SSH, aibox can work on it. How that
machine came to exist is outside aibox's scope.

### 5.3 But Users Need a Path

"Not our problem" is not a product strategy. Users who discover aibox and want
to run it remotely need a clear, tested path. The question is whether that path
is built into aibox or documented alongside it.

**The answer: Document and reference, do not build.**

aibox should:
1. Ship reference Tofu modules (Pattern B) for the most common providers
2. Document DevPod compatibility (Pattern D) as the easiest path
3. Provide clear "getting started remotely" documentation (Pattern A)
4. Never integrate cloud SDKs into the CLI binary (reject Pattern C)

---

## 6. GPU Provisioning Specifically

### 6.1 GPU-Ready Cloud Options

| Provider | Docker-ready? | NVIDIA Toolkit pre-installed? | Provisioning speed | Best for |
|---|---|---|---|---|
| **Hetzner GEX44** | No (dedicated server, manual setup) | No | Hours (dedicated setup) | Persistent GPU dev work |
| **AWS g5.xlarge** | Via Deep Learning AMI (yes) | Yes (DLAMI) | Minutes | On-demand GPU with cloud integration |
| **GCP T4/A100** | Via Container-Optimized OS (partial) | Partial (driver yes, toolkit manual) | Minutes | GCP-native workflows |
| **vast.ai** | Yes (Docker-native) | Yes | Minutes | Cheapest GPU, variable reliability |
| **RunPod** | Yes (Docker-native) | Yes | Sub-minute | Docker-native GPU, good UX |
| **Lambda Labs** | Yes | Yes | Minutes | GPU-focused cloud |

### 6.2 vast.ai and RunPod — Docker-Ready GPU VMs

Both platforms are specifically designed for containerized GPU workloads:

**vast.ai:**
- Marketplace model — hosts set prices, supply/demand driven
- Docker instances with exclusive GPU access, per-second billing
- SSH or Jupyter connectivity
- A100 40GB from ~$0.52/hr, L40 from ~$0.31/hr
- Also offers full VM rental alongside Docker instances
- Variable reliability (depends on host)

**RunPod:**
- Purpose-built GPU cloud with Docker-native deployment
- GPU rentals from $0.20/hr (community cloud) to $1.64/hr (A100 secure cloud)
- Sub-250ms cold start times
- `runpodctl` CLI for deployment
- More consistent than vast.ai, slightly more expensive

**Both are ideal for aibox GPU workloads** because they provide Docker-ready
environments with NVIDIA drivers pre-configured. No post-provisioning needed
for GPU access — just `docker run --gpus all`.

### 6.3 GPU Passthrough to aibox Containers

Once the host has NVIDIA Container Toolkit installed, aibox containers need:

1. **compose deploy section** (already possible in aibox's compose generation):
   ```yaml
   deploy:
     resources:
       reservations:
         devices:
           - driver: nvidia
             count: all
             capabilities: [gpu]
   ```

2. **aibox.toml configuration** (proposed):
   ```toml
   [container.gpu]
   enabled = true
   count = "all"     # or specific number
   # capabilities = ["gpu", "compute"]  # optional fine-tuning
   ```

3. **Verification** inside the container:
   ```bash
   nvidia-smi  # Should show GPU
   python -c "import torch; print(torch.cuda.is_available())"  # Should be True
   ```

---

## 7. Recommendation

### Phase 1: Documentation + DevPod Compatibility (Zero Code)

**Effort:** 1-2 days (documentation only)

1. **Document DevPod as the easiest remote path.** aibox already generates
   compliant devcontainer.json. Test and document:
   ```bash
   devpod up . --provider hetzner   # or aws, gcp, ssh
   devpod ssh <workspace>
   ```
2. **Write a "Remote Development Quick Start" docs page** covering:
   - Prerequisites (cloud account, API token, DevPod installed)
   - Provider setup (Hetzner recommended for cost)
   - Launching a remote aibox environment via DevPod
   - Connecting via SSH + zellij, VS Code, or JetBrains
3. **Test the zellij experience** through DevPod's SSH tunnel. If it works
   well, this becomes the primary recommendation. If not, document the
   SSH provider alternative (user provisions VM manually, DevPod connects
   via SSH).

### Phase 2: Reference Tofu Modules (Low Code, Separate Repo)

**Effort:** 3-5 days

1. **Create `aibox-infra` repository** (or `deploy/tofu/` in main repo) with
   OpenTofu modules for:
   - **Hetzner Cloud** (primary — cheapest, simplest)
   - **AWS EC2** (secondary — most users have an account)
   - Cloud-init script shared across providers
2. **Each module produces:**
   - A VM with Docker installed
   - SSH key configured
   - Firewall (SSH-only by default)
   - Output: IP address, SSH connection string
3. **Include an Ansible playbook** (optional) for post-provisioning tasks
   that cloud-init cannot handle (NVIDIA toolkit, custom config).
4. **Document the two-step workflow:**
   ```bash
   cd deploy/tofu/hetzner
   tofu init && tofu apply -var="hcloud_token=$HCLOUD_TOKEN"
   # Output: ip = "1.2.3.4"
   aibox start --remote ubuntu@1.2.3.4
   ```

### Phase 3: GPU Configuration in aibox.toml (Medium Code)

**Effort:** 1-2 days

1. **Add `[container.gpu]` section** to aibox.toml spec
2. **Generate compose `deploy.resources.reservations`** when GPU enabled
3. **Document GPU provider recommendations:**
   - Persistent: Hetzner GEX44 (€184/mo)
   - On-demand cheap: vast.ai, RunPod
   - On-demand reliable: AWS g5, GCP T4
4. **Add GPU variant to reference Tofu modules** (AWS g5.xlarge with
   Deep Learning AMI, cloud-init installs NVIDIA Container Toolkit)

### What NOT to Build

- **`aibox provision <cloud>`** — this is DevPod's job. Building cloud API
  integrations into aibox would be a massive scope expansion that duplicates
  existing open-source tooling.
- **Cloud SDK dependencies** — adding `aws-sdk`, `hcloud`, or `gcloud` crates
  to the CLI binary would bloat it and add maintenance burden for each provider.
- **VM state management** — tracking which VMs were provisioned, their IPs,
  and lifecycle status is infrastructure orchestration, not environment management.
- **Billing/cost estimation** — cloud pricing changes frequently. Document it,
  do not build it into the tool.

### Decision Summary

| Decision | Choice | Rationale |
|---|---|---|
| Infrastructure ownership | aibox does NOT own infrastructure | "uv for AI" philosophy — stay at container layer |
| Easiest remote path | DevPod + cloud provider | Zero aibox code, DevPod handles provisioning |
| Reference IaC | OpenTofu modules (Hetzner + AWS) | Reproducible, versionable, user-customizable |
| IaC language | HCL (OpenTofu) over Pulumi | Broader adoption, simpler for single-VM use case |
| Default cloud provider | Hetzner Cloud | 5-15x cheaper than hyperscalers for dev servers |
| GPU path | Document providers + add `[container.gpu]` to config | Keep it declarative, let hosting be user's choice |
| Post-provisioning | cloud-init (primary) + Ansible (optional) | cloud-init covers 90% of setup, Ansible for advanced |
| Pattern C (integrated provisioning) | Reject | Duplicates DevPod, massive scope, wrong layer for aibox |

---

## Sources

- [Hetzner Cloud Provider for OpenTofu](https://github.com/hetznercloud/terraform-provider-hcloud)
- [How to Create Hetzner Cloud Servers with OpenTofu](https://oneuptime.com/blog/post/2026-03-20-create-hetzner-cloud-servers-opentofu/view)
- [How to Configure the Hetzner Cloud Provider in OpenTofu](https://oneuptime.com/blog/post/2026-03-20-hetzner-cloud-provider-opentofu/view)
- [OpenTofu and Terraform Registry — hcloud Provider](https://library.tf/providers/hetznercloud/hcloud/latest)
- [Hetzner Cloud Pricing](https://www.hetzner.com/cloud)
- [Hetzner Cloud VPS Pricing Calculator (Mar 2026)](https://costgoat.com/pricing/hetzner)
- [Hetzner Cloud Review 2026: Benchmarks, Pricing, and the Real Trade-offs](https://betterstack.com/community/guides/web-servers/hetzner-cloud-review/)
- [Hetzner Price Adjustment — April 2026](https://docs.hetzner.com/general/infrastructure-and-availability/price-adjustment/)
- [Hetzner GPU Servers — GEX44](https://www.hetzner.com/dedicated-rootserver/gex44/)
- [Hetzner Cost-Optimized Cloud Plans](https://www.hetzner.com/cloud/cost-optimized)
- [AWS EC2 On-Demand Instance Pricing](https://aws.amazon.com/ec2/pricing/on-demand/)
- [AWS EC2 Spot Instances Pricing](https://aws.amazon.com/ec2/spot/pricing/)
- [AWS GPU Instance Pricing Comparison](https://compute.doit.com/gpu)
- [Amazon EC2 Instance Comparison](https://instances.vantage.sh/)
- [GCP GPU Pricing](https://cloud.google.com/compute/gpus-pricing)
- [GCP Compute Engine Pricing](https://cloud.google.com/compute/all-pricing)
- [Google Cloud GPU Pricing 2026](https://gpucost.org/provider/gcp)
- [DigitalOcean Droplet Pricing](https://www.digitalocean.com/pricing/droplets)
- [DigitalOcean Pricing 2026](https://onedollarvps.com/pricing/digitalocean-pricing)
- [Vultr Pricing](https://www.vultr.com/pricing/)
- [Scaleway Pricing](https://www.scaleway.com/en/pricing/)
- [VPS Price Comparison (Updated Mar 2026)](https://getdeploying.com/reference/compute-prices)
- [IaC in 2026: Terraform/OpenTofu vs Pulumi](https://peerobyte.com/blog/infrastructure-as-code-in-2026-terraform-opentofu-vs-pulumi-and-common-mistakes/)
- [Terraform vs Pulumi — Pulumi Docs](https://www.pulumi.com/docs/iac/comparisons/terraform/)
- [Terraform vs Pulumi vs OpenTofu — IaC Comparison 2026](https://eitt.academy/knowledge-base/terraform-vs-pulumi-vs-opentofu-iac-comparison-2026/)
- [Top 12 Cloud Provisioning Tools in 2026](https://spacelift.io/blog/cloud-provisioning-tools)
- [DevPod — Add a Provider](https://devpod.sh/docs/managing-providers/add-provider)
- [DevPod Provider Agent Architecture](https://devpod.sh/docs/developing-providers/agent)
- [DevPod Hetzner Provider](https://github.com/mrsimonemms/devpod-provider-hetzner)
- [Gitpod — Self-hosted, Not Self-managed](https://www.gitpod.io/blog/self-hosted-not-self-managed)
- [Gitpod Self-Hosted Reference Architectures](https://www.gitpod.io/docs/configure/self-hosted/latest/reference-architecture)
- [NVIDIA Container Toolkit — Install Guide](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html)
- [NVIDIA Container Toolkit — GitHub](https://github.com/NVIDIA/nvidia-container-toolkit)
- [How to Set Up NVIDIA GPU Support in Docker for AI/ML](https://oneuptime.com/blog/post/2026-01-16-docker-nvidia-gpu-ai-ml/view)
- [RunPod — Deploy Docker Containers on GPU Cloud](https://www.runpod.io/articles/guides/deploy-fastapi-applications-gpu-cloud)
- [vast.ai — Instances Overview](https://docs.vast.ai/documentation/instances/overview)
- [vast.ai — GPU Pricing](https://vast.ai/pricing)
- [vast.ai Review 2026](https://www.gpunex.com/blog/vast-ai-review-2026/)
- [vast.ai vs RunPod Pricing 2026](https://medium.com/@velinxs/vast-ai-vs-runpod-pricing-in-2026-which-gpu-cloud-is-cheaper-bd4104aa591b)
- [7 Cheapest Cloud GPU Providers in 2026](https://northflank.com/blog/cheapest-cloud-gpu-providers)
- [Use Ansible to Install Docker on Ubuntu — DigitalOcean](https://www.digitalocean.com/community/tutorials/how-to-use-ansible-to-install-and-set-up-docker-on-ubuntu-20-04)
- [Docker Security Tips & Ansible Playbook](https://www.virtualizationhowto.com/2025/07/docker-security-tips-for-container-hosts-ansible-playbook/)
- [Ansible Tutorial: Automate Servers in 7 Steps (2026)](https://tech-insider.org/ansible-tutorial-automate-infrastructure-2026/)
