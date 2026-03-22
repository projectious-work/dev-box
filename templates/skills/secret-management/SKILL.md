---
name: secret-management
description: Guides secure handling of secrets — env vars, .env files, vault patterns. Use when dealing with API keys, passwords, tokens, or credentials.
---

# Secret Management

## When to Use

When the user needs to handle API keys, passwords, tokens, database credentials, or asks "where should I put this secret?", "is this safe?", or "how do I manage credentials?".

## Instructions

1. **Never commit secrets to git:**
   - Add `.env` to `.gitignore` before creating it
   - Use `git log --all -p | grep -i "password\|secret\|key\|token"` to check history
   - If secrets were committed, rotate them immediately (they're compromised)
2. **Local development:**
   - Use `.env` files loaded by the application (dotenv pattern)
   - Provide `.env.example` with placeholder values (committed to git)
   - Document required environment variables in README
3. **CI/CD:**
   - Use the platform's secret store (GitHub Secrets, GitLab CI Variables)
   - Never echo or log secrets in pipelines
   - Use OIDC tokens instead of long-lived credentials where possible
4. **Production:**
   - Use a secrets manager (Vault, AWS Secrets Manager, GCP Secret Manager)
   - Rotate secrets on a schedule (90 days recommended)
   - Use short-lived tokens over permanent credentials
   - Apply least-privilege: each service gets only the secrets it needs
5. **Code patterns:**
   - Read secrets from environment variables, not config files
   - Never hardcode secrets, even for "testing"
   - Never log secrets — redact them in error messages
   - Use separate secrets for each environment (dev/staging/prod)

## Examples

**User:** "I need to add an API key for the payment provider"
**Agent:** Adds `PAYMENT_API_KEY=` to `.env.example`, updates `.gitignore` to include `.env`, reads the key from `os.environ["PAYMENT_API_KEY"]` in code, and documents the required variable.
