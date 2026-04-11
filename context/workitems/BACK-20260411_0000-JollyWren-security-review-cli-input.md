---
apiVersion: processkit.projectious.work/v1
kind: WorkItem
metadata:
  id: BACK-20260411_0000-JollyWren-security-review-cli-input
  created: '2026-04-10T22:36:20+00:00'
  labels:
    old_id: BACK-002
    area: security
spec:
  title: Security review — CLI input validation, container security, supply chain
  state: backlog
  type: task
  priority: high
  description: 'CLI: input validation (container names, hostnames, package names,
    env names — injection vectors in generated Dockerfiles/compose); file ops (symlink
    following, path traversal in backup/restore/env); network calls (TLS, no credential
    leaks in update cmd); TOML config parsing (DoS via large/malformed input). Container:
    default root user (document implications, recommend non-root); mount permissions;
    Claude CLI install script (pins or hash verify); Dockerfile injection via user-controlled
    values (extra_packages, env vars, post_create_command); PulseAudio TCP without
    auth (document exposure). Supply chain: `cargo audit` on Cargo.lock; image provenance/cosign
    (sigstore/cosign for published images); binary checksum verification in base Dockerfile;
    skill hash verification + allowed-tools audit; curl-pipe-bash install script (document
    verification). Old ID: BACK-002.'
---
