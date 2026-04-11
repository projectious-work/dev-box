---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-PluckyThorn-aiadm-aictl-architecture-kubernetes
  created: 2026-04-11
spec:
  title: "aiadm/aictl Architecture — Kubernetes-Inspired CLI Split for Deterministic RBAC"
  type: reference
  state: permanent
  tags: [architecture, aiadm, aictl, kubernetes, rbac, certificates, cli-split]
  skill: research-with-confidence
  source_file: aiadm-aictl-architecture-2026-03.md
---

# aiadm/aictl Architecture — Kubernetes-Inspired CLI Split for Deterministic RBAC

**Date:** 2026-03-28
**Status:** Draft

Research into the proposed aiadm/aictl architectural split: renaming aibox to aiadm
(infrastructure) and introducing aictl (context operations) with certificate-based
authentication, OS-level file lockdown, and deterministic audit logging.

---

## Overview

- Four research areas: OS-level lockdown, kubectl-to-aictl command mapping, impact on 50 DISC-001 decisions, Kubernetes certificate/RBAC mechanics
- The proposal is architecturally sound but OS-level enforcement has fundamental limits
- 14 decisions unchanged, 17 modified, 7 superseded, 12 strengthened
- Recommended approach: layered enforcement with cryptographic signing

---

## 1. OS-Level Lockdown Mechanisms

The goal: make `context/` writable only by the `aictl` process, preventing AI agents with shell access from bypassing RBAC by editing files directly.

### Mechanisms Assessed

| Mechanism | Feasibility | Security vs root agent | Git compatibility |
|---|---|---|---|
| Unix DAC (file permissions) | Low effort | None (trivially bypassed) | Works |
| Host-applied AppArmor | Medium | HIGH (host kernel) | Problematic (git bypass) |
| Linux capability dropping | Easy | Low-medium | Breaks standard tools |
| Read-only mount + sidecar | Complex | HIGH (VFS-level) | BROKEN (showstopper) |
| FUSE process-tree | High effort | Medium | GOOD (best story) |

### The Fundamental Problem

**If git can write to `context/`, and the agent can run git, then the agent can write to `context/` via git.** The FUSE process-tree verification approach is the only one that can distinguish "git invoked by aictl" from "git invoked by agent shell."

### Honest Assessment

**If an AI agent has unrestricted root shell access inside a container, no intra-container mechanism provides absolute protection.** The host-applied AppArmor profile is the only genuinely robust mechanism.

### Recommended Layered Approach

| Layer | Mechanism | Purpose |
|-------|-----------|---------|
| 1 (Speed bump) | Unix DAC | Prevent accidental writes |
| 2 (Kernel) | Host AppArmor OR FUSE | Mechanical enforcement |
| 3 (Git mediation) | FUSE process-tree checks | Allow git only via aictl |
| 4 (Detection) | Cryptographic signatures | Detect tampering post-facto |

**Practical start:** Layer 1 (DAC) + Layer 4 (signing). Evolve toward FUSE when implementation budget allows.

---

## 2. kubectl to aictl Command Mapping

### Core CRUD (1:1 mapping)

| kubectl | aictl | Notes |
|---------|-------|-------|
| `create <kind> <name>` | `create <kind> [--field=val]` | Auth, RBAC, ID generation, file creation, index update, event log |
| `get <kind> [name]` | `get <kind> [id] [--state=X]` | Query SQLite with filters |
| `describe <name>` | `describe <id>` | Full file + cross-refs + events + state machine |
| `delete <name>` | `delete <id> [--force]` | Soft-delete (archive) by default |
| `edit <name>` | `edit <id>` | Opens $EDITOR, validates on save |
| `apply -f <file>` | `apply -f <file>` | Declarative create-or-update |

### New aictl Commands (no kubectl equivalent)

`transition`, `lint`, `sync`, `search`, `comment`, `link/unlink`, `tree`, `board`, `id generate`, `import/export`, `graph`, `archive/unarchive`, `history`, `rollback`

### Statistics

- kubectl commands with direct aictl equivalents: **24**
- kubectl commands adapted semantically: **4**
- kubectl commands with no aictl equivalent: **16** (container/node ops → aiadm)
- New aictl commands with no kubectl equivalent: **12**
- **Total aictl commands: ~40**

### aiadm vs aictl Split

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

---

## 3. Impact on DISC-001 Decisions

| Classification | Count |
|---|---|
| UNCHANGED | 14 |
| MODIFIED | 17 |
| SUPERSEDED | 7 |
| STRENGTHENED | 12 |

### Critical Superseded Decisions

**Decision 9 (RBAC via plain English → mechanical):** Agents can no longer edit state directly. `aictl transition BACK-xxx --to state` replaces direct file editing.

**Decision 19 (Probabilistic RBAC → deterministic):** Certificate identifies actor → actor's roles → permissions checked BEFORE allowing operation.

**Decision 20 (Dual event sources → deterministic base):** aictl command log is now 100% deterministic. Agent reasoning log is probabilistic layer on top.

### The Boundary Shift (Decision 16)

| Layer | Current model | New model |
|-------|--------------|-----------|
| Infrastructure | aibox (init, sync, lint, migrate) | aiadm (init, images, containers, schema, certs) |
| Tooling | — | aictl (CRUD, transitions, queries, validation) |
| Application | Agents (everything else) | Agents (judgment: what to create, when to transition) |

### Impact on Skills

Skills shrink by **60-70%**: from ~150 lines (manual file creation + RBAC + index + events) to ~50 lines (just `aictl create workitem --title "..." --state draft`). Cross-cutting concerns become automatic.

---

## 4. Kubernetes Certificate/RBAC Mechanics

### Key Concepts for aictl Design

1. **Single enforcement point is non-negotiable** — without it, RBAC is advisory
2. **Identity in the credential** — embed username/roles in cert/token (stateless auth)
3. **Additive-only RBAC** — default deny + grants; no explicit deny
4. **Separate trust domains** — identity-signing key vs data-signing key
5. **Short-lived agent tokens** — non-expiring SA tokens are a liability
6. **If the store is directly accessible, document that RBAC is advisory**

### Enforcement Recommendation for aictl

| Option | Description | Recommendation |
|---|---|---|
| A — Advisory | aictl validates but files are plain markdown; agents can bypass | Solo dev only |
| B — Gateway | aictl runs as daemon; context files have restrictive permissions | Team |
| C — Cryptographic | Every file change signed by aictl; unsigned changes rejected on read | Combined with D |
| D — Git-based | Git server pre-receive hooks enforce RBAC; aictl generates signed commits | Team/enterprise |

**Recommendation:** Option C (cryptographic signing) for local + Option D (git hooks) for team/enterprise.

---

## 5. Enforcement Tiers

| Tier | Target user | Certificates | File lockdown | Audit |
|------|------------|-------------|--------------|-------|
| Solo | Alex persona | None | None | aictl logs (optional) |
| Team | Maria persona | Required | DAC + signing | Deterministic |
| Enterprise | CIO/security | Required | DAC + FUSE/AppArmor + signing | Deterministic + compliance export |
| kaits | Orchestrator | Auto-provisioned | Full | Full + cross-project |

---

## 6. Open Questions

1. **Scope of aictl governance:** All files in context/ or only entity files with frontmatter?
2. **Guard evaluation model:** Does aictl trust agent's assertion or evaluate guards mechanically?
3. **Certificate complexity for solo devs:** `--no-auth` mode sufficient?
4. **Rename timing:** When does `aibox` → `aiadm` rename happen?
5. **Structured vs plain-English permissions:** Hybrid possible (structured for enforcement, plain English for documentation).
6. **Daemon vs CLI-only:** Watch/subscribe capability requires daemon model.
