# Observable signals — what to watch for in normal sessions

This is the reference table for **observe mode**. The agent watches for
these patterns during normal work and proposes additions to the owner
profile when evidence accrues. The agent should never edit the profile
silently — see the discipline rules in `interview-protocol.md`.

## Threshold for surfacing

Don't surface a single observation. Surface only when the same signal
appears at least:

- **3 times in the same session** (strong signal in one context), OR
- **3+ sessions** (consistent pattern across time)

When the threshold is met, propose the addition to the owner:

> "I've noticed you consistently TODO. Want me to add this to your
> working-style.md as a preference?"

Wait for explicit approval before editing. Show the proposed diff.

## Communication signals (target file: working-style.md)

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Message length | Average length of owner messages over a session | "Prefers terse messages; typically 1-3 sentences per instruction" |
| Formality level | Presence of greetings, please/thanks, full sentences vs fragments | "Direct and informal; rarely uses pleasantries in task instructions" |
| Technical jargon density | Domain-specific terms without explanation | "Uses Rust ecosystem terms freely (lifetimes, borrowing, trait objects) — no need to explain" |
| Emoji usage | Frequency and context of emoji | "No emoji in technical discussions; occasional thumbs-up for approval" |
| Question patterns | How they ask for information (open-ended vs specific) | "Asks narrow, pointed questions; prefers a direct answer over exploration" |
| Language mixing | Code-switching between natural languages | "English for all technical content; occasionally uses German for informal remarks" |
| Disliked phrases | Reactions to specific AI-generated phrasings | "Strongly dislikes 'delve into', 'tapestry', 'in the realm of', 'cutting-edge'" |
| Preferred formatting | Tables vs lists vs prose, headers vs no headers | "Prefers tables for comparisons, no headers in short responses" |

## Technical preferences (target file: working-style.md or goals-and-context.md)

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Language/framework choices | What they choose when multiple options exist | "Prefers Rust for host tools, Python for scripting, avoids Go" |
| Code patterns | Corrections to generated code, stated preferences | "Always uses Result types; avoids unwrap() in non-test code" |
| Naming conventions | Corrections to names, style of their own code | "snake_case for everything except type names; descriptive but not verbose" |
| Tool preferences | Which tools they reach for first | "argparse over click; prefers shell pipelines for one-off data tasks" |
| Architecture style | Stated preferences, accepted/rejected designs | "Favors flat module structures; pushes back on deep nesting" |
| Error handling | Corrections and stated expectations | "Wants explicit error propagation; dislikes silent failures" |
| Dependency attitude | Eagerness to add vs avoid external deps | "Conservative; prefers stdlib solutions; needs justification for new deps" |
| Test style | Test names, structure, coverage expectations | "Prefers descriptive test names; one assertion per test where possible" |

## Decision patterns (target file: working-style.md)

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Decision speed | Time/messages between question and decision | "Decides quickly on tactical choices; deliberates on architecture" |
| Risk tolerance | Choice between safe/boring and novel/experimental | "Conservative for production code; experimental in dev tooling" |
| Evidence needs | Whether they ask for data, benchmarks, comparisons | "Wants to see alternatives before committing; asks 'what are the tradeoffs?'" |
| Reversibility preference | Whether they favor reversible decisions | "Prefers reversible approaches; explicitly asks 'can we undo this?'" |
| Delegation style | How much autonomy they grant the agent | "Delegates execution freely but wants to approve design decisions" |
| Approval cadence | When they want to be asked vs left alone | "Wants approval before major refactors; routine edits proceed without asking" |

## Workflow patterns (target file: working-style.md)

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Session length | Typical duration of work sessions | "Typical session: 1-3 hours; prefers focused sprints" |
| Task granularity | Size of tasks they assign per message | "Assigns one well-scoped task per message; rarely batches" |
| Parallelization | Whether they run multiple agent sessions | "Single-threaded; works one task to completion before starting the next" |
| Review depth | How carefully they inspect agent output | "Reviews generated code line-by-line; reads diffs before committing" |
| Feedback cadence | When and how they correct course | "Course-corrects mid-task; prefers being interrupted over silent drift" |
| Working hours | Time-of-day patterns | "Mostly works mornings UTC+1; evening sessions are short and focused" |
| Context-switch tolerance | Reaction to changing tasks mid-session | "Dislikes context switches; finishes one thing before pivoting" |

## Goals and priorities (target file: goals-and-context.md)

| Signal | How to observe | Example recorded preference |
|--------|---------------|-----------------------------|
| Stated current focus | "I'm working on...", "right now I need..." | "Current focus: ship aibox 0.15 with processkit consumption" |
| Tradeoff statements | "I'd rather X than Y" | "Will trade speed for correctness; will trade features for simplicity" |
| Things explicitly deprioritized | "Not now, later", "out of scope" | "Multi-cloud abstraction is out of scope for v1" |
| Domain depth | Confidence and specificity in a given area | "Deep in Rust async; moderate in WebAssembly; beginner in WebRTC" |

## Team and relationships (target file: team-and-relationships.md)

These signals are usually only observed when the agent helps prep
communication to/about a specific person. Be especially careful here —
these notes are user-private and should never end up checked into git.

| Signal | How to observe | Example recorded note |
|--------|---------------|-----------------------------|
| Communication style toward person X | How owner phrases messages to/about X | "Alice prefers terse Slack DMs; she finds long threads overwhelming" |
| Sensitivities | Topics owner avoids with X | "Don't mention the Q3 incident in messages to Bob — it's a sore point" |
| Mutual dependencies | What X needs from owner, what owner needs from X | "Carol needs my PR reviews within 24h; I need her DB migration approval before deploy" |
| Cadence | How often the owner interacts with X | "1:1 with Diana every other Friday at 14:00 UTC+1" |

## What NOT to observe

The agent should NOT:
- Track private health, family, or personal-life information that isn't
  directly relevant to working style
- Track financial information unless the owner explicitly asks
- Track political or ideological positions
- Surface observations about people who aren't part of the project
- Surface observations from a single session as if they were consistent
  patterns

When in doubt, don't add it.
