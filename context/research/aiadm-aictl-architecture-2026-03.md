---
id: research-aiadm-aictl
title: "aiadm/aictl Architecture — Kubernetes-Inspired CLI Split for Deterministic RBAC"
type: research
status: draft
date: 2026-03-28
related: [DISC-001]
---

Research into the proposed aiadm/aictl architectural split: renaming aibox to aiadm
(infrastructure) and introducing aictl (context operations) with certificate-based
authentication, OS-level file lockdown, and deterministic audit logging.

## Overview

- Four research areas: OS-level lockdown, kubectl-to-aictl command mapping, impact on
  50 DISC-001 decisions, Kubernetes certificate/RBAC mechanics
- The proposal is architecturally sound but OS-level enforcement has fundamental limits
- 14 decisions unchanged, 17 modified, 7 superseded, 12 strengthened
- Recommended approach: layered enforcement with cryptographic signing

## Details

### 1. OS-Level Lockdown Mechanisms

The goal: make `context/` writable only by the `aictl` process, preventing AI agents
with shell access from bypassing RBAC by editing files directly.

#### Mechanisms Assessed

**Unix File Permissions (DAC)**
- `context/` owned by `aibox-system:aibox-system`, mode `750`; aictl runs setgid
- Feasibility: Low effort
- Security: NONE against agent with sudo/root (trivially `chmod 777`)
- Git: Works if git runs as same group; agent can also use git
- Verdict: Speed bump only, prevents accidental writes

**SELinux / AppArmor (Mandatory Access Control)**
- AppArmor profile: `deny /workspace/context/** w` for all processes except aictl
- Host-applied profiles cannot be overridden from inside container (strongest option)
- Feasibility: Medium-high (policy authoring is non-trivial)
- Security: HIGH when host-applied (agent cannot `aa-teardown` without `CAP_MAC_ADMIN`)
- Git: Problematic — if git gets write exception, agent can use git to write
- Caveat: Requires control of Docker host; managed environments (Codespaces) may not allow

**Linux Capabilities**
- Drop `CAP_DAC_OVERRIDE` and `CAP_FOWNER` from container
- Makes DAC meaningful (root can't bypass file permissions)
- Feasibility: Easy (`docker run --cap-drop=...`)
- Security: Low-medium (broad side effects, breaks many standard tools)
- Verdict: Hardening layer, not standalone solution

**Container-Level Isolation — Read-Only Mount + Sidecar**
- Mount `context/` as `:ro` in dev container; sidecar has `:rw`
- Agent literally cannot write (VFS rejects at kernel level)
- Security: HIGH (requires `CAP_SYS_ADMIN` to remount, Docker drops by default)
- Git: BROKEN — git cannot write to context/ at all (showstopper)
- Verdict: Strongest mechanism but fundamentally breaks git workflow

**FUSE Filesystem**
- Custom FUSE daemon intercepts writes, checks calling process via `/proc/PID/exe`
- Can verify full process tree (git invoked by aictl vs git invoked by shell)
- Feasibility: HIGH implementation effort (Rust `fuser` crate)
- Security: Medium (agent can kill daemon = DoS but not bypass; mitigated by dropping caps)
- Git: GOOD — best mechanism for selective git access
- Verdict: Most flexible, best git story, but significant engineering

#### The Fundamental Problem

**If git can write to `context/`, and the agent can run git, then the agent can write
to `context/` via git.** This is true for every mechanism except read-only mount.

The FUSE process-tree verification approach is the only one that can distinguish
"git invoked by aictl" from "git invoked by agent shell."

#### Cross-Platform Equivalents

| OS | Best mechanism | Strength vs admin-level agent |
|----|---------------|-------------------------------|
| Linux (container) | Host-applied AppArmor + FUSE | Medium-High |
| Linux (bare metal) | SELinux/AppArmor | Medium (root can disable) |
| Windows | NTFS ACLs + Integrity Levels | Medium |
| macOS | sandbox-exec profiles | Medium |

#### Honest Assessment

**If an AI agent has unrestricted root shell access inside a container, no intra-container
mechanism provides absolute protection.** The host-applied AppArmor profile is the only
genuinely robust mechanism (enforcement in host kernel, container cannot modify).

#### Recommended Layered Approach

| Layer | Mechanism | Purpose |
|-------|-----------|---------|
| 1 (Speed bump) | Unix DAC | Prevent accidental writes |
| 2 (Kernel) | Host AppArmor OR FUSE | Mechanical enforcement |
| 3 (Git mediation) | FUSE process-tree checks | Allow git only via aictl |
| 4 (Detection) | Cryptographic signatures | Detect tampering post-facto |

**Practical recommendation:** Start with Layer 1 (DAC) + Layer 4 (signing). This gives
honest protection against accidental writes plus cryptographic detection of tampering,
with no git compatibility headaches. Evolve toward FUSE when implementation budget allows.

The security model should be: **"make it hard to tamper, and always detect when tampering
occurs"** rather than **"make it impossible to tamper."**

---

### 2. kubectl to aictl Command Mapping

#### Conceptual Translation

| Kubernetes | aibox |
|-----------|-------|
| Resource kind | Primitive kind (17 kinds) |
| Namespace | Scope (SCOPE-xxx) |
| Pod | Work Item (atomic unit) |
| Label | Tag / label key-value |
| Annotation | Custom field |
| kubeconfig | `~/.aibox/identity.toml` + certificate |
| etcd | `context/` filesystem |
| API server | aictl binary |

#### Core CRUD Commands (1:1 mapping)

| kubectl | aictl | Under the hood |
|---------|-------|----------------|
| `create <kind> <name>` | `create <kind> [name] [--field=val]` | Auth, RBAC check, generate petname ID, create .md file, update SQLite, log event |
| `get <kind> [name]` | `get <kind> [id] [--state=X]` | Query SQLite index with filters |
| `describe <name>` | `describe <id>` | Full file + cross-refs + events + state machine info |
| `delete <name>` | `delete <id> [--force]` | Soft-delete (archive) by default; checks cross-refs |
| `edit <name>` | `edit <id>` | Opens .md file in $EDITOR, validates on save |
| `apply -f <file>` | `apply -f <file>` | Declarative create-or-update; file-as-truth |
| `patch <name> -p '...'` | `patch <id> --field=val` | Partial field update; supports `+=` / `-=` for arrays |
| `replace -f <file>` | `replace -f <file>` | Full file overwrite (entity must exist) |

#### Inspection Commands

| kubectl | aictl | Notes |
|---------|-------|-------|
| `logs <pod>` | `logs <id> [--since=24h]` | JSONL event log entries for entity (not stdout) |
| `events [--for=X]` | `events [--type=X --since=Y]` | Dual-source events (agent + infrastructure) |
| `top pods` | `metrics [--kind=X]` | Process health: velocity, cycle time, WIP counts |

#### Auth and RBAC Commands

| kubectl | aictl | Notes |
|---------|-------|-------|
| `auth whoami` | `auth whoami` | Identity + Actor + Roles + Permissions + cert validity |
| `auth can-i <verb> <kind>` | `auth can-i <verb> <kind>` | Checks RBAC; supports `--as=@handle` |
| `config view/use-context` | `config view/use-context` | Manages identity + project registry |
| `create role` | `create role <name>` | Permissions now structured (not only plain English) |
| `create rolebinding` | `bind <role> --actor=<id>` | Bidirectional binding |

#### Resource Information

| kubectl | aictl | Notes |
|---------|-------|-------|
| `api-resources` | `api-resources` | Lists 17 primitive kinds with verbs and shortnames |
| `api-versions` | `api-versions` | `aibox/v1` |
| `explain <kind>` | `explain <kind> [field.path]` | Schema documentation with state machine info |
| `cluster-info` | `info` | Project path, versions, index stats, CA validity |

#### New aictl Commands (no kubectl equivalent)

| Command | Purpose |
|---------|---------|
| `aictl transition <id> --to <state>` | First-class state machine transition with guard logging |
| `aictl lint [--fix]` | Post-facto validation (schema, state machine, cross-refs, RBAC) |
| `aictl sync` | Rebuild SQLite index from filesystem |
| `aictl search <query> [--semantic]` | Full-text + vector search across entities |
| `aictl comment <id> "text"` | Add comment to entity |
| `aictl link/unlink <a> --to <b>` | Manage typed cross-references |
| `aictl tree [<id>]` | Hierarchy visualization |
| `aictl board [--scope=X]` | Kanban terminal view |
| `aictl id generate` | Petname ID generation |
| `aictl import/export` | External system integration |
| `aictl graph [--format=dot]` | Dependency graph generation |
| `aictl archive/unarchive` | Cold storage management |
| `aictl history <id>` | Git log for entity file |
| `aictl rollback <id>` | Revert entity to previous git version |

#### kubectl Commands with NO aictl Equivalent (moved to aiadm)

`exec`, `cp`, `port-forward`, `attach`, `proxy`, `scale`, `autoscale`, `cordon/drain`,
`taint`, `certificate`, `token`, `run`, `expose`, `kustomize` — all container/node/
infrastructure operations that belong in aiadm, not aictl.

#### aiadm Commands

| aiadm | kubeadm equivalent | Purpose |
|-------|-------------------|---------|
| `aiadm init` | `kubeadm init` | Scaffold project, create CA |
| `aiadm reset` | `kubeadm reset` | Tear down context structure |
| `aiadm upgrade` | `kubeadm upgrade` | Schema version migration |
| `aiadm cert generate/approve/list/revoke` | `kubeadm certs` | Certificate lifecycle |
| `aiadm image build/push` | (docker) | Container image management |
| `aiadm container start/stop` | N/A | Dev container lifecycle |
| `aiadm skill/process install` | N/A | Extension management |
| `aiadm doctor` | N/A | Infrastructure health check |

#### Summary Statistics

- kubectl commands with direct aictl equivalents: **24**
- kubectl commands adapted semantically: **4** (set->transition, rollout->history, top->metrics, cluster-info->info)
- kubectl commands with no aictl equivalent: **16** (container/node ops -> aiadm)
- New aictl commands with no kubectl equivalent: **12**
- **Total aictl commands: ~40**

---

### 3. Impact on DISC-001 Decisions

#### Classification Summary

| Classification | Count | Decisions |
|---------------|-------|-----------|
| UNCHANGED | 14 | 1, 2, 5, 10, 11, 14, 15, 17, 25, 28, 31, 32, 34, 36, 37, 39, 44, 45 |
| MODIFIED | 17 | 3, 8, 12, 13, 16, 18, 22, 23, 24, 26, 27, 30, 33, 38, 40, 41, 46, 49, 50 |
| SUPERSEDED | 7 | 9, 19, 20, 21, 35, 43, 47, 48 |
| STRENGTHENED | 12 | 4, 6, 7, 29, 42 |

Note: Some decisions map to multiple classifications depending on sub-aspect; totals
reflect primary classification.

#### Critical Superseded Decisions

**Decision 9 (State machine guards — agents edit state directly):**
Agents can no longer edit state directly. `aictl transition BACK-xxx --to state` replaces
direct file editing. However, guards remain plain English — aictl trusts the agent's
assertion that guards are satisfied. Hybrid: probabilistic evaluation, deterministic
execution and logging.

**Decision 19 (RBAC via plain English, probabilistic):**
RBAC becomes mechanical. Certificate identifies actor -> actor's roles -> permissions
checked BEFORE allowing the operation. Plain English descriptions become documentation
of the mechanical rules, not the rules themselves. Permissions can also be structured:

```yaml
spec:
  permissions:
    - action: [create, edit, delete]
      kinds: [WorkItem, Decision]
  restrictions:
    - action: [edit, delete]
      kinds: [Process, StateMachine, Role]
```

**Decision 20 (Dual event sources):**
Collapses into: (1) aictl command log — deterministic, captures WHAT; (2) agent reasoning
log — probabilistic, captures WHY; (3) aiadm infrastructure — deterministic. The critical
"what changed" log is now 100% deterministic.

**Decision 21 (No hook execution infrastructure):**
aictl IS execution infrastructure. Agents remain the judgment layer but aictl handles
mechanical execution (file writes, transitions, RBAC, logging, validation).

**Decision 43, 47, 48 (Identity and RBAC flow):**
identity.toml replaced by certificate-based identity. RBAC flow becomes fully mechanical:
cert -> verify against CA -> extract identity -> match Actor -> load Roles -> check
permissions -> allow/deny.

#### Key Insight: The Boundary Shift (Decision 16)

The infrastructure/application boundary redraws:

| Layer | Current model | New model |
|-------|--------------|-----------|
| Infrastructure | aibox (init, sync, lint, migrate) | aiadm (init, images, containers, schema, certs) |
| Tooling | — | aictl (CRUD, transitions, queries, validation) |
| Application | Agents (everything else) | Agents (judgment: what to create, when to transition) |

Agents still decide WHAT to do. aictl handles HOW it gets written. This is stronger:
the current model asks agents to be reliable at mechanical tasks (file formatting, event
logging) where they're weakest. The new model asks agents only for judgment where they're
strongest.

#### Impact on Skills

Skills transform from **file manipulation instructions** to **command invocation instructions**.

Current pattern (~150 lines):
1. Check RBAC (read actor, interpret permissions)
2. Generate word-ID
3. Create file with correct path, frontmatter, three-level body
4. Update INDEX.md
5. Log event via event-log skill

New pattern (~50 lines):
1. Run `aictl create workitem --title "..." --state draft --owner @current`
   (aictl handles: ID, file, RBAC, events, index — all automatically)

Skills shrink by 60-70%. Cross-cutting concerns (RBAC, event logging, INDEX.md) become
automatic. Skills focus purely on JUDGMENT guidance.

#### The Probabilistic Paradigm Survives

The probabilistic model becomes a LAYER on top of a deterministic base:

**Deterministic base (aictl):** File writes authenticated/authorized, every mutation
logged, schema validated, state transitions recorded.

**Probabilistic layer (agents + skills):** WHAT to create, WHEN to transition (interpreting
plain English guards), HOW to respond to suggestions, WHETHER to record reasoning.

#### Migration Path

| Phase | Description |
|-------|------------|
| 0 (Compatibility) | `aibox` CLI continues working. aiadm/aictl as new binaries alongside |
| 1 (Opt-in) | `aiadm init --with-auth` sets up CA. `aiadm migrate --enable-auth` for existing |
| 2 (Default) | New projects default to aictl-managed. Legacy mode still works |
| 3 (Deprecation) | Direct file editing deprecated. Skills updated to use aictl |

Solo developers (Alex persona) can use `--no-auth` mode indefinitely.

#### Performance Assessment

| Operation | Overhead |
|-----------|---------|
| Rust binary startup | 1-5ms cold, <1ms warm |
| Certificate verification | 0.1-1ms |
| Per-operation (read + validate + write + index + log) | 5-20ms |
| 50 operations per session total | ~1 second |

Overhead is dwarfed by LLM inference time (seconds per operation). Not a concern.
For batch scenarios (kaits creating 100 items), `aictl apply -f batch.yaml` amortizes.

#### Complexity Cost

| Added complexity | Mitigation |
|-----------------|------------|
| Two CLIs | Clear naming: aiadm=admin, aictl=daily. Skills hide aictl from agents |
| Certificate management | `aiadm cert init` auto-generates; `aiadm cert issue` single command |
| OS-level permissions | Container user controls; Docker-native |
| Schema management | `apiVersion` field enables versioned migration |

Justified for enterprise/team/kaits. For solo devs: tiered enforcement (no-auth mode).

---

### 4. Kubernetes Certificate Flow

#### CA Creation (kubeadm init)

Three separate trust domains created:
1. **Cluster CA** (`ca.crt`/`ca.key`) — signs API server cert, user certs, kubelet certs
2. **Front-proxy CA** — signs extension API server certs (separate chain)
3. **etcd CA** — signs etcd server, peer, and client certs (separate chain)

Certificates generated under `/etc/kubernetes/pki/`:
- `ca.crt/key` — root CA (crown jewel)
- `apiserver.crt/key` — API server TLS (SANs include cluster DNS, node IP)
- `apiserver-kubelet-client.crt/key` — API server client cert for kubelet calls
- `etcd/ca.crt/key`, `etcd/server.crt/key`, `etcd/peer.crt/key` — etcd PKI
- `sa.key/pub` — RSA keypair for service account JWT signing (not a cert)

**CA key protection:** `ca.key` at `0600` on control plane. Whoever holds it can mint
arbitrary identities. No CRL or OCSP — K8s has NO certificate revocation mechanism.

#### CSR Workflow

1. User generates key: `openssl genrsa -out user.key 2048`
2. Creates CSR: `openssl req -new -key user.key -out user.csr -subj "/CN=alice/O=developers"`
   - CN = Kubernetes username (hardcoded mapping)
   - O = group memberships (hardcoded mapping)
3. Submit as CertificateSigningRequest resource via API
4. Admin approves: `kubectl certificate approve alice-csr`
5. Signed cert retrieved from CSR status

**Auto-approval for kubelets:** Bootstrap token -> kubelet generates CSR ->
`csrapprover` controller auto-approves if well-formed (correct CN/O pattern).

**Certificate rotation:** Kubelet auto-rotates via `--rotate-certificates`. No built-in
rotation for user certs (re-do CSR workflow).

**Critical limitation:** No revocation. Compromised cert options: (1) wait for expiry,
(2) remove RBAC bindings, (3) rotate entire CA.

#### kubeconfig Structure

```yaml
clusters:     # Server endpoints + CA certs to trust
users:        # Credentials (cert+key, token, or exec plugin)
contexts:     # Tuples of (cluster, user, namespace)
current-context: <active context>
```

Three auth methods in `users`: client certificate (base64 PEM), bearer token (JWT),
exec plugin (external binary returning token).

#### Service Accounts (Agent Model)

| Aspect | User Account | Service Account |
|--------|-------------|-----------------|
| Created by | External (cert/OIDC) | Kubernetes API |
| Namespaced | No | Yes |
| Revocable | No (no cert revocation) | Yes (delete SA/token) |
| Stored in K8s | No | Yes (as resource) |

**Projected (bound) tokens (1.24+):** Short-lived JWTs (default 1hr), auto-refreshed,
bound to specific pod (invalid when pod deleted). Major security improvement over
legacy non-expiring tokens.

#### RBAC Mechanics

**Evaluation algorithm:**
1. Extract identity from request (CN/O from cert, or SA name from JWT)
2. Collect ALL RoleBindings in namespace + ALL ClusterRoleBindings
3. For matching bindings, check if referenced Role/ClusterRole allows the verb+resource
4. **ANY allow = ALLOWED** (additive only, no explicit deny)
5. **No match = DENIED** (default deny)

**No explicit deny exists.** Cannot say "user X cannot do Y." Only grant or absence-of-grant.

**Privilege escalation protection:**
- `bind` verb: needed to create RoleBinding referencing a Role
- `escalate` verb: needed to modify Role to grant permissions you don't hold

**`system:masters` backdoor:** Hardcoded group that bypasses all RBAC. Any cert with
`O=system:masters` is cluster-admin. Cannot be restricted. Must protect via CA key control.

#### API Server as Single Enforcement Point

The API server is the ONLY component that:
- Authenticates requests
- Evaluates RBAC
- Runs admission controllers
- Persists state to etcd

**No other component accesses etcd directly.** This is what makes enforcement deterministic:
exactly ONE code path for state changes, and that path enforces all rules.

**Direct etcd access = complete bypass.** No auth, no RBAC, no admission. This is why
etcd has its own CA and mutual TLS.

#### What API Server Does NOT Enforce

| Feature | Where actually enforced |
|---------|----------------------|
| NetworkPolicy | CNI plugin (iptables/eBPF) |
| Resource limits (CPU/mem) | Kubelet (cgroups) |
| Scheduling constraints | kube-scheduler |
| Image pull policy | Kubelet + container runtime |

#### Analogy Mapping to aictl

| Kubernetes | aictl | Key similarity |
|-----------|-------|---------------|
| kube-apiserver | aictl binary | Single enforcement point for all context writes |
| etcd | `context/` filesystem | Backing store with no native access control |
| Namespace | Scope | Isolation boundary for RBAC |
| Cluster CA | aibox CA | Signs identity certs; holder can mint any identity |
| kubeconfig | `~/.aibox/identity.toml` + certs | Endpoint + credentials + context |
| User (cert) | Human operator | CN = username, O = groups |
| Service Account | AI agent identity | Namespaced, revocable, programmatic |
| SA projected token | Agent session token | Short-lived, bound to session |
| Role | Project-scoped permissions | Verb+kind rules within a scope |
| ClusterRole | Global permissions | Cross-project admin |
| Admission controller | aictl validation | Schema + guards + constraints |

#### Enforcement Options for aictl

**Option A — Advisory:** aictl validates but files are plain markdown. Agents can bypass.
Like NetworkPolicy with non-supporting CNI.

**Option B — Gateway (K8s-like):** aictl runs as daemon/server. Context files have
restrictive permissions. Only aictl can write. True mechanical enforcement but requires
daemon.

**Option C — Cryptographic:** Every file change signed by aictl. Unsigned changes
rejected on read. Enforcement in the data itself, no daemon needed.

**Option D — Git-based:** Git server pre-receive hooks enforce RBAC. aictl generates
signed commits. Strongest for team scenarios — immutable audit log (git history) for free.

**Recommendation:** Option C (cryptographic signing) for local enforcement + Option D
(git hooks) for shared/team enforcement. Combined, this gives:
- Solo: signed files detect tampering, no daemon overhead
- Team: git server rejects unauthorized pushes
- Enterprise: both layers active

#### Key Takeaways from K8s for aictl Design

1. **Single enforcement point is non-negotiable** — without it, RBAC is advisory
2. **Identity in the credential** — embed username/roles in cert/token (stateless auth)
3. **Additive-only RBAC** — simpler, sufficient (default deny + grants)
4. **Separate trust domains** — at minimum: identity-signing key vs data-signing key
5. **Short-lived agent tokens** — K8s learned non-expiring SA tokens were a liability
6. **If the store is directly accessible, document that RBAC is advisory**

---

### 5. Synthesis and Recommendations

#### Architecture Recommendation

```
aiadm (kubeadm analog)          aictl (kubectl analog)
- init / reset                  - create / get / describe / delete
- cert generate / approve       - edit / apply / patch
- image build / push            - transition / lint / sync
- container start / stop        - logs / events / metrics
- skill / process install       - auth whoami / can-i
- upgrade / migrate             - search / board / tree / graph
- doctor                        - link / comment / archive
```

#### Enforcement Tiers

| Tier | Target user | Certificates | File lockdown | Audit |
|------|------------|-------------|--------------|-------|
| Solo | Alex persona | None | None | aictl logs (optional) |
| Team | Maria persona | Required | DAC + signing | Deterministic |
| Enterprise | CIO/security | Required | DAC + FUSE/AppArmor + signing | Deterministic + compliance export |
| kaits | Orchestrator | Auto-provisioned | Full | Full + cross-project |

#### Phased Implementation

| Phase | Deliverable |
|-------|------------|
| 1 | aictl CRUD commands (create/get/describe/delete/edit) with schema validation. No auth yet. |
| 2 | `aiadm init --with-auth`, CA creation, cert issue/approve. aictl auth whoami/can-i. |
| 3 | Mechanical RBAC enforcement in aictl. Deterministic audit log. |
| 4 | OS-level lockdown (DAC + signing). Git hook enforcement. |
| 5 | FUSE filesystem (optional, for high-security environments). |

#### Open Questions for Owner

1. **Scope of aictl governance:** Does aictl govern ALL files in context/, or only
   entity files with frontmatter? Research reports and work instructions are "narrative
   content" — should agents edit these directly or through aictl?

2. **Guard evaluation model:** When `aictl transition` is called, does aictl (a) trust
   the agent's assertion that guards are satisfied, or (b) attempt to evaluate guards
   mechanically? Option (a) preserves the probabilistic philosophy. Option (b) requires
   a guard expression language.

3. **Certificate complexity for solo devs:** Is a `--no-auth` mode sufficient, or should
   solo mode use a simpler mechanism (e.g., a local passphrase instead of certificates)?

4. **Rename timing:** When does the rename from `aibox` to `aiadm` happen? Before or
   after the current CLI is feature-complete? The rename is breaking — all docs, skills,
   CLAUDE.md references change.

5. **Structured vs plain-English permissions:** Should Role definitions use structured
   permission specs (verb+kind rules) alongside or instead of plain English? The K8s
   model uses structured rules exclusively. A hybrid (structured for enforcement, plain
   English for documentation) is possible.

6. **Daemon vs CLI-only:** Should aictl run as a long-lived daemon (like the K8s API
   server) for real-time enforcement, or remain a CLI that validates per-invocation?
   Daemon enables: watch/subscribe, incremental index updates, persistent auth session.
   CLI-only is simpler but less capable.
