---
apiVersion: processkit.projectious.work/v1
kind: Note
metadata:
  id: NOTE-20260410_2335-ThriftyOtter-addon-process-package-skill
  created: 2026-04-11
spec:
  title: "Addon / Process Package / Skill Mapping Audit"
  type: reference
  state: permanent
  tags: [addons, skills, packages, audit, orphans]
  skill: research-with-confidence
  source_file: addon-skill-mapping-audit-2026-03.md
---

# Addon / Process Package / Skill Mapping Audit

**Date:** 2026-03-26
**Scope:** All 85 skill templates, 13 process packages, 22 addons

---

## 1. Summary

| Category | Count |
|---|---|
| Total skills | 85 |
| Skills in process packages | 50 |
| Skills in addon `skills:` fields | 12 (4 unique beyond packages) |
| Orphaned skills (no package, no addon) | 31 |

The system has 85 skill templates but only 54 are reachable without explicit `[skills] include`.
The remaining 31 (36%) are orphans that users must discover and manually include.

---

## 2. Complete Mapping Table

### Legend
- **PKG** = Assigned via process package
- **ADDON** = Assigned via addon `skills:` field
- **BOTH** = In both a package and an addon
- **ORPHAN** = Not in any package or addon

Key entries (selected — see source for full table):

| # | Skill | Source | Package / Addon |
|---|---|---|---|
| 1-9 | agent-management, owner-profile, backlog-context, decisions-adr, event-log, context-archiving, standup-context, session-handover, inter-agent-handover | PKG | core, tracking, standups, handover |
| 10-22 | estimation-planning, retrospective, code-review, testing-strategy, debugging, refactoring, tdd-workflow, error-handling, git-workflow, integration-testing, data-science, data-visualization, feature-engineering | PKG | product, code, research |
| 23-24 | documentation, latex-authoring | BOTH | documentation + multiple docs addons |
| 25-51 | design, architecture, security, data, operations, kubernetes, terraform, node, python, go, rust skills | PKG/ADDON | design, architecture, security, data, operations packages; kubernetes, node, python, go, rust addons |
| 61-88 | ai-fundamentals, api-design, caching-strategies, code-generation, database-migration, database-modeling, distributed-tracing, dns-networking, flutter-development, graphql-patterns, grpc-protobuf, java-patterns, linux-administration, llm-evaluation, load-testing, ml-pipeline, nosql-patterns, pixijs-gamedev, postmortem-writing, prompt-engineering, rag-engineering, reflex-python, release-semver, seo-optimization, shell-scripting, sql-patterns, sql-style-guide, webhook-integration | **ORPHAN** | -- |

---

## 3. Orphaned Skills — Recommendations

### 3.1 High Priority — Clear package fit

| Skill | Recommendation | Rationale |
|---|---|---|
| api-design | **architecture** package | API design is a core architecture concern |
| release-semver | **code** package | Release versioning is part of the dev workflow |
| postmortem-writing | **operations** package | Postmortems are incident-response adjacent |
| load-testing | **operations** package | Load testing is an operational concern |
| distributed-tracing | **operations** package | Observability alongside logging/metrics |
| shell-scripting | **operations** package | Operational scripting |
| linux-administration | **operations** package | System administration |
| dns-networking | **operations** package | Network infrastructure |
| caching-strategies | **architecture** package | Cross-cutting architecture pattern |
| database-modeling | **data** package | Data modeling is a core data concern |
| database-migration | **data** package | Schema migration is a data concern |
| sql-patterns | **data** package | SQL is a data access pattern |
| sql-style-guide | **data** package | SQL style is a data concern |
| nosql-patterns | **data** package | NoSQL is a data storage pattern |
| webhook-integration | **architecture** package | Integration pattern |

### 3.2 Medium Priority — AI/ML skills need a home

| Skill | Recommendation |
|---|---|
| ai-fundamentals | New **ai** package or AI addon skills |
| prompt-engineering | New **ai** package or AI addon skills |
| llm-evaluation | New **ai** package or AI addon skills |
| rag-engineering | New **ai** package or AI addon skills |
| ml-pipeline | **research** package or new **ai** package |
| code-generation | New **ai** package |

### Proposed New `ai` Process Package

```
ProcessPackage {
    name: "ai",
    description: "AI engineering, prompt design, LLM evaluation, and RAG",
    skills: &[
        "ai-fundamentals",
        "prompt-engineering",
        "llm-evaluation",
        "rag-engineering",
        "code-generation",
        "ml-pipeline",
    ],
}
```

---

## 4. Validation Issues in Existing Mappings

### 4.1 python addon over-broad includes
- `fastapi-patterns` in python addon: not every Python user needs FastAPI.
- `pandas-polars` is borderline — also in `data` package.
- **Recommendation:** Remove `fastapi-patterns` from python addon; keep `python-best-practices` only.

### 4.2 node addon includes `tailwind`
- Tailwind is a specific CSS framework, not universal to Node.js development.
- **Recommendation:** Remove `tailwind` from node addon. Add to future `frontend` or `css` addon.

### 4.3 operations package is already the largest (8 skills)
Adding recommended orphans would push it to 14 skills.
- **Recommendation:** Consider splitting into `operations` and `infrastructure` sub-packages.

### 4.4 Missing skill template: `inter-agent-handover`
Referenced by handover package but no corresponding directory found.

---

## 5. Priority Order for Fixing Orphans

### Tier 1 — Quick wins (add to existing packages, no new packages)
api-design → architecture; caching-strategies → architecture; webhook-integration → architecture;
release-semver → code; postmortem-writing, load-testing, distributed-tracing → operations;
database-modeling, database-migration, sql-patterns, sql-style-guide, nosql-patterns → data

### Tier 2 — Moderate
shell-scripting, linux-administration, dns-networking → operations; java-patterns → new java addon;
reflex-python → python addon

### Tier 3 — New `ai` package (design decision needed)
ai-fundamentals, prompt-engineering, llm-evaluation, rag-engineering, ml-pipeline, code-generation

### Tier 4 — Niche (keep as orphans or future addons)
flutter-development, pixijs-gamedev, graphql-patterns, grpc-protobuf

---

## 6. Addons Without Any Skills

| Addon | Potential Skills to Add |
|---|---|
| ai-claude | `prompt-engineering`, `ai-fundamentals` |
| ai-gemini | `prompt-engineering`, `ai-fundamentals` |
| ai-mistral | `prompt-engineering`, `ai-fundamentals` |
| ai-aider | `prompt-engineering`, `ai-fundamentals` |
| cloud-aws | `dns-networking`, `secret-management` |
| cloud-gcp | `dns-networking`, `secret-management` |
| cloud-azure | `dns-networking`, `secret-management` |

---

## 7. Coverage After Proposed Changes

| Category | Before | After |
|---|---|---|
| Package-assigned skills | 50 | 68 |
| Addon-only skills | 10 | 11 |
| Orphaned skills | 31 | 4 |
| Total skills | 85 | 85 |

Remaining orphans: `flutter-development`, `pixijs-gamedev`, `graphql-patterns`, `grpc-protobuf`
— all niche framework skills appropriate for future addons.
