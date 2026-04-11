---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-CalmWolf-software-development-process-deep
  created: 2026-04-11
spec:
  title: "Software Development Process Deep Dive: Primitive Coverage Analysis"
  type: reference
  state: permanent
  tags: [process, primitives, sdlc, state-machines, gap-analysis, coverage]
  skill: research-with-confidence
  source_file: software-dev-process-deep-dive-2026-03.md
---

# Software Development Process Deep Dive: Primitive Coverage Analysis

**Date:** 2026-03-27
**Relates to:** BACK-055, DISC-001
**Purpose:** Exhaustive mapping of the full software development lifecycle against the 17 aibox primitives to identify gaps in coverage.

---

## 1. Lifecycle → Primitive Mapping (Summary)

The 17 primitives (WorkItem, Event, DecisionRecord, Artifact, Role, Process, StateMachine, Category, CrossReference, Gate, Metric, Schedule, Scope, Constraint, Context, Discussion, Actor) cover all 10 lifecycle stages: Product Discovery, Requirements, Architecture, Implementation, Testing, CI/CD, Release Management, Operations, Maintenance, End-of-Life.

**Key insight:** All 17 primitives are exercised by the software development lifecycle. No primitive is redundant.

### Primitive Coverage Matrix (H=heavy, M=moderate, L=light)

| Primitive | Discovery | Reqts | Arch | Impl | Test | CI/CD | Release | Ops | Maint | EOL |
|---|---|---|---|---|---|---|---|---|---|---|
| Work Item | H | H | M | H | M | L | M | H | H | M |
| Event | L | L | L | H | H | H | H | H | M | M |
| Decision | M | M | H | L | L | L | M | M | M | H |
| Artifact | H | H | H | H | H | H | H | H | M | H |
| Gate | M | M | H | L | H | H | H | M | M | M |
| Constraint | L | H | H | M | H | H | H | H | H | H |

---

## 2. Gap Analysis: What is NOT Covered

| Gap | Recommended Model | Priority |
|---|---|---|
| Environments (dev/staging/prod) | Scope subtype `environment` with deployed_version, promotion_gates, config | High |
| Deployment history | Event type `deployment` with before/after artifact versions | High |
| Feature flags | Work Item subtype `feature-flag` with per-environment state | Medium |
| External dependencies | Artifact subtype `external-dependency` with version/license/security fields | Medium |
| Escalation chains | Process subtype with ordered Role references | Low |

**Key principle:** No new primitives needed. All gaps are modeled as subtypes or compositions of existing primitives.

---

## 3. Missing Process Templates (Prioritized)

**Must-have for v1:**
1. `incident-response.md` — every software project needs this
2. `technical-design.md` — bridges requirements and implementation
3. `spike-research.md` — structures exploration work
4. `hotfix.md` — emergency variant of bug-fix with different gates

**Should-have for v1.x:**
5. `dependency-update.md`, 6. `retrospective.md`, 7. `postmortem.md`
8. `backlog-grooming.md`, 9. `deprecation.md`, 10. `security-review.md`

**Nice-to-have:** `on-call-handoff.md`, `rollback.md`, `experiment.md`, `onboarding.md`, `architecture-review.md`, `end-of-life.md`, `capacity-planning.md`

---

## 4. State Machine Definitions

### Feature Lifecycle (simplified)
`idea → proposed → accepted → planned → in-design → in-development → in-review → in-testing → ready-for-release → released → done`
Terminal states: `deferred`, `rejected`, `cancelled`

### Bug Lifecycle
`reported → triaged → confirmed → in-progress → in-review → in-testing → ready-for-release → released → verified-fixed`
Terminal states: `duplicate`, `invalid`, `wont-fix`, `cannot-reproduce`

### Incident Lifecycle

| State | SLA (SEV1) |
|---|---|
| `detected` | 5 min to acknowledge |
| `acknowledged` | 15 min to investigate |
| `investigating` | 1 hour to mitigation |
| `mitigating` | 4 hours to resolve |
| `resolved` → `postmortem` → `closed` | 5 business days for postmortem |

**Severity taxonomy:**

| Severity | Description | Response SLA | Resolution SLA |
|---|---|---|---|
| SEV1 / Critical | Service completely down, data loss, security breach | 5 min | 4 hours |
| SEV2 / Major | Service severely degraded, major feature unavailable | 15 min | 8 hours |
| SEV3 / Minor | Partial degradation, workaround available | 1 hour | 48 hours |
| SEV4 / Low | Cosmetic issue, minor inconvenience | 4 hours | 1 week |

### Release Lifecycle
`planning → development → feature-freeze → stabilization → release-candidate → staging-verification → production-deploy → monitoring → stable`
Safety exit: `rolled-back` → hotfix cycle

### PR / Code Review Lifecycle
`draft → open → changes-requested → updated → approved → merged`

**Gates in code review:** CI pipeline, minimum approvals, no unresolved threads, branch up-to-date, coverage threshold, security scan.

---

## 5. Key Architectural Insight

The primitives work because they are **composable**. Every real-world activity is modeled as a COMPOSITION of primitives:

- **A deployment** = Process + Event + Gate + Context + Artifact
- **An incident** = WorkItem + StateMachine + Category + Constraint + Process + Event + Artifact + Decision
- **A code review** = Process + Gate + Role + Artifact + Event + Constraint

This composability is the ontology's strength. aibox should lean into it rather than creating specialized primitives for every software development concept.
