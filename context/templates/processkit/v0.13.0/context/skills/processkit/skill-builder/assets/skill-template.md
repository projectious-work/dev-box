---
name: TODO-skill-name
description: |
  TODO: One paragraph describing what the skill does (the "what"). Use
  when the user says "TODO: trigger phrase 1", "TODO: trigger phrase 2",
  or "TODO: trigger phrase 3" (the "when"). Keep under 1024 characters
  total. NO angle brackets allowed in this field.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-TODO-skill-name
    version: "1.0.0"
    created: TODO-YYYY-MM-DDTHH:MM:SSZ
    category: TODO  # process | language | framework | infrastructure | architecture | design | data | ai | api | security | observability | database | performance | meta
    layer: null  # 0-4 for process skills only; null for everything else
    # uses:
    #   - skill: index-management
    #     purpose: "TODO: explain why this skill uses index-management."
    # provides:
    #   primitives: []
    #   mcp_tools: []
    #   assets: []
    #   processes: []
---

# TODO Skill Title

## Intro

TODO: 1-3 sentences. What this skill is and when it applies. Just enough
for the agent to decide whether the skill is relevant to the current task.
Cut everything that doesn't help that decision.

## Overview

TODO: Key concepts, the main workflow, the most common operations.
Enough for the agent to act in typical cases without consulting Full
reference. Use tables, numbered steps, and code blocks freely. Be
specific.

### TODO subsection 1

TODO: ...

### TODO subsection 2

TODO: ...

## Gotchas

REQUIRED. Agent-specific failure modes — the things models get wrong
that humans wouldn't. Provider-neutral (do not assume the agent is
Claude). Each item must name a specific failure AND a specific
countermeasure. 5-10 items is the right size.

- **TODO failure mode 1.** TODO: explanation and what to do instead.
- **TODO failure mode 2.** TODO: explanation and what to do instead.
- **TODO failure mode 3.** TODO: explanation and what to do instead.
- **TODO failure mode 4.** TODO: explanation and what to do instead.
- **TODO failure mode 5.** TODO: explanation and what to do instead.

(See also "Anti-patterns" under Full reference for general
practitioner pitfalls that apply to humans and agents alike.)

## Full reference

### TODO deep-dive section 1

TODO: Edge cases, complete field specs, rare workflows, error modes.
This section can be long. If it grows past ~1000 words, push detail to
`references/<topic>.md` and link to it from here.

### TODO deep-dive section 2

TODO: ...

### Anti-patterns

TODO: General practitioner anti-patterns that apply to humans AND
agents alike. (Agent-specific failure modes go in Gotchas, not here.)

- **TODO anti-pattern 1.** TODO.
- **TODO anti-pattern 2.** TODO.

### Cross-references

TODO:
- Other skills this one composes with
- The relevant primitive schemas under `src/primitives/schemas/`
- Any external documentation links
