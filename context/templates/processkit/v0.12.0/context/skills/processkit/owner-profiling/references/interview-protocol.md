# Interview protocol — owner profile bootstrap

This is the per-file question list the agent uses in **bootstrap mode** (when
the owner profile doesn't exist yet) and **refine mode** (when targeting one
file). Discipline rules below apply in both cases.

## Discipline rules

1. **Sequential, not batched.** Ask questions one at a time. Wait for the
   answer before asking the next. Never dump a list of 6 questions at once.
2. **Stop when you have enough.** Most files need 3-4 questions before you
   can draft. Don't pad.
3. **Draft, then validate.** When you have enough, write a draft and show
   it to the owner. Ask "what doesn't sound right?" Revise.
4. **Don't pad.** Files should be short and dense. Resist the urge to fill
   sections with generic content.
5. **Honor pauses.** If the owner says "skip that" or "I don't know yet",
   leave the section blank with a `TODO` marker. Come back later.
6. **Never edit silently.** Even after approval, show the diff before
   saving. The owner is always in the loop.

## Recommended order

The four files take roughly 40 minutes total but should NOT be done in one
sitting unless the owner is fresh and asking for it. Suggested ordering:

1. **identity.md** (~10 min) — quick wins, immediately useful
2. **working-style.md** (~15 min) — the highest-leverage file
3. **goals-and-context.md** (~10 min)
4. **team-and-relationships.md** (~5 min/person, optional — many users skip this initially)

It is fine to do (1) in the first session, (2) in the next, and so on.

## Per-file questions

### identity.md

1. What's your name and current role or title?
2. What organization or company are you with, if any?
3. If you had to explain what you actually do to someone at a dinner party
   — not your title, but what you spend your time on — what would you say?
4. What do people come to you for? What's the thing where someone says
   "you should talk to [your name] about that"?

**Stop after Q3 or Q4.** Draft using the identity.md template.

### working-style.md

1. **Communication length.** When you ask me to do something, do you tend
   to write a sentence, a paragraph, or a longer brief? When I respond, what
   length feels right — short summary, medium with bullets, or detailed?
2. **Communication tone.** Formal or informal? Are pleasantries (please,
   thanks, hi) expected, neutral, or unwelcome?
3. **Vocabulary you avoid.** Are there words or phrases you find sycophantic,
   fluffy, or hate seeing in AI output? Things like "delve into", "tapestry",
   "in the realm of"?
4. **Hard rules.** What are the absolute non-negotiables? Time zones,
   working hours, languages, tools you refuse to use, dependencies you
   refuse to add?
5. **Strong preferences.** What do you usually want but could be flexible
   on? Naming conventions, commit message style, formatting habits?
6. **Things you visceral hate.** What's the stuff that makes you sigh
   when you see it in agent output?
7. **Weekly rhythm.** What does a typical work week look like for you?
   Recurring meetings, deep-work blocks, cadences?
8. **Decision authority.** What do you decide alone? What do you delegate?
   What do you consult others on?

**Stop after 5-6 questions.** Working-style is medium-length; it's worth
investing more time here than in any other file.

### goals-and-context.md

1. **Current goals.** What are you trying to accomplish in the next quarter
   or season of work? Concrete outcomes, not aspirations.
2. **Longer-term goals.** This year and beyond. Where are you heading?
3. **Common tradeoffs.** When forced to choose between speed and quality,
   breadth and depth, reversible and bold — where do you usually land?
4. **What you're NOT prioritizing right now.** Things that are important
   but on the back burner, so I stop suggesting them.
5. **Areas of expertise.** What do you know deeply? Where can I operate at
   your level without explaining basics?
6. **Where you're a beginner.** What topics would you appreciate more
   explanation on, not less? Where do you want me to teach?

**Stop after 4-5 questions.** Some sections will be empty initially —
that's fine. Come back when the goals firm up.

### team-and-relationships.md

For each person (5-8 total):

1. Their name and role.
2. Your relationship type (manager, direct report, peer, client, collaborator).
3. How you interact — cadence, channel, formal/informal.
4. What they need from you.
5. What you need from them.
6. Anything an agent should know when preparing communication to/about
   them — communication style, sensitivities, working patterns.

**Stop after 6 questions per person.** Many users skip this file initially
and add a few people over time as they encounter agent-prep needs ("draft
me a response to TODO" → "wait, I should record what TODO needs from me").

## Validation question (run after each file)

After you draft a file, ask the owner:

> "Anything in this draft that doesn't sound right or feels off? Anything
> that's missing? Anything that's too padded?"

Revise based on the answer. Repeat until the owner says "looks right".
Then save.

## What to do when you don't have enough information

If the owner gives a one-word answer or says "I don't know", do NOT pad
with generic content. Leave the section as `TODO` with the question
visible:

```markdown
## What I'm Known For

TODO — answer pending. Question: "What do people come to you for?"
```

The next session can revisit. An incomplete profile is better than a
made-up one.
