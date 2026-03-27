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

The system has 85 skill templates but only 54 are reachable without explicit
`[skills] include`. The remaining 31 (36%) are orphans that users must
discover and manually include.

---

## 2. Complete Mapping Table

### Legend

- **PKG** = Assigned via process package
- **ADDON** = Assigned via addon `skills:` field
- **BOTH** = In both a package and an addon
- **ORPHAN** = Not in any package or addon

| # | Skill | Source | Package / Addon |
|---|---|---|---|
| 1 | agent-management | PKG | core |
| 2 | owner-profile | PKG | core |
| 3 | backlog-context | PKG | tracking |
| 4 | decisions-adr | PKG | tracking |
| 5 | event-log | PKG | tracking |
| 6 | context-archiving | PKG | tracking |
| 7 | standup-context | PKG | standups |
| 8 | session-handover | PKG | handover |
| 9 | inter-agent-handover | PKG | handover |
| 10 | estimation-planning | PKG | product |
| 11 | retrospective | PKG | product |
| 12 | code-review | PKG | code |
| 13 | testing-strategy | PKG | code |
| 14 | debugging | PKG | code |
| 15 | refactoring | PKG | code |
| 16 | tdd-workflow | PKG | code |
| 17 | error-handling | PKG | code |
| 18 | git-workflow | PKG | code |
| 19 | integration-testing | PKG | code |
| 20 | data-science | PKG | research |
| 21 | data-visualization | PKG | research |
| 22 | feature-engineering | PKG | research |
| 23 | documentation | BOTH | documentation + addons: docs-hugo, docs-mdbook, docs-mkdocs, docs-starlight, docs-docusaurus, docs-zensical, latex, typst |
| 24 | latex-authoring | BOTH | documentation + addon: latex |
| 25 | excalidraw | PKG | design |
| 26 | infographics | PKG | design |
| 27 | logo-design | PKG | design |
| 28 | frontend-design | PKG | design |
| 29 | mobile-app-design | PKG | design |
| 30 | software-architecture | PKG | architecture |
| 31 | system-design | PKG | architecture |
| 32 | domain-driven-design | PKG | architecture |
| 33 | event-driven-architecture | PKG | architecture |
| 34 | secure-coding | PKG | security |
| 35 | threat-modeling | PKG | security |
| 36 | dependency-audit | PKG | security |
| 37 | auth-patterns | PKG | security |
| 38 | secret-management | PKG | security |
| 39 | dependency-management | PKG | security |
| 40 | data-pipeline | PKG | data |
| 41 | data-quality | PKG | data |
| 42 | pandas-polars | BOTH | data + addon: python |
| 43 | embedding-vectordb | PKG | data |
| 44 | ci-cd-setup | PKG | operations |
| 45 | dockerfile-review | PKG | operations |
| 46 | container-orchestration | BOTH | operations + addon: kubernetes |
| 47 | logging-strategy | PKG | operations |
| 48 | metrics-monitoring | PKG | operations |
| 49 | incident-response | PKG | operations |
| 50 | alerting-oncall | PKG | operations |
| 51 | performance-profiling | PKG | operations |
| 52 | kubernetes-basics | ADDON | addon: kubernetes |
| 53 | terraform-basics | ADDON | addon: infrastructure |
| 54 | typescript-patterns | ADDON | addon: node |
| 55 | tailwind | ADDON | addon: node |
| 56 | python-best-practices | ADDON | addon: python |
| 57 | fastapi-patterns | ADDON | addon: python |
| 58 | go-conventions | ADDON | addon: go |
| 59 | concurrency-patterns | ADDON | addon: go, rust |
| 60 | rust-conventions | ADDON | addon: rust |
| 61 | ai-fundamentals | **ORPHAN** | -- |
| 62 | api-design | **ORPHAN** | -- |
| 63 | caching-strategies | **ORPHAN** | -- |
| 64 | code-generation | **ORPHAN** | -- |
| 65 | database-migration | **ORPHAN** | -- |
| 66 | database-modeling | **ORPHAN** | -- |
| 67 | distributed-tracing | **ORPHAN** | -- |
| 68 | dns-networking | **ORPHAN** | -- |
| 69 | flutter-development | **ORPHAN** | -- |
| 70 | graphql-patterns | **ORPHAN** | -- |
| 71 | grpc-protobuf | **ORPHAN** | -- |
| 72 | java-patterns | **ORPHAN** | -- |
| 73 | linux-administration | **ORPHAN** | -- |
| 74 | llm-evaluation | **ORPHAN** | -- |
| 75 | load-testing | **ORPHAN** | -- |
| 76 | ml-pipeline | **ORPHAN** | -- |
| 77 | nosql-patterns | **ORPHAN** | -- |
| 78 | pixijs-gamedev | **ORPHAN** | -- |
| 79 | postmortem-writing | **ORPHAN** | -- |
| 80 | prompt-engineering | **ORPHAN** | -- |
| 81 | rag-engineering | **ORPHAN** | -- |
| 82 | reflex-python | **ORPHAN** | -- |
| 83 | release-semver | **ORPHAN** | -- |
| 84 | seo-optimization | **ORPHAN** | -- |
| 85 | shell-scripting | **ORPHAN** | -- |
| -- | inter-agent-handover | PKG | handover (no template dir found) |
| 86 | sql-patterns | **ORPHAN** | -- |
| 87 | sql-style-guide | **ORPHAN** | -- |
| 88 | webhook-integration | **ORPHAN** | -- |

**Note:** `inter-agent-handover` is referenced by the handover package but
was not in the `ls` output of skill template directories. This may be a
missing template or it may be embedded elsewhere.

---

## 3. Orphaned Skills -- Full List with Recommendations

### 3.1 High Priority -- Clear package fit

These orphans have an obvious home in an existing package or addon.

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

### 3.2 Medium Priority -- Addon-gated skills

These should be added to existing or new addon `skills:` fields.

| Skill | Recommendation | Rationale |
|---|---|---|
| java-patterns | New **java** addon | Language-specific, like go/rust/python addons |
| flutter-development | New **flutter** addon (or **dart** language addon) | Framework-specific |
| reflex-python | **python** addon | Python web framework |
| fastapi-patterns | Already in python addon | (Validation issue -- see section 5) |
| pixijs-gamedev | New **gamedev** addon or **node** addon | Framework-specific for JS game dev |
| seo-optimization | **node** addon or new **web** addon | Primarily a web/frontend concern |
| tailwind | Already in node addon | Confirmed correctly placed |

### 3.3 Medium Priority -- AI/ML skills need a home

These form a natural cluster that no current package or addon covers.

| Skill | Recommendation | Rationale |
|---|---|---|
| ai-fundamentals | New **ai** package or AI addon skills | Core AI knowledge |
| prompt-engineering | New **ai** package or AI addon skills | Prompt design is foundational for AI work |
| llm-evaluation | New **ai** package or AI addon skills | LLM evaluation and benchmarking |
| rag-engineering | New **ai** package or AI addon skills | RAG is a key AI pattern |
| ml-pipeline | **research** package or new **ai** package | ML pipeline overlaps with data science |
| code-generation | New **ai** package | AI-assisted code generation patterns |

### 3.4 Low Priority -- Niche skills, fine as orphans

| Skill | Recommendation | Rationale |
|---|---|---|
| graphql-patterns | Keep orphan; add to future **graphql** addon | Niche API technology |
| grpc-protobuf | Keep orphan; add to future **grpc** addon | Niche API technology |
| concurrency-patterns | Already in go + rust addons | Correctly addon-gated |

---

## 4. Proposed New Packages and Addon Changes

### 4.1 New Process Package: `ai`

**Rationale:** AI/ML skills form a coherent cluster that many users need but
no package currently provides. The existing `research` package covers data
science but not AI engineering.

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

### 4.2 Expand Existing Packages

**architecture** -- add 3 skills:
- `api-design`
- `caching-strategies`
- `webhook-integration`

**code** -- add 1 skill:
- `release-semver`

**operations** -- add 5 skills:
- `postmortem-writing`
- `load-testing`
- `distributed-tracing`
- `shell-scripting`
- `linux-administration`
- `dns-networking`

(This would bring operations to 14 skills. Consider splitting into
`operations` + `infrastructure` sub-packages if this feels too large.)

**data** -- add 4 skills:
- `database-modeling`
- `database-migration`
- `sql-patterns`
- `sql-style-guide`
- `nosql-patterns`

(This would bring data to 9 skills.)

### 4.3 New Addon Skills Fields

| Addon | Add Skills |
|---|---|
| python | `reflex-python` |
| node | `seo-optimization` (debatable) |
| New: java | `java-patterns` |
| New: flutter/dart | `flutter-development` |
| ai-claude / ai-gemini / ai-mistral | `prompt-engineering`, `ai-fundamentals` (if no `ai` package) |
| cloud-aws / cloud-gcp / cloud-azure | `dns-networking` (if not in operations) |

### 4.4 New Preset: `ai-product`

If the `ai` package is created:
```
ProcessPreset {
    name: "ai-product",
    description: "AI product development (managed + code + ai + data)",
    packages: &["core", "tracking", "standups", "handover", "code", "ai", "data"],
}
```

---

## 5. Validation Issues in Existing Mappings

### 5.1 python addon includes `fastapi-patterns` and `pandas-polars`

**Issue:** Not every Python user needs FastAPI (a web framework) or
pandas/polars (data manipulation). A Django or Flask developer gets
FastAPI patterns loaded unnecessarily.

**Recommendation:** Remove `fastapi-patterns` from the python addon. Users
who need it can add it via `[skills] include`. Keep `python-best-practices`
as the only universal Python skill. `pandas-polars` is borderline -- it is
also in the `data` package, so Python+data users get it regardless. Consider
removing from python addon to avoid forcing it on non-data Python users.

### 5.2 node addon includes `tailwind`

**Issue:** Tailwind CSS is a specific CSS framework, not universal to all
Node.js development. Backend Node.js developers or those using other CSS
approaches do not need it.

**Recommendation:** Remove `tailwind` from the node addon. Add it to a
future `frontend` or `css` addon, or leave as explicit include. Keep
`typescript-patterns` as the universal Node skill.

### 5.3 go and rust addons both include `concurrency-patterns`

**Issue:** This is not a problem per se -- concurrency is important for both
languages. But the skill is a generic concurrency skill, not language-specific.
If a user has both Go and Rust addons, the skill is deduplicated correctly.

**Status:** Acceptable. No change needed.

### 5.4 Six docs addons all map to `documentation` skill

**Issue:** All six docs-framework addons (hugo, mdbook, mkdocs, starlight,
docusaurus, zensical) include the same `documentation` skill. This is correct
but redundant if the user also selects the `documentation` package. No harm
since skills are deduplicated.

**Status:** Acceptable. No change needed.

### 5.5 operations package is already the largest (8 skills)

**Issue:** Adding the recommended orphans would push it to 14 skills.
This risks becoming a grab-bag.

**Recommendation:** Consider splitting into:
- `operations` (ci-cd-setup, dockerfile-review, container-orchestration, logging-strategy, metrics-monitoring, incident-response, alerting-oncall, postmortem-writing, load-testing, distributed-tracing)
- `infrastructure` package (linux-administration, shell-scripting, dns-networking, performance-profiling)

### 5.6 Missing skill template: `inter-agent-handover`

**Issue:** The `handover` package references `inter-agent-handover` but no
corresponding directory exists in `templates/skills/`. This is either a
missing template or the skill is generated/embedded differently.

**Recommendation:** Verify whether this template exists elsewhere or needs
to be created.

---

## 6. Priority Order for Fixing Orphans

### Tier 1 -- Quick wins (add to existing packages, no new packages needed)

1. `api-design` -> architecture
2. `caching-strategies` -> architecture
3. `webhook-integration` -> architecture
4. `release-semver` -> code
5. `postmortem-writing` -> operations
6. `load-testing` -> operations
7. `distributed-tracing` -> operations
8. `database-modeling` -> data
9. `database-migration` -> data
10. `sql-patterns` -> data
11. `sql-style-guide` -> data
12. `nosql-patterns` -> data

**Effort:** One PR modifying `process_registry.rs` and updating tests.

### Tier 2 -- Moderate (add to operations, create java addon)

13. `shell-scripting` -> operations
14. `linux-administration` -> operations
15. `dns-networking` -> operations
16. `java-patterns` -> new java addon YAML
17. `reflex-python` -> python addon YAML
18. `seo-optimization` -> evaluate, possibly node addon or orphan

**Effort:** One PR for operations expansion + new addon YAMLs.

### Tier 3 -- New `ai` package (design decision needed)

19. `ai-fundamentals` -> new ai package
20. `prompt-engineering` -> new ai package
21. `llm-evaluation` -> new ai package
22. `rag-engineering` -> new ai package
23. `ml-pipeline` -> new ai package
24. `code-generation` -> new ai package

**Effort:** New package in `process_registry.rs`, possibly new preset, docs update.

### Tier 4 -- Niche (keep as orphans or future addons)

25. `flutter-development` -> future flutter addon
26. `pixijs-gamedev` -> future gamedev addon
27. `graphql-patterns` -> future graphql addon or keep orphan
28. `grpc-protobuf` -> future grpc addon or keep orphan

**Effort:** None immediate. Track as future addon candidates.

---

## 7. Addons Without Any Skills

The following addons define no `skills:` field at all:

| Addon | Category | Potential Skills to Add |
|---|---|---|
| ai-claude | ai | `prompt-engineering`, `ai-fundamentals` |
| ai-gemini | ai | `prompt-engineering`, `ai-fundamentals` |
| ai-mistral | ai | `prompt-engineering`, `ai-fundamentals` |
| ai-aider | ai | `prompt-engineering`, `ai-fundamentals` |
| cloud-aws | tools | `dns-networking`, `secret-management` |
| cloud-gcp | tools | `dns-networking`, `secret-management` |
| cloud-azure | tools | `dns-networking`, `secret-management` |
| preview-enhanced | tools | (none -- media tools, no matching skill) |

---

## 8. Coverage After Proposed Changes

If all Tier 1-3 recommendations are implemented:

| Category | Before | After |
|---|---|---|
| Package-assigned skills | 50 | 68 |
| Addon-only skills | 10 | 11 |
| Orphaned skills | 31 | 4 |
| Total skills | 85 | 85 |

Remaining orphans: `flutter-development`, `pixijs-gamedev`, `graphql-patterns`,
`grpc-protobuf` -- all niche framework skills appropriate for future addons.
