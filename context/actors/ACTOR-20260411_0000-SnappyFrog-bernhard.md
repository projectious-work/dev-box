---
apiVersion: processkit.projectious.work/v1
kind: Actor
metadata:
  id: ACTOR-20260411_0000-SnappyFrog-bernhard
  created: '2026-04-10T22:09:07+00:00'
spec:
  type: human
  name: Bernhard
  active: true
  joined_at: '2026-04-10T22:09:07+00:00'
  handle: bnaard
  expertise:
  - AI consulting
  - DevOps
  - containerized development environments
  - Python
  - Rust
  - shell scripting
  - AI-assisted workflows
  preferences:
    communication_style: Concise and direct, pragmatic over theoretical. No trailing
      summaries after completing work.
    communication_language: English for code and docs, German native speaker
    code_style: Zero clippy warnings with -D warnings; conventional commits; Rust
      for host tools
    commit_style: 'Always reference GitHub issue numbers in commits (fixes #N, refs
      #N). Include Cargo.lock in version bump commits.'
    review_style: Users expect --version flag and tarball contents docs.
    release_authority: Direct release authority — when owner says ship it, do the
      full ritual end-to-end. Phase 2 is always owner's job on macOS host.
    pr_style: One big change over many small PRs for breaking releases. Direct commits
      to main, no PR ceremony.
    working_context: 'Europe/Berlin timezone. Current focus: aibox CLI evolution,
      migrating existing projects to aibox.'
    non_negotiables:
    - Provider neutrality — no AI provider lock-in anywhere
    - Cross-host neutrality — no macOS-only solutions
    - processkit content stays in processkit, not aibox
---
