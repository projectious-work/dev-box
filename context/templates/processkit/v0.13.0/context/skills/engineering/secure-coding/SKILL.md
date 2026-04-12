---
name: secure-coding
description: |
  Secure coding practices grounded in the OWASP Top 10. Use when reviewing code for security issues, implementing input validation, hardening web applications, or adding defenses against injection, XSS, or CSRF.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-secure-coding
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Secure Coding

## Intro

Most application vulnerabilities come from a small set of recurring
mistakes: trusting input, concatenating strings into queries,
forgetting to encode output, leaving secrets in source. Apply OWASP
Top 10 patterns to identify and fix them.

## Overview

### Input validation

Validate every input at the boundary — never trust client-supplied
data:

- **Allowlist over denylist** — define what IS valid, not what isn't.
- **Validate type, length, range, and format** before processing.
- **Reject unexpected input** — fail closed, not open.
- **Validate on the server.** Client-side validation is for UX only.

```python
import re

def validate_username(username: str) -> str:
    """Allowlist approach: only permit safe characters."""
    if not re.match(r'^[a-zA-Z0-9_]{3,30}$', username):
        raise ValueError("Invalid username format")
    return username
```

### Output encoding

Encode output based on the context where it lands:

| Context        | Encoding             | Example          |
|----------------|----------------------|------------------|
| HTML body      | HTML entity encoding | `&lt;script&gt;` |
| HTML attribute | Attribute encoding   | `" -> &quot;`    |
| JavaScript     | JS hex encoding      | `\x3Cscript\x3E` |
| URL parameter  | Percent encoding     | `%3Cscript%3E`   |
| CSS            | CSS hex encoding     | `\3C script\3E`  |

Use the framework's built-in template escaping. Never build HTML with
string concatenation.

### Injection prevention

**SQL injection** — use parameterized queries:

```python
cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
```

**Command injection** — use library functions or subprocess with an
argument list. Never `shell=True` with user input:

```python
subprocess.run(["convert", filename, "output.png"], check=True)
```

### CSRF protection

- Use anti-CSRF tokens (synchronizer token pattern) for state-changing
  requests.
- Set `SameSite=Lax` or `SameSite=Strict` on session cookies.
- Verify `Origin` and `Referer` headers as defense in depth.
- Never use GET for state-changing operations.

### Security headers

Apply these on every response:

```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'
Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
```

Start with a strict CSP and relax only as needed. Never use
`unsafe-eval`.

### Secrets and dependencies

- Never hardcode secrets. Use environment variables or a secrets
  manager.
- Add `.env`, `*.pem`, and `credentials.json` to `.gitignore`.
- Audit dependencies regularly: `npm audit`, `pip-audit`, `cargo audit`.
- Pin versions in lockfiles and minimize the dependency tree.

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **String concatenation to build SQL queries.** `"SELECT * FROM users WHERE id = " + user_id` is a SQL injection vulnerability whenever `user_id` comes from an untrusted source. Use parameterized queries or prepared statements unconditionally — not "only for user input", because distinguishing trusted from untrusted input is error-prone. The parameterized form is not harder to write.
- **Client-side-only input validation.** Validation in JavaScript can be bypassed by anyone who sends a raw HTTP request. Client-side validation is for user experience only; it provides zero security. All security-relevant validation — type, length, range, format — must happen on the server, on every request.
- **Returning stack traces or internal error details to clients.** An unhandled exception that returns the full stack trace, database query, or file path to the client leaks the internal structure of the application to attackers. All unhandled exceptions must be caught at the boundary and returned as a generic error message; log the full detail internally.
- **Using GET requests for state-changing operations.** A GET request that creates, modifies, or deletes data can be triggered by a CSRF attack — the browser will follow GET links embedded in a malicious page without any user action. Use POST, PUT, PATCH, or DELETE for state-changing operations and protect them with anti-CSRF tokens.
- **Trusting the `alg` header in a JWT.** A JWT that claims `"alg": "none"` should be rejected outright — it has no signature. A JWT that claims `"alg": "HS256"` when the server uses `RS256` can be forged by an attacker using the public key as the HMAC secret. Always explicitly specify which algorithms are allowed; never use the algorithm from the token header to choose the validation method.
- **Hardcoded secrets in source code.** An API key, password, or signing key hardcoded in source code is stored in git history permanently — even after deletion it remains in previous commits. Any user with repository access has the secret. All secrets must be loaded at runtime from environment variables or a secrets manager.
- **Insufficient authorization checks — checking once at the boundary.** Verifying that a user is logged in at the route level but not checking whether they own the specific resource they are accessing allows authenticated users to read or modify other users' data. Check resource ownership on every protected endpoint: not just "is the user authenticated?" but "does this user own this specific record?"

## Full reference

### Security review checklist

1. All user input is validated (type, length, format).
2. SQL queries use parameterized statements.
3. HTML output is context-encoded — no raw string interpolation.
4. CSRF tokens protect state-changing endpoints.
5. Security headers are set (CSP, HSTS, X-Frame-Options).
6. No secrets in source code or logs.
7. Authentication uses bcrypt/argon2, not MD5/SHA1.
8. File uploads validate type and size and store outside the webroot.
9. Error messages don't leak stack traces or internal details.
10. Dependencies are audited and up to date.

See `references/owasp-checklist.md` for the full OWASP Top 10 with
prevention checklists and secure code patterns.

### OWASP Top 10 highlights

- **A01 Broken Access Control** — deny by default; check authorization
  on every request, not just in the UI; use indirect object
  references; log access-control failures.
- **A02 Cryptographic Failures** — bcrypt or argon2id for passwords;
  AES-256-GCM for data at rest; TLS 1.2+ in transit; rotate keys.
- **A03 Injection** — parameterized queries everywhere; subprocess
  with argument lists; ORM methods over raw SQL; allowlist input.
- **A04 Insecure Design** — rate-limit auth endpoints, return generic
  errors to prevent enumeration, threat-model during design.
- **A05 Security Misconfiguration** — disable debug in production,
  remove default credentials, audit cloud bucket permissions.
- **A06 Vulnerable Components** — run `pip-audit` / `npm audit` /
  `cargo audit` in CI; pin lockfiles; monitor with Dependabot.
- **A07 Authentication Failures** — strong passwords, rate limiting,
  session regeneration after login, MFA support.
- **A08 Software & Data Integrity Failures** — never deserialize
  untrusted data with formats that allow code execution; verify
  signatures on updates; protect CI/CD with access controls.
- **A09 Logging & Monitoring Failures** — log auth events and access
  failures; never log secrets; alert on anomalies.
- **A10 SSRF** — allowlist URLs and hosts; block private/internal IP
  ranges; validate redirect targets.

### Secure patterns by category

**Access control** — explicit ownership check on every protected
endpoint:

```python
@app.get("/api/users/{user_id}/profile")
def get_profile(user_id: int, current_user: User = Depends(get_current_user)):
    if current_user.id != user_id and "admin" not in current_user.roles:
        raise HTTPException(status_code=403, detail="Forbidden")
    return db.query(User).filter(User.id == user_id).first()
```

**Password hashing** — never roll your own:

```python
from passlib.context import CryptContext
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
password_hash = pwd_context.hash(password)
# pwd_context.verify(password, password_hash)
```

**Generic error messages** — prevent enumeration:

```python
@app.post("/reset-password")
@limiter.limit("3/hour")
def reset_password(email: str):
    send_reset_email(email)
    return {"message": "If the email exists, a reset link was sent."}
```

**SSRF defence** — validate URLs against an allowlist and block
private IP ranges:

```python
from urllib.parse import urlparse
import ipaddress

ALLOWED_HOSTS = {"api.example.com", "cdn.example.com"}

def validate_url(url: str) -> str:
    parsed = urlparse(url)
    if parsed.scheme not in ("http", "https"):
        raise ValueError("Invalid scheme")
    if parsed.hostname not in ALLOWED_HOSTS:
        raise ValueError("Host not allowed")
    try:
        ip = ipaddress.ip_address(parsed.hostname)
        if ip.is_private or ip.is_loopback:
            raise ValueError("Internal addresses not allowed")
    except ValueError:
        pass
    return url
```

### Anti-patterns

- Building SQL with f-strings or `+`.
- Disabling output escaping in templates "just this once".
- `subprocess.run(..., shell=True)` with any user-derived value.
- Using GET for actions that mutate state.
- Returning stack traces to clients.
- Storing passwords with unsalted hashes (MD5, SHA1, plain SHA256).
- Trusting the `alg` header in a JWT — explicitly list allowed
  algorithms.
- Deserializing untrusted YAML, pickle, or similar formats that allow
  code execution.
