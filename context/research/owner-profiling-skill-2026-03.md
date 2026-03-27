# Owner Profiling Skill Design — Automated Owner Profile Learning

**Date:** 2026-03-26
**Status:** Draft

---

## 1. Problem Statement

Every aibox project has a `context/OWNER.md` file that captures the project owner's identity, preferences, and working context. Today this file is manually written and rarely updated. It captures a static snapshot -- name, role, language preferences, timezone -- but misses the rich behavioral patterns that emerge over dozens of agent interactions.

An agent that has worked with an owner for 50 sessions knows things that are not in OWNER.md: that the owner prefers terse commit messages, dislikes unnecessary abstractions, always wants to see test output before merging, and tends to work in 2-hour bursts with tight scope. This knowledge dies with each session unless explicitly recorded.

The goal is a skill that teaches agents to observe these patterns during normal work and incrementally enrich OWNER.md, turning it from a static identity card into a living preference profile that improves every agent session.

---

## 2. Observable Signals

### 2.1 Communication Style

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Message length | Average length of owner messages over a session | "Prefers terse messages; typically 1-3 sentences per instruction" |
| Formality level | Presence of greetings, please/thanks, full sentences vs fragments | "Direct and informal; rarely uses pleasantries in task instructions" |
| Technical jargon density | Domain-specific terms without explanation | "Uses Rust ecosystem terms freely (lifetimes, borrowing, trait objects) -- no need to explain" |
| Emoji usage | Frequency and context of emoji | "No emoji in technical discussions; occasional thumbs-up for approval" |
| Question patterns | How they ask for information (open-ended vs specific) | "Asks narrow, pointed questions; prefers a direct answer over exploration" |
| Language mixing | Code-switching between natural languages | "English for all technical content; occasionally uses German for informal remarks" |

### 2.2 Technical Preferences

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Language/framework choices | What they choose when multiple options exist | "Prefers Rust for host tools, Python for scripting, avoids Go" |
| Code patterns | Corrections to generated code, stated preferences | "Always uses Result types; avoids unwrap() in non-test code" |
| Naming conventions | Corrections to names, style of their own code | "snake_case for everything except type names; descriptive but not verbose" |
| Tool preferences | Which tools they reach for first | "argparse over click; prefers shell pipelines for one-off data tasks" |
| Architecture style | Stated preferences, accepted/rejected designs | "Favors flat module structures; pushes back on deep nesting" |
| Error handling | Corrections and stated expectations | "Wants explicit error propagation; dislikes silent failures" |
| Dependency attitude | Eagerness to add vs avoid external deps | "Conservative; prefers stdlib solutions; needs justification for new deps" |

### 2.3 Decision Patterns

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Decision speed | Time/messages between question and decision | "Decides quickly on tactical choices; deliberates on architecture" |
| Risk tolerance | Choice between safe/boring and novel/experimental | "Conservative for production code; experimental in dev tooling" |
| Evidence needs | Whether they ask for data, benchmarks, comparisons | "Wants to see alternatives before committing; asks 'what are the tradeoffs?'" |
| Reversibility preference | Whether they favor reversible decisions | "Prefers reversible approaches; explicitly asks 'can we undo this?'" |
| Delegation style | How much autonomy they grant the agent | "Delegates execution freely but wants to approve design decisions" |

### 2.4 Workflow Patterns

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Session length | Typical duration of work sessions | "Typical session: 1-3 hours; prefers focused sprints" |
| Task granularity | Size of tasks they assign per message | "Assigns one well-scoped task per message; rarely batches" |
| Parallelization | Whether they run multiple agent sessions | "Single-threaded; works one task to completion before starting the next" |
| Review depth | How carefully they inspect agent output | "Reviews generated code line-by-line; reads diffs before committing" |
| Commit rhythm | When and how often they want commits | "Wants commits after each logical unit; dislikes large accumulated changes" |
| Testing expectations | When they want tests run, what coverage they expect | "Expects tests to pass before any commit; wants clippy clean at all times" |

### 2.5 Domain Expertise Map

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Correction areas | Topics where owner corrects the agent | "Deep Rust knowledge -- corrects lifetime annotations, borrow checker suggestions" |
| Deferral areas | Topics where owner accepts agent suggestions without pushback | "Defers on CSS/frontend styling decisions" |
| Explanation requests | What they ask the agent to explain | "Asks about advanced Docker multi-stage patterns -- learning this area" |
| Teaching moments | When they explain something to the agent | "Explains containerization nuances unprompted -- core expertise" |

### 2.6 Feedback Patterns

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Approval signals | What earns positive feedback | "Approves concise, working solutions; praises when agent catches edge cases" |
| Correction triggers | What prompts corrections or pushback | "Corrects over-engineering, unnecessary abstractions, verbose output" |
| Recurring instructions | Instructions repeated across sessions | "Frequently says 'be concise', 'don't summarize what I already know'" |
| Pet peeves | Things that consistently trigger negative reactions | "Dislikes: emoji in code/docs, unnecessary pleasantries, restating the obvious" |

---

## 3. Proposed OWNER.md Structure

### 3.1 Current Format

The current OWNER.md is a simple, manually written profile:

```markdown
# Owner Profile

- **Name:** ...
- **Role:** ...
- **Contact:** ...

## Background
(static facts)

## Preferences
(manually stated preferences)

## Working Context
(timezone, current focus)
```

This is good as a foundation. The proposal extends it without breaking the existing structure.

### 3.2 Proposed Extended Format

```markdown
# Owner Profile

- **Name:** Bernhard (bnaard)
- **Role:** Founder at projectious.work, AI consulting
- **Contact:** GitHub @bnaard

## Background

- **Domain expertise:** AI consulting, DevOps, containerized development environments
- **Primary languages:** Python, Rust, shell scripting
- **Years of experience:** Senior engineer, 10+ years

## Preferences

- **Communication style:** Concise and direct, pragmatic over theoretical
- **Communication language:** English (code and docs), German native speaker
- **Code style preferences:** argparse not click, dataclass-based config, Rust for host tools
- **Review preferences:** Always reference GitHub issue numbers in commits

## Working Context

- **Timezone:** Europe/Berlin
- **Current focus:** aibox CLI evolution

---

## Observed Patterns

<!-- This section is maintained by the owner-profiling skill.
     Entries include confidence levels and last-confirmed dates.
     The owner can edit or delete any observation at any time. -->

### Communication

- Prefers terse instructions; rarely more than 3 sentences per task (high confidence, 2026-03-20)
- Direct style; no pleasantries in task context (high confidence, 2026-03-18)

### Technical

- Always wants clippy clean before commits (high confidence, 2026-03-22)
- Prefers flat module structures over deep nesting (medium confidence, 2026-03-15)

### Decisions

- Decides tactical choices quickly; deliberates on architecture (medium confidence, 2026-03-19)
- Conservative on adding new dependencies (high confidence, 2026-03-22)

### Workflow

- Works in focused 1-3 hour sessions (medium confidence, 2026-03-20)
- Expects one commit per logical unit of work (high confidence, 2026-03-21)

### Expertise

- Deep: Rust, containers, DevOps, CLI design (high confidence)
- Learning: frontend patterns, CSS (low confidence, 2026-03-10)

### Feedback

- Corrects verbose output and over-engineering (high confidence, 2026-03-22)
- Approves concise solutions that handle edge cases (medium confidence, 2026-03-18)
```

### 3.3 Design Principles for the Observed Patterns Section

1. **Additive only** -- new observations append or refine existing entries; the skill never deletes manually written content in the upper sections.
2. **Confidence levels** -- `low` (observed 1-2 times), `medium` (3-5 times), `high` (6+ times or explicitly confirmed by owner).
3. **Date stamps** -- each observation records the date it was last confirmed or updated.
4. **Human-readable** -- plain markdown bullets, no structured data formats. The file must remain useful to a human reader.
5. **Owner sovereignty** -- the owner can edit, delete, or override any observation. Manually written entries in the upper sections always take precedence.

---

## 4. Privacy and Consent

### 4.1 Opt-in Mechanism

Owner profiling must be explicitly enabled. Proposed mechanism:

```toml
# aibox.toml
[owner]
profiling = true    # default: false
```

When `profiling = false` (or absent), the skill is inactive. Agents read OWNER.md but never write to the "Observed Patterns" section.

First-run behavior: when an agent detects that OWNER.md exists but has no "Observed Patterns" section and profiling is enabled, it asks once:

> "I noticed owner profiling is enabled. I'll periodically note patterns in your working style in OWNER.md so future sessions can serve you better. You can review and edit these observations anytime, or disable profiling in aibox.toml. Proceed?"

### 4.2 Transparency Rules

- The agent must announce when it writes to OWNER.md: "I added an observation to your owner profile: [brief description]."
- The agent must not silently update the file.
- On request, the agent summarizes all current observations.

### 4.3 What Must Never Be Recorded

- Personal information unrelated to work (health, family, finances)
- Authentication credentials, API keys, passwords
- Private opinions about other people or organizations
- Information shared in confidence ("don't put this anywhere")
- Behavioral inferences about emotional state or personality traits beyond work style

### 4.4 Owner Controls

- `aibox owner show` -- display current profile including observed patterns
- `aibox owner clear` -- remove all observed patterns, keep manual sections
- `aibox owner disable` -- set `profiling = false` in aibox.toml
- Direct file editing -- the owner can always edit OWNER.md directly

---

## 5. Relationship to Existing Systems

### 5.1 Claude Code Memory

Claude Code maintains per-user memory across sessions. This is ephemeral, model-managed, and not visible in the project repository.

| Aspect | Claude Code Memory | OWNER.md |
|--------|-------------------|-----------|
| Scope | Per-user, cross-project | Per-project, single owner |
| Persistence | Model-managed, opaque | File in repo, version-controlled |
| Visibility | Only to the model | To all agents and humans |
| Portability | Tied to Claude Code | Portable across any AI tool |
| Editability | No direct editing | Full owner control |

### 5.2 Complementary Roles

- **Claude Code memory** handles ephemeral, session-to-session continuity: "last time we were working on X", "user prefers dark mode in their editor."
- **OWNER.md** captures durable, project-relevant preferences that any agent (not just Claude Code) should know: coding conventions, decision style, expertise map.
- **CLAUDE.md** contains project-level instructions that apply regardless of who the owner is. OWNER.md is personal; CLAUDE.md is structural.

### 5.3 Interaction Pattern

```
Session starts
  |
  v
Agent reads CLAUDE.md (project rules)
  |
  v
Agent reads OWNER.md (owner profile + observed patterns)
  |
  v
Agent applies both during the session
  |
  v
Session ends -- agent reflects on new observations
  |
  v
Agent updates OWNER.md "Observed Patterns" section (if profiling enabled)
```

---

## 6. Implementation Options

### Option A: Pure Skill (Recommended for Phase 1)

A SKILL.md file that instructs agents to self-reflect periodically and update OWNER.md.

**How it works:**
- The skill triggers at natural breakpoints: session end, after significant feedback, when the owner corrects a pattern.
- The agent reviews the session for patterns, compares against existing OWNER.md observations, and proposes updates.
- Updates are shown to the owner before writing.

**Pros:**
- Zero CLI changes required
- Works immediately with any agent that reads skills
- Simple to iterate on instructions
- No new infrastructure

**Cons:**
- Relies on the agent following instructions consistently
- No automated trigger -- depends on agent judgment for timing
- Different agents may observe/record differently

### Option B: Skill + Post-Session Hook

The session-handover skill already runs at session end. Add a profiling reflection step to the handover flow.

**How it works:**
- After generating the session handover note, the agent runs a second reflection pass focused on owner patterns.
- Integrates naturally into the existing end-of-session ritual.

**Pros:**
- Consistent timing (every session end)
- Pairs naturally with session-handover skill
- Single trigger point

**Cons:**
- Couples two independent skills
- If the owner skips handover, profiling also skips
- Handover is already a multi-step process; adding more may feel heavy

### Option C: Skill + CLI Command

`aibox owner update` triggers a guided reflection where the agent reviews recent sessions and updates the profile.

**How it works:**
- CLI command reads recent session handover notes and git history.
- Presents a structured reflection prompt to the agent.
- Agent proposes updates; owner approves.

**Pros:**
- Owner-initiated, maximum consent
- Can batch-process multiple sessions
- Clear separation from normal workflow

**Cons:**
- Requires CLI implementation (Rust code)
- Owner must remember to run it
- Loses real-time observation advantage

### Option D: Automatic via Claude Code Memory Bridge

Periodically sync relevant Claude Code memory observations into OWNER.md.

**How it works:**
- A background process or periodic skill reads the agent's memory/context for owner-relevant patterns.
- Writes structured observations to OWNER.md.

**Pros:**
- Leverages existing memory system
- Could be very accurate (model already tracks this)

**Cons:**
- Claude Code memory is opaque and not API-accessible
- Tightly couples to one agent implementation
- Privacy concerns around automated extraction

---

## 7. Recommendation

**Phase 1: Option A (Pure Skill)**

Start with a pure skill. It requires no CLI changes, ships as a standard skill file, and can be iterated quickly by editing the SKILL.md. The skill should:

1. Trigger at session end (or when the owner says "update my profile").
2. Review the session for new or refined observations.
3. Show proposed updates to the owner before writing.
4. Write to the "Observed Patterns" section of OWNER.md.
5. Never modify the manually written upper sections.

**Phase 2: Integration with Session Handover**

Once the pure skill proves useful, add an optional profiling step to the session-handover flow. This is a one-line addition to the handover skill: "If owner profiling is enabled, also run the owner-profiling skill."

**Phase 3: CLI Support (if warranted)**

If owners want batch processing or explicit control, add `aibox owner update` and `aibox owner show` commands. This is only worth building if the skill-based approach shows clear demand.

---

## 8. Draft Skill: owner-profiling

```markdown
---
name: owner-profiling
description: Observes the project owner's working style, preferences, and decision patterns during sessions and records findings in context/OWNER.md. Trigger at session end, after significant owner feedback, or when the owner says "update my profile".
---

# Owner Profiling

## When to Use

Trigger this skill when:
- The owner says "update my profile", "what have you learned about me", or "owner profile"
- A session is ending (after session-handover, if that skill is active)
- The owner gives significant feedback that reveals a preference pattern (correction, repeated instruction, explicit preference statement)

Do NOT trigger this skill if owner profiling is disabled. Check:
- If `aibox.toml` contains `[owner] profiling = false`, do not run.
- If OWNER.md has no "Observed Patterns" section and no profiling setting exists, ask the owner before proceeding.

## Instructions

### 1. Read the Current Profile

Read `context/OWNER.md` in full. Note:
- The manually written sections (everything above the `---` / "Observed Patterns" heading) are owner-authored and must not be modified.
- The "Observed Patterns" section is skill-managed. If it does not exist yet, you will create it.

### 2. Reflect on the Session

Review the current session for signals in these categories:

**Communication:** Message length, formality, jargon level, question style.
**Technical:** Language/tool choices, code patterns, naming, error handling, dependency attitude.
**Decisions:** Speed, risk tolerance, evidence needs, delegation style.
**Workflow:** Session length, task granularity, commit rhythm, testing expectations.
**Expertise:** Where the owner corrected you (deep knowledge), where they deferred (less knowledge).
**Feedback:** What triggered corrections, what earned approval, repeated instructions.

For each category, ask yourself:
- Did I observe something new that is not already in OWNER.md?
- Did I observe something that confirms an existing observation (raise confidence)?
- Did I observe something that contradicts an existing observation (flag for review)?

### 3. Propose Updates

Present your proposed updates to the owner before writing. Format:

```
I noticed some patterns this session. Proposed updates to your owner profile:

**New observations:**
- [category] observation text (confidence: low/medium/high)

**Confidence updates:**
- [existing observation] -- seen again, raising from medium to high

**Potential conflicts:**
- [existing observation] vs [new observation] -- please clarify
```

Wait for the owner to approve, modify, or reject each observation.

### 4. Write Updates

After owner approval, update the "Observed Patterns" section of `context/OWNER.md`.

Rules:
- Add new observations as bullet points under the appropriate category heading.
- Each bullet ends with `(confidence: low/medium/high, YYYY-MM-DD)`.
- If an existing observation is confirmed, update its confidence level and date.
- If an existing observation is contradicted and the owner clarifies, replace it.
- Never modify content above the "Observed Patterns" section.
- Keep observations concise: one line per pattern, no paragraphs.

If the "Observed Patterns" section does not exist, create it after a `---` separator with this structure:

```
---

## Observed Patterns

<!-- Maintained by the owner-profiling skill. Edit or delete any entry freely. -->

### Communication

### Technical

### Decisions

### Workflow

### Expertise

### Feedback
```

### 5. Announce the Update

After writing, confirm what was added:

> "Updated your owner profile with N new observations and M confidence updates."

## What NOT to Record

Never record:
- Personal information unrelated to work (health, family, finances)
- Credentials, API keys, or secrets
- Opinions about other people
- Information the owner shared in confidence
- Emotional state or personality inferences
- Anything the owner asks you not to record

## Confidence Levels

- **Low:** Observed 1-2 times. May be situational rather than a pattern.
- **Medium:** Observed 3-5 times. Likely a genuine preference.
- **High:** Observed 6+ times, or explicitly confirmed by the owner.

When an observation has been `low` for 3+ sessions without reconfirmation, consider removing it (ask first).

## Example

After a session where the owner repeatedly asked for shorter explanations, corrected a `click`-based CLI to use `argparse`, and approved a flat module layout:

```
I noticed some patterns this session. Proposed updates to your owner profile:

**New observations:**
- [Feedback] Prefers short explanations; pushed back on verbose output twice (confidence: low)

**Confidence updates:**
- [Technical] "argparse over click" -- seen again, already high confidence, updating date

**No conflicts detected.**
```

After approval, OWNER.md gains:

```markdown
### Feedback

- Prefers short explanations; pushes back on verbose output (low, 2026-03-26)
```

And the existing argparse entry gets its date updated.
```

---

## 9. Open Questions

1. **Multi-owner projects:** The current design assumes a single owner. If a project has multiple contributors, should there be per-person profiles (e.g., `context/owners/alice.md`)? Deferred to Phase 2.

2. **Observation decay:** Should old observations with low confidence be automatically pruned? Current proposal: agent suggests removal after 3 sessions without reconfirmation, but never deletes without asking.

3. **Cross-project learning:** If the same owner uses aibox across multiple projects, should observations transfer? This could be handled by a shared OWNER.md template or a global `~/.aibox/owner.md`, but adds complexity. Deferred.

4. **Skill interaction:** Should other skills read OWNER.md observations to adjust their behavior? For example, `session-handover` could adjust verbosity based on the owner's communication style preference. This happens naturally if the agent reads OWNER.md at session start, but explicit cross-skill references could make it more reliable.

5. **Granularity of consent:** The current design is all-or-nothing (profiling enabled/disabled). Should owners be able to opt into specific categories (e.g., "track my technical preferences but not my communication style")? Probably over-engineering for Phase 1.

---

## 10. Summary

| Aspect | Decision |
|--------|----------|
| Implementation | Pure skill (SKILL.md), no CLI changes |
| Storage | "Observed Patterns" section in existing OWNER.md |
| Consent | Explicit opt-in via `aibox.toml` setting |
| Trigger | Session end, owner request, or significant feedback |
| Confidence | Three levels (low/medium/high) with date stamps |
| Privacy | Strict exclusion list; owner can edit/delete anytime |
| Phase 1 scope | Ship the skill, test with real usage, iterate on instructions |
