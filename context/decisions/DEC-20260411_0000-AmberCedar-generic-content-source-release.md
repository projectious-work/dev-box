---
apiVersion: processkit.projectious.work/v1
kind: DecisionRecord
metadata:
  id: DEC-20260411_0000-AmberCedar-generic-content-source-release
  created: '2026-04-10T22:33:37+00:00'
spec:
  title: Generic content-source release-asset fetcher with SHA256 verification
  state: accepted
  decision: 'The `cli/src/content_source.rs` fetcher is content-source-neutral: it
    tries a release-asset tarball first (`{source}/releases/download/{version}/processkit-{version}.tar.gz`),
    verifies against the sibling `.sha256` file when present, and falls back to git-tarball
    then git-clone for branch/unreleased-commit consumption. The `release_asset_url_template`
    in `[processkit]` is configurable for non-GitHub hosts. SHA256 recorded in `aibox.lock`
    as `release_asset_sha256: Option<String>`.'
  context: The original fetch strategy walked the full git repo tarball with skip
    rules inside aibox. This was slow, required aibox to know processkit's internal
    repo layout, and was not bit-reproducible. processkit needed a way to declare
    an explicit shipping contract.
  rationale: 'Explicit shippable contract: processkit decides what''s in the release
    artifact. Smaller/faster fetch. No skip rules needed in aibox''s walker. Bit-exact
    reproducibility via recorded asset SHA256. The hybrid (release-asset primary,
    git fallback) supports both pinned production use and branch-based development
    without code changes. The URL template makes the machinery processkit-compatible-alternative-friendly.'
  alternatives:
  - option: Keep git-tarball walk as the only strategy
    rejected_because: Requires aibox to maintain skip rules mirroring processkit's
      internal layout; slow; not bit-reproducible
  - option: Use a lockfile with pinned file hashes instead of a release tarball
    rejected_because: Requires processkit to publish a separate lockfile; more complex
      and no cleaner than a tarball + sha256 pair
  consequences: processkit must publish a release script that builds the tarball and
    attaches it + checksum to each GitHub Release. The URL template default (`{source}/releases/download/{version}/processkit-{version}.tar.gz`)
    must be stable. The `aibox.lock` schema gains `release_asset_sha256`.
  deciders:
  - ACTOR-20260411_0000-SnappyFrog-bernhard
  decided_at: '2026-04-10T22:33:37+00:00'
---
