# Agent Coordination Patterns Reference

Patterns for orchestrating multiple AI agents in Claude Code and similar tool-using environments.

## Patterns

### 1. Pipeline

Agents execute sequentially, each consuming the previous agent's output.

```
Researcher -> Planner -> Coder -> Reviewer
```

- **Use when:** Tasks have a natural order (understand, plan, build, verify)
- **Context passing:** Each stage writes a structured output file; next stage reads it
- **Failure mode:** If any stage fails, retry that stage or back up one stage
- **Best for:** Feature implementation, bug investigation and fix, migration tasks

### 2. Fan-Out / Fan-In

A planner decomposes work into independent subtasks, multiple agents execute in parallel, a coordinator merges results.

```
Planner -> [Agent A, Agent B, Agent C] -> Coordinator
```

- **Use when:** Subtasks are independent (separate files, separate modules, separate concerns)
- **Context passing:** Planner writes a task spec per subtask; coordinator reads all outputs
- **Failure mode:** Failed subtasks retry independently; coordinator proceeds with partial results if acceptable
- **Best for:** Multi-file refactoring, documentation across modules, test coverage expansion
- **Caution:** In Claude Code, true parallelism means separate agent sessions -- ensure no file conflicts

### 3. Supervisor

One agent continuously monitors and directs other agents, reassigning work based on progress and results.

```
Supervisor <-> [Worker A, Worker B]
Supervisor reviews outputs, redirects as needed
```

- **Use when:** Tasks are complex, requirements may shift based on intermediate findings
- **Context passing:** Workers report back to supervisor after each step; supervisor maintains the overall plan
- **Failure mode:** Supervisor detects stalls or errors and reassigns or adjusts the plan
- **Best for:** Large refactors, exploratory tasks where the path is unclear upfront
- **In practice:** In Claude Code, a "supervisor" is typically the main agent session that spawns sub-tasks

### 4. Consensus

Multiple agents independently solve the same problem, then results are compared to select the best or synthesize a combined answer.

```
[Agent A, Agent B, Agent C] -> Evaluator -> Final Answer
```

- **Use when:** The problem has multiple valid approaches and you want to explore the solution space
- **Context passing:** Each agent gets the same input; evaluator reads all outputs and selects or merges
- **Failure mode:** If all agents fail, reframe the problem; if one succeeds, use that output
- **Best for:** Architecture decisions, complex algorithm design, security review (multiple perspectives)
- **Cost:** Expensive -- multiplies token usage. Only use for high-stakes decisions.

### 5. Blackboard

Agents share a common workspace (a file or set of files) that any agent can read and write. Agents pick up work items and post results.

```
[Agent A, Agent B] <-> Shared Workspace (blackboard file)
```

- **Use when:** Tasks are loosely coupled but share state (e.g., building a document, populating a data structure)
- **Context passing:** A single file (e.g., `_blackboard.md`) with structured sections per topic
- **Failure mode:** Agents may overwrite each other -- use section-based ownership to prevent conflicts
- **Best for:** Collaborative document creation, knowledge base building, incremental investigation
- **In practice:** Works well in Claude Code when agents take turns (no true concurrency)

---

## Anti-Patterns

### Context Overload

**Problem:** Passing the entire codebase or conversation history to every agent.
**Symptom:** Agents produce vague or off-target responses; token limits hit.
**Fix:** Give each agent only the files and context it needs for its specific subtask. Use file paths and line ranges, not full file contents, in handoff messages.

### Circular Delegation

**Problem:** Agent A asks Agent B, which asks Agent A, creating an infinite loop.
**Symptom:** No progress, repeated handoffs, growing context with no new artifacts.
**Fix:** Enforce a strict pipeline direction. Every handoff must move the task forward. If an agent cannot proceed, it escalates to the user -- not back to the previous agent.

### No Verification

**Problem:** Skipping the review step because "the coder agent is good enough."
**Symptom:** Bugs, incomplete implementations, style violations, missed requirements.
**Fix:** Always include a review gate. At minimum: run tests, run linter, check the output against the original spec. Self-review (same agent, reviewer role) is better than no review.

### Premature Parallelism

**Problem:** Fan-out before understanding task dependencies, causing merge conflicts or duplicated work.
**Symptom:** Agents modify the same files, produce conflicting changes, or redo each other's work.
**Fix:** Always run a planning stage first. Only fan out when subtasks are provably independent (different files, different modules). When in doubt, use a pipeline.

### Vague Handoffs

**Problem:** Handoff says "I made some changes, please review" without specifics.
**Symptom:** Receiving agent wastes time re-discovering what was done.
**Fix:** Use the structured handoff format: status, file list, verification steps taken, blockers, and explicit next action.

---

## Pattern Decision Matrix

Choose a pattern based on task characteristics:

| Characteristic | Pipeline | Fan-Out | Supervisor | Consensus | Blackboard |
|---|---|---|---|---|---|
| Sequential dependencies | Yes | No | Partial | No | Partial |
| Independent subtasks | No | Yes | Yes | Yes | Yes |
| Uncertain requirements | No | No | Yes | Partial | Partial |
| Multiple valid solutions | No | No | No | Yes | No |
| Shared evolving state | No | No | No | No | Yes |
| Low token budget | Yes | No | Yes | No | Yes |
| High-stakes decision | No | No | Partial | Yes | No |

**Default choice:** Start with Pipeline. It is the simplest, lowest-risk pattern. Escalate to Supervisor if the task scope shifts during execution. Use Fan-Out only when the planner confirms subtasks are independent. Reserve Consensus for decisions with significant long-term impact.

## Practical Tips for Claude Code

1. **One agent session = one role at a time.** Switch roles explicitly ("Now acting as Reviewer").
2. **Use files for context, not memory.** Write `_handoff.md` or `_findings.md`; read them in the next step.
3. **Keep scratch files temporary.** Clean up `_agent_notes.md`, `_blackboard.md` after workflow completion.
4. **Verify with tools, not judgment.** Run `cargo test`, `npm test`, or the appropriate linter instead of "I think this looks right."
5. **Time-box research.** Researchers should answer specific questions, not "tell me everything about X."
6. **Limit fan-out to 3-4 parallel subtasks.** More than that and coordination overhead exceeds the benefit.
7. **Always include file paths.** Every handoff and finding should reference absolute paths and line numbers.
