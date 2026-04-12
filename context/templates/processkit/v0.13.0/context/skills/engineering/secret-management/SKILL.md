---
name: secret-management
description: |
  Handle secrets safely — env vars, .env files, vaults, rotation. Use when dealing with API keys, passwords, tokens, database credentials, or any other sensitive value the application needs at runtime.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-secret-management
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Secret Management

## Intro

Secrets are anything an attacker could use to impersonate the
application: API keys, passwords, tokens, database credentials,
signing keys. They must never be committed to source control, must be
loaded at runtime from a secure source, and must be rotated when
compromised or on a schedule.

## Overview

### Never commit secrets

- Add `.env` to `.gitignore` *before* creating it.
- Search history when in doubt:
  `git log --all -p | grep -i "password\|secret\|key\|token"`.
- If a secret was ever committed, treat it as compromised and rotate
  it immediately. Removing the commit does not unleak the value.

### Local development

- Load secrets from a `.env` file via the application's dotenv loader.
- Commit a `.env.example` with placeholder values so new contributors
  know what's required.
- Document the required environment variables in the README.

### CI/CD

- Use the platform's secret store (GitHub Secrets, GitLab CI variables,
  CircleCI contexts).
- Never echo or log a secret in pipeline output.
- Prefer OIDC short-lived tokens (e.g. GitHub Actions to AWS) over
  long-lived static credentials.

### Production

- Use a real secrets manager: HashiCorp Vault, AWS Secrets Manager,
  GCP Secret Manager, Azure Key Vault.
- Rotate on a schedule (90 days is a common default).
- Prefer short-lived issued tokens over permanent credentials.
- Apply least privilege: each service receives only the secrets it
  needs.
- Use distinct secrets per environment (dev / staging / prod).

### Code patterns

- Read secrets from environment variables, not from config files
  checked into the repo.
- Never hardcode secrets — not even temporarily "for testing".
- Never log secrets; redact them in error messages and stack traces.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Treating a secret that was ever committed to git as safe after deletion.** Removing a secret from the current branch does not remove it from git history — anyone with access to the repository can recover it with `git log -p`. A secret that was committed must be treated as compromised immediately: rotate it, revoke the old value, and verify it is not cached in any deployed system.
- **Sharing secrets across environments.** Using the same database password, API key, or signing key in development, staging, and production means a leak from any environment exposes production. Provision distinct secrets per environment and rotate them independently. A staging breach should not require production rotation.
- **Storing secrets in container images at build time via `ARG` or `ENV`.** Build arguments and environment variables baked into an image are visible in the image layer history (`docker history`). Anyone who can pull the image can extract the secret. Inject secrets at runtime via environment variables from a secrets manager or a Kubernetes Secret, not at build time.
- **Using base64 encoding as if it were encryption.** A secret that is base64-encoded is obfuscated, not protected. Base64 is trivially reversible in one command. Never treat base64 encoding as a security control; it is an encoding format, not encryption.
- **Long-lived static credentials where short-lived OIDC tokens are available.** Static API keys and passwords that never expire are permanently valid once leaked. Many cloud platforms support OIDC-based workload identity where the CI or runtime system gets a short-lived token that expires in minutes or hours. Use OIDC wherever available; it eliminates the risk of long-lived credential exposure.
- **Secrets shared over Slack, email, or chat.** Communication platforms retain message history, are accessible to administrators, and may be indexed by third-party search tools. Never transmit a secret value through a chat channel — use a secrets manager with access controls and an audit log so every access is tracked.
- **No audit of who can read production secrets.** A secrets manager with no access policy review may have dozens of service accounts, developers, and CI jobs with access to production credentials long after those access rights are no longer needed. Review who can read each secret quarterly and apply least-privilege: each service should only be able to read the specific secrets it needs.

## Full reference

### Adding a new secret — checklist

1. Confirm `.env` is in `.gitignore`.
2. Add the variable name (with no value) to `.env.example`.
3. Document the variable in the README.
4. Read it in code via the environment, e.g.
   `os.environ["PAYMENT_API_KEY"]`.
5. Add it to the CI secret store and the production secrets manager.
6. Verify the application fails fast at startup if it's missing,
   rather than crashing later in a request.

### Rotation playbook

1. Generate the new secret in the upstream system.
2. Add it alongside the old secret (most providers allow both to be
   active at once).
3. Roll the new value into the secrets manager.
4. Restart or hot-reload services to pick up the new value.
5. Revoke the old secret only after the rollout is verified.
6. Record the rotation date in the team's secret inventory.

### Detecting leaks

- Run a secret scanner in CI: `gitleaks`, `trufflehog`, or the native
  GitHub secret scanning. Catch leaks before they hit `main`.
- Periodically scan production logs for accidentally-emitted secrets.
- Subscribe to alerts from your hosting provider's leaked-credential
  notifications.

### Anti-patterns

- Committing `.env` "just for now" and planning to remove it later.
- Putting secrets in container images at build time.
- Sharing secrets over Slack or email.
- Reusing the same database password across environments.
- Storing secrets in client-side code or mobile binaries — they will
  be extracted.
- Treating base64 encoding as encryption.
