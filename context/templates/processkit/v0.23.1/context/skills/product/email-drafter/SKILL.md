---
name: email-drafter
description: |
  Drafts clear, professional emails ready to copy and paste — from a brief description of the situation, audience, and intent. Use when asked to write, draft, or compose an email, reply to a message, or suggest how to phrase something professionally.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-email-drafter
    version: "1.0.0"
    created: 2026-04-08T00:00:00Z
    category: product
    layer: 2
---

# Email Drafter

## Intro

A good email delivers one clear message, respects the reader's time,
and makes the desired next action obvious. Before writing, identify
the situation, the audience's relationship and context, and the single
thing you need the recipient to do or understand. Then write the
shortest email that achieves that goal.

## Overview

### Before you write — gather the inputs

Ask for (or infer) these before drafting:

1. **Situation** — what happened or what is needed
2. **Recipient** — their role, relationship (colleague, client, manager,
   stranger), and level of context about the topic
3. **Intent** — the one thing the email must accomplish (inform, request,
   apologize, confirm, decline, follow up)
4. **Tone** — formal, professional-but-warm, casual, urgent
5. **Any constraints** — deadline, prior emails in the thread, sensitive
   content to avoid

If the user has given enough context, draft immediately. Do not ask
for information that can be reasonably inferred.

### Email types and their patterns

**Request**
> Subject: [Action needed] — [Topic] by [Date]  
> Open with context (1 sentence). State the ask directly. Give
> rationale in 1-2 sentences. Specify the deadline and format of
> response needed. Close with thanks or next step.

**Status update**
> Subject: [Project] update — [Date/Phase]  
> Open with the one-line headline (good news, concern, or neutral).
> Bullet the key points (3-5 max). State what happens next and who
> owns it. Close with "let me know if you have questions."

**Follow-up / nudge**
> Subject: Re: [original subject]  
> Acknowledge the prior exchange in one line. Restate the ask
> briefly. If there is a deadline, name it. Keep the tone light
> unless the situation is urgent.

**Apology / acknowledgment**
> Subject: [Topic] — apology / correction  
> Lead with the apology, not context. Be specific about what went
> wrong. State what has changed or will change. No overexplaining.

**Decline / boundary**
> Subject: Re: [original subject]  
> Acknowledge the ask. State the decline directly. Give a brief,
> honest reason (optional). Offer an alternative if one exists.
> Close warmly.

**Announcement**
> Subject: [What's changing] — effective [Date]  
> Open with the what and when. Explain why in 2-3 sentences. State
> what the recipient needs to do (if anything). Include a contact
> for questions.

### Subject line rules

- Start with the type if actionable: `[Action needed]`, `[FYI]`,
  `[Decision required]`, `[Update]`
- 6-10 words maximum — enough to survive a mobile inbox preview
- Be specific: `Q1 budget review — please approve by Friday` beats
  `Budget question`
- Never leave subject blank

### Formatting and length

- **One screen** — most emails should fit without scrolling
- **One paragraph per idea** — no walls of text
- **Bullets for lists of 3+** — easier to scan than prose
- **No bold/italic unless signaling urgency** — decorative formatting
  reads as noise in plain-text clients
- **Call to action at the end** — end with what you want: "Please
  reply by Thursday" or "No response needed — just keeping you in
  the loop"
- **Signature** — include name and title only; leave phone/links for
  formal external emails

### Tone calibration

| Relationship | Opening | Closing |
|---|---|---|
| Close colleague | Hi [first name], | Thanks / Cheers |
| Professional peer | Hi [first name], | Best / Thanks |
| Client / external | Dear [Mr./Ms. Last] / Hi [First], | Best regards / Thank you |
| Executive / senior | Hi [First], | Thanks / Best |
| Unknown / cold | Dear [First Last], | Best regards |

Match the energy the recipient uses in prior threads. When in doubt,
one step more formal than you think you need.

### Draft output format

Produce the email as copy-paste ready text:

```
Subject: [Subject line]

[Salutation],

[Body — 1-4 paragraphs, or bullets where appropriate]

[Call to action or closing line]

[Sign-off],
[Name]
[Title if applicable]
```

Do not wrap in code blocks or markdown unless the user explicitly
wants it formatted that way.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Writing a long email when the intent can be stated in two sentences.** Longer emails get less attention, not more. Before finalizing, ask: "what is the single thing this email must accomplish?" Then cut everything that doesn't serve that goal. If the email needs more than one screen, it probably needs to be a document with a covering email instead.
- **Burying the ask at the bottom.** Readers scan emails from the top. The request, decision, or key fact must appear in the first or second sentence — not after two paragraphs of background. Move context after the ask, not before.
- **Drafting without knowing the audience's relationship and context.** An email to a close colleague who knows the project needs no preamble; the same email to an external stakeholder needs a one-sentence orientation. Always check the recipient type before setting the tone and level of detail.
- **Using passive voice to soften bad news.** "Mistakes were made" and "the deadline was missed" avoid accountability. Professional emails about problems use active voice and first-person ownership: "We missed the deadline" and "I should have flagged this earlier." Passive voice reads as evasive.
- **Copying too many people.** Every extra recipient in CC adds noise and diffuses accountability. CC only people who need the information to do their job. Use BCC only for large announcements where reply-all chaos would be a problem.
- **Treating the subject line as an afterthought.** Subjects determine whether the email is opened and found later. A vague subject ("Question", "Hi", "Follow-up") fails both jobs. Write the subject last, after the body is clear, so it accurately reflects the content.
- **Sending the first draft without a read-through.** The most common email errors — wrong name, forgotten attachment, accidentally harsh phrasing, missing call to action — are caught in a single read-aloud pass. Always read the draft once from the recipient's perspective before finalizing.

## Full reference

### Reply vs. new thread

Reply to existing threads when the email is clearly part of an
ongoing conversation. Start a new thread when:
- The topic has substantially shifted from the original subject
- The original thread is long (>10 messages) and the new ask is
  distinct
- The recipient group is different from the original

When replying, trim the quoted thread to the last 1-2 relevant
messages — full thread quoting adds length without value.

### Sensitive topics

Apologies, performance feedback, salary discussions, and conflict
resolution almost always land better in person or on video.
If the email must be written:
- Lead with empathy, not defensiveness
- Use "I" statements, not "you" statements
- Propose a call to continue the conversation
- Never send an angry email immediately — draft it, save it, review
  after an hour

### Templates for common scenarios

**Meeting request:**
> Subject: [Topic] — 30-min meeting request  
> Hi [Name],  
> I'd like to connect about [topic] — [1 sentence on why it matters
> now]. Would you have 30 minutes available [proposed window]? Happy
> to adjust if that doesn't work.  
> Best, [Name]

**Resignation:**
> Subject: Resignation — [Name], effective [Date]  
> Dear [Manager],  
> I am writing to formally resign from my position as [title],
> effective [date].  
> [1-2 sentences of appreciation.]  
> I am happy to assist with the transition in any way that would
> be helpful.  
> Thank you for [specific thing].  
> Best regards, [Name]

**Reference request:**
> Subject: Reference request — [Your name], [Role] at [Company]  
> Hi [Name],  
> I'm applying for [role] at [company] and would be grateful if
> you'd be willing to serve as a reference. The role is [1 line].  
> Would you be comfortable speaking to my work on [specific
> project/skill]? If so, they may contact you at [recruiter email]
> in the coming [timeframe].  
> Please let me know if you have any questions or if you'd prefer
> not to — either is completely fine.  
> Many thanks, [Name]
