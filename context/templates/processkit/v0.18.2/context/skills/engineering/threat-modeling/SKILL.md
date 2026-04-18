---
name: threat-modeling
description: |
  Threat modeling with STRIDE — data flow diagrams, trust boundaries, risk. Use when designing a new system, reviewing architecture for security, conducting a security review, or prioritizing security work by risk.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-threat-modeling
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Threat Modeling

## Intro

A threat model is a structured analysis of how a system could be
attacked. STRIDE walks each component and data flow through six
threat categories, producing a prioritized list of risks and
mitigations.

## Overview

### STRIDE categories

| Category                   | Threat                        | Property violated |
|----------------------------|-------------------------------|-------------------|
| **S**poofing               | Pretending to be someone else | Authentication    |
| **T**ampering              | Modifying data or code        | Integrity         |
| **R**epudiation            | Denying an action occurred    | Non-repudiation   |
| **I**nformation Disclosure | Exposing data                 | Confidentiality   |
| **D**enial of Service      | Making the system unavailable | Availability      |
| **E**levation of Privilege | Gaining unauthorized access   | Authorization     |

### Step 1 — Draw a data flow diagram

Map processes, data stores, data flows, and external entities. Mark
trust boundaries where data crosses between trust levels.

```
                    Trust Boundary
                    +---------------------------------+
  [User Browser] ---|---> [Web Server] ---> [App Server] ---> [Database]
                    |         |                  |
                    |         v                  v
                    |    [Static Files]    [Cache (Redis)]
                    +---------------------------------+
                           |
                           v
                    [External Auth Provider]
```

- **Processes** — web server, app server, background workers.
- **Data stores** — database, cache, file system, message queue.
- **Data flows** — HTTP requests, database queries, API calls.
- **External entities** — users, third-party services, admins.
- **Trust boundaries** — network segments, process boundaries,
  privilege levels.

### Step 2 — Identify trust boundaries

Each crossing is a potential attack point:

- Internet to DMZ.
- DMZ to internal network.
- Application to database.
- Service to service in a microservice mesh.
- Regular user vs admin privilege levels.

### Step 3 — Apply STRIDE to each element

For every process, data store, and data flow, ask:

- **Spoofing** — can an attacker impersonate a legitimate entity?
- **Tampering** — can data be modified in transit or at rest?
- **Repudiation** — can a user deny performing an action?
- **Information disclosure** — can sensitive data leak?
- **Denial of service** — can this component be overwhelmed?
- **Elevation of privilege** — can a user gain higher privileges?

### Step 4 — Risk assessment

Rate each threat by likelihood and impact:

| Rating | Likelihood                      | Impact                           |
|--------|---------------------------------|----------------------------------|
| High   | Exploitable with public tools   | Data breach, full compromise     |
| Medium | Requires some skill or access   | Partial data exposure, downtime  |
| Low    | Requires insider access or luck | Minor data exposure, degradation |

**Risk = Likelihood × Impact**

| Likelihood \ Impact | High     | Medium | Low    |
|---------------------|----------|--------|--------|
| High                | Critical | High   | Medium |
| Medium              | High     | Medium | Low    |
| Low                 | Medium   | Low    | Low    |

### Step 5 — Mitigations

Map each STRIDE category to common mitigations:

| STRIDE category        | Common mitigations                                |
|------------------------|---------------------------------------------------|
| Spoofing               | MFA, certificate pinning, mutual TLS              |
| Tampering              | Digital signatures, checksums, immutable logs     |
| Repudiation            | Audit logging, tamper-evident logs, signatures    |
| Information Disclosure | Encryption (TLS, AES), access controls, masking   |
| Denial of Service      | Rate limiting, autoscaling, CDN, circuit breakers |
| Elevation of Privilege | Least privilege, input validation, sandboxing     |

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Skipping the data flow diagram and jumping straight to a threat list.** Without a data flow diagram with trust boundaries, threats are enumerated without knowing where data crosses between trust levels — the points where attackers actually operate. The DFD is not optional; it determines which threats are relevant to which components.
- **Threat modeling once at design time and never revisiting.** A system that has grown to include new external integrations, new microservices, or new data flows has a different attack surface than it did at initial design. A threat model that is not updated when the architecture changes is a threat model of a system that no longer exists.
- **Treating "low likelihood" as "permanently ignore."** A threat rated low likelihood because it requires insider access or sophisticated skill is not safe to dismiss forever. Circumstances change: attackers gain new capabilities, insiders become disgruntled, and "low likelihood" events happen. Re-rate risks annually or when the environment changes.
- **Mitigations with no owner or ticket.** A threat model that lists 15 mitigations in a document with no associated tasks means zero of them will be implemented. Every mitigation must be tracked to a ticket or commit with an owner. A mitigation that is only in the threat model document has not been mitigated.
- **Modeling the intended architecture, not the production system.** The diagram shows a load balancer in front of the API, but production has a debug port open directly to the application server. The threat model of the intended architecture misses the actual attack surface. Threat model what is deployed, not what was planned.
- **Confusing authentication threats with authorization threats.** Spoofing (STRIDE-S) is about impersonating an identity — a weak login mechanism. Elevation of privilege (STRIDE-E) is about gaining capabilities beyond what the identity is authorized for — a broken access control check. Fixing authentication does not fix authorization. Apply both categories separately to each component.
- **Stopping at the application boundary and ignoring CI/CD and cloud infrastructure.** The CI/CD pipeline, container registry, cloud console, and IAM configuration are part of the attack surface — a compromised build pipeline can inject code into every deployment. Include infrastructure, supply chain, and human entry points in the attack surface mapping.

## Full reference

### Attack surface mapping

Enumerate every entry point an attacker could use:

- **Network** — open ports, exposed services, public endpoints.
- **Application** — API endpoints, file upload, search, user input
  fields.
- **Authentication** — login, password reset, session management.
- **Data** — database access, file storage, backups, logs.
- **Infrastructure** — cloud console, CI/CD pipeline, container
  registry.
- **Human** — phishing targets, social engineering, insider threats.

### Output format

Structure the threat model as a single document:

```
## Threat Model: [System Name]

### System Description
Brief description of the system and its purpose.

### Data Flow Diagram
ASCII diagram showing components and trust boundaries.

### Assets
List of valuable data and resources to protect.

### Threats
| ID | STRIDE | Component | Threat | Risk | Mitigation |
|----|--------|-----------|--------|------|------------|
| T1 | S      | Login API | Credential stuffing | High | Rate limiting, MFA |
| T2 | I      | Database  | SQL injection leaks PII | Critical | Parameterized queries |

### Recommendations
Prioritized list of security improvements.
```

### When to threat model

- During initial design of any system handling sensitive data.
- Before adding a new external integration or trust boundary.
- After an incident, to find adjacent risks.
- On a periodic schedule for high-value systems (annually or per
  major release).

### Anti-patterns

- Threat modeling once at design time and never revisiting.
- Skipping the data flow diagram and going straight to a threat list.
- Treating "low likelihood" as "ignore forever".
- Producing a document nobody reads. The output should drive a
  prioritized punch list of work, not sit in a wiki.
- Modeling abstract systems instead of the actual architecture in
  production.

### Tips for sharper models

- Walk the diagram with someone who didn't build the system; gaps
  surface fast.
- Track every mitigation back to a ticket or commit. A mitigation
  with no owner is a mitigation that won't happen.
- Re-rate risks after mitigations land — residual risk is what
  matters for prioritization.
