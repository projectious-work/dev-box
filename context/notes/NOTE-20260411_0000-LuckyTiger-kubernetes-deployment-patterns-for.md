---
id: NOTE-20260411_0000-LuckyTiger-kubernetes-deployment-patterns-for
title: "Kubernetes Deployment Patterns for aibox"
type: reference
status: permanent
created: 2026-04-11T00:00:00Z
tags: [kubernetes, helm, devpod, k8s]
skill: research-with-confidence
source_file: kubernetes-deployment-2026-03.md
---

# Kubernetes Deployment Patterns for aibox — Research Report — March 2026

Research for BACK-068. Evaluates how aibox should scaffold Helm charts for
Kubernetes deployment, covering compose-to-K8s translation, chart structure,
dev-environment-on-K8s solutions, secret management, and integration design.
Conducted 2026-03-26.

---

## 1. Docker Compose to Kubernetes Translation

aibox generates `.devcontainer/docker-compose.yml` from `aibox.toml`. The
question is whether to translate that compose file into K8s manifests
automatically, or scaffold Helm charts independently.

### 1.1 Kompose

Kompose (`kompose convert`) translates Docker Compose files into Kubernetes
Deployments, Services, and PVCs.

| Aspect | Assessment |
|---|---|
| Maturity | CNCF sandbox project, stable, widely used |
| Compose v2 support | Full support for `services`, `volumes`, `ports`, `environment` |
| Output formats | K8s YAML manifests, Helm charts (`--chart`), OpenShift |
| Handles `build:` | No — Kompose assumes pre-built images; `build:` context is ignored |
| Volume translation | Named volumes become PVCs; bind mounts (`.aibox-home/...`) require manual mapping |
| `command: sleep infinity` | Translated literally — works but is a K8s anti-pattern |
| Networking | Each service becomes a Service + Deployment; compose `networks:` map to Services |
| Environment variables | Translated to env in Pod spec or ConfigMaps |

**Limitations for aibox:**
- aibox compose uses `build: context: .` with a local Dockerfile — Kompose cannot
  handle this. The image must be pre-built and pushed to a registry (GHCR).
- Bind mounts like `{{ host_root }}/.ssh:{{ container_home }}/.ssh:ro` have no K8s
  equivalent. These must become Secrets (SSH keys), ConfigMaps (vim/zellij config),
  or PVCs (persistent home directories).
- The `stdin_open: true` + `tty: true` + `sleep infinity` pattern is how aibox keeps
  containers alive for `docker exec` attachment. In K8s, the equivalent is a long-running
  Pod with `kubectl exec`, but proper dev environments use a dedicated solution (see Section 3).
- Kompose's `--chart` flag produces a basic Helm chart but with flat values and no
  templating sophistication.

**Verdict:** Kompose is useful as a reference for one-off migration, but not suitable
as an ongoing generation target. aibox should scaffold its own Helm chart with
purpose-built templates.

### 1.2 Docker Compose Kubernetes Mode

Docker Desktop shipped experimental `docker compose up --kube` support using the
built-in Kubernetes cluster. This feature was removed in Docker Desktop 4.x and
is no longer maintained. Not a viable path.

### 1.3 Manual Translation Patterns

The compose template maps to K8s objects as follows:

| Compose concept | K8s object | Notes |
|---|---|---|
| `services.<name>` | Deployment + Service | One replica by default |
| `build:` | None (pre-build required) | Must push image to registry first |
| `container_name` | Pod metadata.name / Deployment name | |
| `hostname` | `spec.hostname` on Pod | |
| `ports` | Service (ClusterIP/NodePort/LoadBalancer) + Ingress | |
| `volumes` (bind mount, config) | ConfigMap volume mount | For vim, zellij, yazi, git config |
| `volumes` (bind mount, secrets) | Secret volume mount | For SSH keys, API keys |
| `volumes` (workspace) | PVC | Workspace persistence |
| `environment` | `env` in container spec, or ConfigMap/Secret ref | |
| `command: sleep infinity` | Drop — use proper entrypoint or dev tool | |
| `stdin_open` + `tty` | Not needed in K8s dev patterns | |

---

## 2. Helm Chart Scaffolding

### 2.1 Recommended K8s Objects

For an aibox-generated Helm chart, the following objects should be scaffolded:

| Object | Purpose | Always? | Configurable via values? |
|---|---|---|---|
| **Deployment** | Runs the dev container (1 replica) | Yes | replicas, resources, nodeSelector, tolerations |
| **Service** | Exposes ports within cluster | Yes | type (ClusterIP/NodePort/LB), ports list |
| **ConfigMap** | Non-sensitive env vars + config files | Yes | environment map from aibox.toml |
| **Secret** | SSH keys, API keys, provider tokens | Yes | externalSecret reference or inline |
| **PersistentVolumeClaim** | Workspace storage + home directory | Yes | storageClass, size |
| **Ingress** | HTTP access (web-based IDE, app ports) | Optional | host, TLS, annotations |
| **ServiceAccount** | RBAC for the pod | Optional | create toggle, annotations (IRSA/Workload Identity) |
| **HorizontalPodAutoscaler** | Auto-scaling (app deployments, not dev) | Optional | min/max replicas, CPU/memory targets |

**MVP subset:** Deployment, Service, ConfigMap, Secret, PVC. Add Ingress and
ServiceAccount in a follow-up.

### 2.2 values.yaml Structure

Best practice: mirror the aibox.toml mental model so users recognize the mapping.

```yaml
# values.yaml — generated from aibox.toml by `aibox deploy k8s`

# Image configuration
image:
  registry: ghcr.io/projectious-work/aibox
  repository: python            # from aibox.toml [aibox].image (now addons)
  tag: "0.13.2"                 # from aibox.toml [aibox].version
  pullPolicy: IfNotPresent

# Container configuration (maps to [container])
container:
  name: my-project
  hostname: aibox
  user: aibox
  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: "2"
      memory: 2Gi

# Networking (maps to [container].ports)
service:
  type: ClusterIP
  ports: []
  # - name: http
  #   port: 8080
  #   targetPort: 8080

ingress:
  enabled: false
  className: ""
  annotations: {}
  hosts: []
  tls: []

# Storage
persistence:
  workspace:
    enabled: true
    storageClass: ""            # empty = default StorageClass
    size: 10Gi
    accessModes:
      - ReadWriteOnce
  home:
    enabled: true
    storageClass: ""
    size: 1Gi

# Environment variables (maps to [container].environment)
environment: {}
  # GIT_AUTHOR_NAME: "Dev User"
  # GIT_AUTHOR_EMAIL: "dev@example.com"

# Secrets — reference external secrets or provide inline
secrets:
  # Option 1: inline (not recommended for production)
  ssh:
    enabled: false
    privateKey: ""
  # Option 2: reference existing K8s secret
  existingSecret: ""
  # Option 3: External Secrets Operator
  externalSecret:
    enabled: false
    secretStoreRef:
      name: ""
      kind: ClusterSecretStore

# AI provider configuration (maps to [ai])
ai:
  providers: []
  # - name: claude
  #   apiKeySecret: claude-api-key    # name of K8s Secret

# Init container (maps to [container].post_create_command)
initContainer:
  enabled: false
  command: ""

# Pod scheduling
nodeSelector: {}
tolerations: []
affinity: {}

# Service account
serviceAccount:
  create: true
  annotations: {}
  name: ""

# HPA (for application deployments, not dev environments)
autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 3
  targetCPUUtilizationPercentage: 80
```

### 2.3 Multi-Container Pods (Sidecar Pattern)

aibox environments may need companion containers:

| Sidecar use case | Pattern |
|---|---|
| AI provider daemon (if needed) | Sidecar container in same Pod |
| Database for development | Separate Deployment + Service (not sidecar) |
| Log collector / metrics | K8s-native sidecar (KEP-753, GA in K8s 1.29+) |
| SSH server for remote attach | Sidecar with `sshd` + shared volume |
| File sync (e.g., mutagen) | Sidecar with shared PVC |

Helm template pattern for sidecars:

```yaml
# In deployment.yaml template
spec:
  containers:
    - name: {{ .Values.container.name }}
      image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
      # ... main container spec
    {{- range .Values.sidecars }}
    - name: {{ .name }}
      image: {{ .image }}
      {{- with .ports }}
      ports: {{ toYaml . | nindent 8 }}
      {{- end }}
    {{- end }}
```

### 2.4 Init Containers for Setup

The `post_create_command` in aibox.toml runs setup after container creation. In
K8s, this maps to an init container:

```yaml
initContainers:
  {{- if .Values.initContainer.enabled }}
  - name: setup
    image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
    command: ["/bin/bash", "-c"]
    args:
      - {{ .Values.initContainer.command | quote }}
    volumeMounts:
      - name: workspace
        mountPath: /workspace
      - name: home
        mountPath: {{ include "aibox.homePath" . }}
  {{- end }}
```

Alternatively, for lightweight setup, a `postStart` lifecycle hook on the main
container can replace the init container pattern. However, init containers are
preferred because they block Pod readiness until complete, and failures are
clearly visible.

---

## 3. Dev Environment on Kubernetes — Existing Solutions

### 3.1 Comparison Matrix

| Feature | DevPod | Coder | Eclipse Che | Telepresence |
|---|---|---|---|---|
| **Architecture** | Client-side CLI, pluggable providers | Server (control plane) + agents | Operator on K8s (DevWorkspace CRD) | Client daemon + traffic manager |
| **K8s provider** | Yes (built-in) | Yes (native) | Yes (only K8s) | Bridges local to K8s |
| **DevContainer spec** | Full support | Partial (envbuilder) | Own CRD (DevWorkspace) | N/A — uses existing services |
| **Persistent storage** | PVC per workspace | PVC per workspace | PVC per workspace | Local filesystem |
| **SSH access** | Built-in (`devpod ssh`) | Built-in (coder ssh) | Via route/port-forward | N/A |
| **Port forwarding** | Automatic (openvpn/wireguard) | Built-in | Built-in | Full cluster DNS + intercepts |
| **GPU support** | Via K8s device plugin + tolerations | Native (Coder templates) | Via DevWorkspace spec | N/A |
| **Multi-IDE** | VS Code, JetBrains, terminal | VS Code, JetBrains, terminal, web | VS Code (Che-Theia), JetBrains | IDE-agnostic (local tools) |
| **Self-hosted** | Client-only (no server) | Yes (helm chart, 1 pod) | Yes (operator) | Yes (traffic manager) |
| **License** | Apache 2.0 | AGPL (OSS) / Enterprise | EPL 2.0 | Proprietary (Ambassador Labs) |
| **Maturity** | Production-ready, active | Production-ready, mature | Mature but complex | Mature, widely used |

### 3.2 DevPod (Loft Labs)

Most relevant to aibox because it already supports the devcontainer spec.

- `devpod up . --provider kubernetes` spins up a Pod from `.devcontainer/`.
- Uses the devcontainer spec natively — aibox already generates this.
- Creates a PVC for `/workspace`, mounts it into the Pod.
- SSH access via `devpod ssh <workspace>` using a built-in SSH server.
- Port forwarding through an encrypted tunnel (WireGuard or OpenVPN).
- GPU: user configures `nodeSelector` and `tolerations` in the provider options.
- **aibox integration opportunity:** aibox could be a DevPod provider or simply
  ensure its devcontainer.json is DevPod-compatible (it already is).

### 3.3 Coder

Full remote development platform with K8s as a first-class backend.

- Coder templates (Terraform-based) define workspace infrastructure.
- `coder create` provisions a K8s Deployment + PVC + Service.
- Rich agent that handles SSH, port-forward, dotfile sync, IDE connections.
- Supports workspace prebuilds (snapshot PVCs for fast startup).
- GPU: Coder templates explicitly request `nvidia.com/gpu` resources.
- **aibox integration opportunity:** Generate a Coder template from aibox.toml.
  This is essentially a Terraform file that creates K8s resources — heavier than
  a Helm chart but more powerful.

### 3.4 Eclipse Che / DevWorkspace Operator

- Kubernetes-native: installs as an operator, defines workspaces via CRDs.
- DevWorkspace CRD is a K8s-native workspace spec (distinct from devcontainer).
- Heavy infrastructure: requires Che server, dashboard, devfile registry, plugin registry.
- Conversion: Che can consume devcontainer.json (via devfile v2 adapter), but
  the mapping is lossy and the UX is Eclipse-centric.
- **aibox integration opportunity:** Low. Che's architecture is opinionated and
  heavy. Not a good match for aibox's lightweight philosophy.

### 3.5 Telepresence

- Does not create remote dev environments — instead, bridges a local dev
  environment to a remote K8s cluster.
- `telepresence connect` gives local processes cluster DNS and service access.
- `telepresence intercept <service>` routes cluster traffic to local machine.
- Best for "develop locally, test against remote services."
- **aibox integration opportunity:** Complementary. An aibox container running
  locally could use Telepresence to access remote K8s services. Not relevant
  for the "deploy dev env to K8s" use case.

### 3.6 How They Handle Key Concerns

| Concern | Best approach |
|---|---|
| **Persistent storage** | PVC (all solutions use this). StorageClass matters — use `ReadWriteOnce` for single-user dev, `ReadWriteMany` (NFS/EFS) for shared workspaces. |
| **SSH access** | Embedded SSH server in the Pod (DevPod, Coder) or `kubectl exec` with a wrapper. DevPod's approach is cleanest. |
| **Port forwarding** | `kubectl port-forward` is basic but works. DevPod/Coder add encrypted tunnels. For web UIs, Ingress with auth is better. |
| **GPU** | K8s device plugin (`nvidia/k8s-device-plugin`). Request via `resources.limits: nvidia.com/gpu: 1`. Requires node with GPU + driver. Node selector/tolerations for GPU node pools. |

---

## 4. Secret Management on Kubernetes

### 4.1 Comparison Matrix

| Solution | Complexity | External dependency | Encryption at rest | Rotation | Best for |
|---|---|---|---|---|---|
| **K8s Secrets (plain)** | Trivial | None | Only if etcd encryption enabled | Manual | Testing, non-sensitive config |
| **Sealed Secrets** | Low | Sealed Secrets controller | Yes (asymmetric crypto) | Manual re-seal | Git-committed secrets |
| **External Secrets Operator** | Medium | ESO controller + external store (Vault, AWS SM, GCP SM) | Yes (external store) | Automatic | Production, multi-cloud |
| **SOPS + age/GPG** | Low | None (client-side) | Yes (age/GPG encryption) | Manual re-encrypt | Small teams, git-ops |
| **HashiCorp Vault (direct)** | High | Vault server + injector/CSI | Yes | Automatic | Enterprise, complex policies |

### 4.2 Assessment for Dev Environments

**Sealed Secrets** — Developer creates a SealedSecret manifest, commits to git.
The controller in the cluster decrypts it into a regular Secret. Simple workflow,
but requires the controller to be installed on the cluster. No external
dependencies beyond K8s.

**SOPS + age** — Secrets are encrypted in YAML files using `age` keys. Decrypted
at deploy time (e.g., in a CI pipeline or via ArgoCD SOPS plugin). No K8s
controller needed. Works with `helm secrets` plugin for Helm-native integration.

**External Secrets Operator (ESO)** — Best for organizations already using AWS
Secrets Manager, GCP Secret Manager, or HashiCorp Vault. ESO syncs external
secrets into K8s Secrets. More infrastructure but handles rotation automatically.

**Recommendation for aibox:** SOPS + age is the simplest option that provides
real encryption without requiring cluster-side infrastructure. It works with
`helm secrets` for transparent Helm integration. For organizations with existing
secret stores, ESO should be documented as the recommended upgrade path.

### 4.3 What Secrets Does an aibox Dev Environment Need?

| Secret | Source | K8s object |
|---|---|---|
| SSH private key | `~/.ssh/id_ed25519` | Secret (mounted as file) |
| Git credentials | `~/.config/git/credentials` | Secret (mounted as file) |
| Claude API key | `ANTHROPIC_API_KEY` | Secret (env var ref) |
| Other AI provider keys | `OPENAI_API_KEY`, etc. | Secret (env var ref) |
| Custom env vars with secrets | `[container].environment` | Secret (env var ref) |

---

## 5. aibox Integration Design

### 5.1 Option Comparison

| Aspect | Option A: `aibox deploy k8s` | Option B: `aibox init --deploy k8s` | Option C: `deploy/` + `aibox sync` |
|---|---|---|---|
| **When it runs** | On-demand command | At project init time | Scaffolded at init, updated on sync |
| **Chart location** | Generated to `deploy/helm/` | Generated to `deploy/helm/` | Lives in `deploy/helm/` |
| **Regeneration** | Each `aibox deploy k8s` regenerates | Manual re-run of init (awkward) | `aibox sync` keeps chart in sync |
| **Compose relationship** | Independent generation from aibox.toml | Parallel to compose generation | Parallel to compose generation |
| **User mental model** | "I deploy when I'm ready" | "I choose deploy target at init" | "Chart stays in sync like compose" |
| **Implementation complexity** | Medium (new command) | Low (flag on existing command) | Medium (sync integration) |

### 5.2 Recommended Approach: Hybrid A+C

**Phase 1 (MVP):** `aibox deploy k8s` generates a Helm chart from aibox.toml
into `deploy/helm/<container-name>/`. This is a one-shot generation that users
can then customize. The chart is committed to git.

**Phase 2:** `aibox sync` gains awareness of `deploy/helm/` and updates
values.yaml when aibox.toml changes (image version, ports, env vars). The
templates themselves are not regenerated unless `aibox deploy k8s --force` is run.

**Phase 3:** `aibox deploy k8s --apply` runs `helm upgrade --install` against a
configured cluster.

### 5.3 Chart Location

**Recommended: `deploy/helm/<chart-name>/`**

Rationale:
- `.devcontainer/` is for local development — mixing in K8s manifests confuses the
  purpose boundary.
- `deploy/` is a widely recognized convention for deployment manifests (Helm, Kustomize,
  Terraform). It signals "this is how you deploy to infrastructure."
- `<chart-name>` subdirectory allows multiple charts (e.g., dev environment chart +
  application chart).

Directory structure after `aibox deploy k8s`:

```
deploy/
  helm/
    my-project/
      Chart.yaml
      values.yaml
      templates/
        _helpers.tpl
        deployment.yaml
        service.yaml
        configmap.yaml
        secret.yaml
        pvc.yaml
        ingress.yaml          (disabled by default)
        serviceaccount.yaml   (optional)
        hpa.yaml              (optional)
      .helmignore
```

### 5.4 Image Registry Handling

The compose workflow builds images locally. K8s requires images in a registry.

**Workflow:**

1. `aibox deploy k8s` adds registry info to `values.yaml`:
   ```yaml
   image:
     registry: ghcr.io/<org>
     repository: <project-name>
     tag: latest
   ```

2. User pushes image:
   ```bash
   docker build -t ghcr.io/<org>/<project>:latest .devcontainer/
   docker push ghcr.io/<org>/<project>:latest
   ```

3. Future: `aibox deploy k8s --build --push` automates build + push.

**aibox.toml extension** (future):

```toml
[deploy.kubernetes]
registry = "ghcr.io/my-org"
namespace = "dev"
context = "my-cluster"
```

### 5.5 Translation from aibox.toml to Helm values

| aibox.toml field | Helm values.yaml field |
|---|---|
| `[aibox].version` | `image.tag` |
| `[container].name` | `container.name`, chart name |
| `[container].hostname` | `container.hostname` |
| `[container].user` | `container.user` (securityContext.runAsUser) |
| `[container].post_create_command` | `initContainer.command` |
| `[container].environment` | `environment` map |
| `[ai].providers` | `ai.providers` list |
| `[addons]` | Determines base image repository |

---

## 6. Recommendation

### MVP Definition

The most practical first step, given aibox's compose-based architecture:

**Implement `aibox deploy k8s` as a chart generator.** It reads `aibox.toml`,
generates a Helm chart to `deploy/helm/<name>/`, and prints instructions for
deploying.

**MVP scope:**

1. **Generate 5 K8s objects:** Deployment, Service, ConfigMap, Secret (template
   for SSH + API keys), PVC (workspace + home).
2. **values.yaml** derived from aibox.toml with sensible defaults for resources,
   storage class, and service type.
3. **No auto-deploy.** Users run `helm install` themselves. This avoids
   kubeconfig management complexity in the MVP.
4. **No DevPod integration yet.** Document that the generated devcontainer.json
   already works with `devpod up --provider kubernetes` for users who want that
   path today.
5. **Secret management:** Generate Secret templates with placeholder values and
   a commented example for SOPS + age. No External Secrets in MVP.

**Implementation estimate:** ~2-3 days of Rust development.

- New module: `cli/src/deploy.rs` (or `cli/src/helm.rs`)
- New Jinja2 templates: `cli/src/templates/helm/` (Chart.yaml.j2, values.yaml.j2,
  deployment.yaml.j2, service.yaml.j2, configmap.yaml.j2, secret.yaml.j2, pvc.yaml.j2)
- New CLI subcommand: `aibox deploy k8s` in `cli/src/cli.rs`
- Integration with existing `config.rs` for reading aibox.toml

### Post-MVP Roadmap

| Phase | Feature | Effort |
|---|---|---|
| MVP | `aibox deploy k8s` generates Helm chart | 2-3 days |
| Phase 2 | `aibox sync` updates values.yaml on config changes | 1 day |
| Phase 3 | Ingress + ServiceAccount + HPA templates | 1 day |
| Phase 4 | `aibox deploy k8s --apply` runs helm install/upgrade | 1-2 days |
| Phase 5 | `aibox deploy k8s --build --push` builds + pushes image | 1 day |
| Phase 6 | DevPod provider integration (aibox as DevPod provider) | 3-5 days |
| Phase 7 | Coder template generation | 2-3 days |
| Phase 8 | External Secrets Operator support in chart | 1 day |
| Phase 9 | GPU resource configuration in aibox.toml | 1 day |

### Key Design Decisions for BACK-068

| Decision | Recommendation | Rationale |
|---|---|---|
| Kompose vs custom scaffold | Custom scaffold | Kompose cannot handle aibox's build-context pattern and produces unrefined output |
| Chart location | `deploy/helm/<name>/` | Clear separation from local dev (.devcontainer/) |
| Secret management (MVP) | Plain Secret templates + SOPS docs | Lowest barrier; ESO as documented upgrade |
| Dev-on-K8s integration | Document DevPod compatibility | aibox devcontainer.json already works with DevPod; no code needed |
| Image registry | User-configured in values.yaml | No auto-push in MVP; add in Phase 5 |
| Init vs `aibox deploy` command | New `deploy` subcommand | Clean separation of concerns; `init` is already overloaded |
| Regeneration strategy | One-shot generate, user owns chart | Avoids losing user customizations; Phase 2 adds selective sync |

---

## Appendix A: Helm Chart Template Sketch

### Chart.yaml

```yaml
apiVersion: v2
name: {{ name }}
description: Kubernetes deployment for {{ name }} aibox environment
type: application
version: 0.1.0
appVersion: "{{ aibox_version }}"
```

### deployment.yaml (simplified)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "chart.fullname" . }}
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ include "chart.name" . }}
  template:
    metadata:
      labels:
        app.kubernetes.io/name: {{ include "chart.name" . }}
    spec:
      serviceAccountName: {{ include "chart.serviceAccountName" . }}
      {{- if .Values.initContainer.enabled }}
      initContainers:
        - name: setup
          image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          command: ["/bin/bash", "-c", {{ .Values.initContainer.command | quote }}]
          volumeMounts:
            - name: workspace
              mountPath: /workspace
      {{- end }}
      containers:
        - name: {{ .Values.container.name }}
          image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          {{- with .Values.container.resources }}
          resources: {{ toYaml . | nindent 12 }}
          {{- end }}
          env:
            {{- range $key, $val := .Values.environment }}
            - name: {{ $key }}
              value: {{ $val | quote }}
            {{- end }}
            {{- range .Values.ai.providers }}
            - name: {{ .envVar }}
              valueFrom:
                secretKeyRef:
                  name: {{ .apiKeySecret }}
                  key: api-key
            {{- end }}
          volumeMounts:
            - name: workspace
              mountPath: /workspace
            - name: home
              mountPath: /home/{{ .Values.container.user }}
            {{- if .Values.secrets.ssh.enabled }}
            - name: ssh-keys
              mountPath: /home/{{ .Values.container.user }}/.ssh
              readOnly: true
            {{- end }}
      volumes:
        - name: workspace
          persistentVolumeClaim:
            claimName: {{ include "chart.fullname" . }}-workspace
        - name: home
          persistentVolumeClaim:
            claimName: {{ include "chart.fullname" . }}-home
        {{- if .Values.secrets.ssh.enabled }}
        - name: ssh-keys
          secret:
            secretName: {{ include "chart.fullname" . }}-ssh
            defaultMode: 0400
        {{- end }}
      {{- with .Values.nodeSelector }}
      nodeSelector: {{ toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations: {{ toYaml . | nindent 8 }}
      {{- end }}
```

---

## Appendix B: DevPod Quick-Start (No Code Required)

Users who want remote K8s development today, without waiting for `aibox deploy k8s`:

```bash
# Install DevPod
brew install devpod    # or: curl -fsSL https://devpod.sh | bash

# Add Kubernetes provider
devpod provider add kubernetes

# Start workspace from aibox project (uses .devcontainer/)
devpod up . --provider kubernetes --ide vscode

# SSH into the workspace
devpod ssh my-project
```

This works because aibox already generates a compliant `devcontainer.json`. The
only prerequisite is a kubeconfig pointing to a cluster with sufficient resources.
DevPod handles PVC creation, image building (via kaniko), and SSH tunneling.

This path should be documented in aibox docs immediately as a zero-code K8s
deployment option, independent of the Helm chart work in BACK-068.
