---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-TallDawn-python3-uv-unconditionally-in
  created: '2026-04-10T22:34:58+00:00'
spec:
  title: python3 + uv unconditionally in the base-debian image
  state: accepted
  decision: 'v0.16.5 bakes `python3` (Debian Trixie, 3.13.x) and `uv` (from `ghcr.io/astral-sh/uv:latest`
    via `COPY --from=`) into the base-debian image unconditionally. The python addon''s
    purpose shifts from "Python at all" to "additional Python tooling beyond the base
    minimum" (poetry, pdm, alternative versions, recommended skills). Size impact:
    ~75–100 MB added to an ~800 MB base (~10% growth).'
  context: 'processkit''s MCP servers are PEP 723 scripts that run via `uv run`. For
    MCP servers to work, both `python3` and `uv` MUST be present in the container.
    Every aibox project consumes processkit; therefore python+uv are always needed.
    Gating them on a separate addon was a footgun: skipping the python addon gave
    "MCP servers fail to launch" with no obvious diagnosis.'
  rationale: Baking into the base eliminates the footgun at the cost of a one-time
    ~10% image size increase. The alternative (auto-add the python addon as a transitive
    require of any skill with mcp/) introduces a new coupling direction — skills influencing
    addon selection — which violates the install pipeline's layering. `uv python install`
    was rejected because it downloads Python at runtime, slowing first launch; apt-installed
    python3 is faster and smaller.
  alternatives:
  - option: Auto-add python addon at sync time when any installed skill has mcp/
    rejected_because: 'Introduces coupling: skills influencing addon selection; python
      addon is more than just python+uv; still requires addon-build step on first
      use'
  - option: Use python:3.13-slim-trixie upstream image as base
    rejected_because: Replaces the base image identity (debian → python); complicates
      future language addon story
  - option: Manage python via uv python install
    rejected_because: Downloads Python at runtime, slows first launch; apt-installed
      python3 is faster and is uv's documented happy path
  consequences: All aibox base images include python3 + uv. First `aibox sync` after
    the v0.16.5 base image triggers a fresh image pull. Projects with the python addon
    already enabled see no behavioral change. Min Python supported by uv is 3.9; Debian
    Trixie ships 3.13 — well above minimum.
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:34:58+00:00'
---
