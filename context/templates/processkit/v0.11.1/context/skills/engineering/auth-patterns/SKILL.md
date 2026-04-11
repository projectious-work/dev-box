---
name: auth-patterns
description: |
  Authentication and authorization patterns — OAuth2, JWT, sessions, RBAC/ABAC. Use when implementing login flows, securing APIs, managing tokens, designing permission systems, or reviewing auth code.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-auth-patterns
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: engineering
---

# Authentication & Authorization Patterns

## Intro

Authentication proves who a caller is; authorization decides what they
can do. Use industry-standard protocols (OAuth2, OIDC, JWT) and proven
patterns (session cookies, RBAC, ABAC) rather than inventing your own.

## Overview

### OAuth2 flow selection

Pick the flow based on the client's ability to keep secrets:

| Client type                | Flow                  | Token storage             |
|----------------------------|-----------------------|---------------------------|
| Server-side web app        | Authorization Code    | Server-side session       |
| Single-page app (SPA)      | Auth Code + PKCE      | Memory (not localStorage) |
| Mobile / native app        | Auth Code + PKCE      | Secure device storage     |
| Service-to-service         | Client Credentials    | Environment variable      |
| CLI / limited-input device | Device Authorization  | Display user code         |

See `references/oauth-flows.md` for full flow diagrams and exchange
parameters.

### JWT essentials

- Always validate signature, `exp`, `iss`, and `aud`.
- Keep payloads small — JWTs ride on every request.
- Never store secrets in payloads; base64 is encoding, not encryption.
- Use short access-token expiry (5–15 min).
- Prefer `RS256` or `ES256` for public APIs over `HS256`.
- Rotate signing keys via a JWKS endpoint.

See `references/jwt-reference.md` for the structure, claims, and full
validation checklist.

### Session management

For server-rendered apps using session cookies:

- Generate cryptographically random session IDs (≥128 bits).
- Store session data server-side (Redis, database) — never inside the
  cookie body.
- Set `HttpOnly`, `Secure`, and `SameSite=Lax` (or `Strict`) on the
  cookie.
- Apply idle timeout (15–30 min) and absolute timeout (8–24 hours).
- Regenerate the session ID after login to prevent session fixation.

### RBAC vs ABAC

**RBAC** assigns permissions to roles and roles to users. Simple and
appropriate when access depends only on the user's job function.

```python
ROLES = {
    "viewer": ["read:projects", "read:reports"],
    "editor": ["read:projects", "read:reports", "write:projects"],
    "admin":  ["read:projects", "read:reports", "write:projects",
               "manage:users", "manage:settings"],
}

def has_permission(user, permission):
    return any(permission in ROLES[role] for role in user.roles)
```

**ABAC** evaluates attributes of subject, resource, action, and
context. Use it when access depends on ownership, department, time of
day, or other context.

```python
def can_access(subject, action, resource, context):
    if action == "edit" and resource.owner_id == subject.id:
        return True
    if "admin" in subject.roles and resource.department == subject.department:
        return True
    if action == "deploy" and not context.is_maintenance_window:
        return False
    return False
```

### Token refresh

Access tokens should be short-lived. Refresh tokens trade themselves
for new access tokens at the auth server:

```
Client                    Auth Server                 API
  |                           |                        |
  |--- Request (access token) ----------------------->|
  |<-- 401 Unauthorized -------------------------------|
  |--- Refresh token -------->|                        |
  |<-- New access + refresh --|                        |
  |--- Retry (new access token) --------------------->|
  |<-- 200 OK ----------------------------------------|
```

Refresh tokens must be stored securely (httpOnly cookie or secure
device storage), single-use (rotated on every refresh), and revocable
(server-side denylist or family tracking).

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **JWT signature validation skipped or incomplete.** Accepting a JWT without verifying the signature, or without validating `exp` (expiry), `iss` (issuer), and `aud` (audience) claims, means any JWT — including expired, foreign, or forged ones — will be accepted. Always validate all four: signature, expiry, issuer, and audience. Explicitly specify the allowed algorithm list so an attacker cannot switch to `alg: none`.
- **Storing access tokens in localStorage.** Tokens stored in `localStorage` are accessible to any JavaScript running on the page, including injected scripts via XSS. Keep access tokens in memory only; store refresh tokens in `httpOnly` cookies that JavaScript cannot read.
- **Returning different error messages for "user not found" vs "wrong password."** `"User does not exist"` vs `"Incorrect password"` allows an attacker to enumerate valid usernames by observing which error they receive. Always return the same generic message: `"Invalid email or password."` regardless of which check failed.
- **Not regenerating the session ID after login.** If a user has a session cookie before logging in (e.g., from a previous anonymous session) and the same session ID is reused after authentication, an attacker who captured the pre-login session cookie can now use it to access the authenticated session. Regenerate the session ID immediately after successful login.
- **Using `HS256` for tokens consumed by clients you don't control.** `HS256` uses a shared symmetric key — anyone who knows the key can forge tokens. For public APIs where tokens are validated by multiple parties, use `RS256` or `ES256` (asymmetric), so clients can validate with the public key without being able to forge new tokens.
- **No rate limiting on authentication endpoints.** A login or password-reset endpoint without rate limiting allows unlimited brute-force attempts. Apply progressive delays, lockout, or CAPTCHA after N failed attempts per IP or per account. Use a token-bucket or sliding-window rate limiter, not a simple counter.
- **Granting blanket scopes or roles "for now."** Giving every service or user admin-level access because it's easier to configure means a compromised credential gives the attacker the full blast radius. Apply least-privilege from the beginning: each service credential covers only the permissions that service legitimately needs.

## Full reference

### API key patterns

- Prefix keys for identification: `sk_live_abc123` (secret),
  `pk_live_xyz789` (public).
- Hash keys before storage (SHA-256). Show the full key only once at
  creation.
- Scope keys to specific permissions; never grant full access by
  default.
- Embed a key ID in the key itself so lookups don't require hashing
  every stored key.

### CORS configuration

```javascript
// Express.js example
app.use(cors({
  origin: ["https://app.example.com"],     // Never use "*" with credentials
  methods: ["GET", "POST", "PUT", "DELETE"],
  allowedHeaders: ["Authorization", "Content-Type"],
  credentials: true,                        // Allow cookies
  maxAge: 86400,                            // Cache preflight 24h
}));
```

Never set `Access-Control-Allow-Origin: *` together with credentials.

### Security review checklist

1. Auth tokens are transmitted only over HTTPS.
2. Passwords are hashed with bcrypt or argon2 — never MD5/SHA1.
3. JWT signatures are validated; `alg: none` is rejected.
4. Refresh tokens are rotated on use and revocable.
5. Session IDs are regenerated after authentication.
6. CORS origins are explicitly listed (no wildcards with credentials).
7. Rate limiting is applied to login and token endpoints.
8. Failed login attempts trigger progressive delays or lockout.

### JWT validation in practice

Always specify allowed algorithms explicitly to prevent algorithm
confusion attacks (an attacker switching `RS256` to `HS256` and signing
with the public key):

```python
import jwt
from jwt import PyJWKClient

jwks_client = PyJWKClient("https://auth.example.com/.well-known/jwks.json")

def validate_token(token: str) -> dict:
    signing_key = jwks_client.get_signing_key_from_jwt(token)
    return jwt.decode(
        token,
        signing_key.key,
        algorithms=["RS256"],       # Explicit allowlist
        issuer="https://auth.example.com",
        audience="https://api.example.com",
        options={
            "require": ["exp", "iss", "aud", "sub"],
            "verify_exp": True,
            "verify_iss": True,
            "verify_aud": True,
        },
    )
```

### Common JWT vulnerabilities

- **Algorithm confusion** — fix by explicitly listing allowed
  algorithms; never trust the token's `alg` header.
- **Missing signature validation** — reject tokens with `alg: none`.
- **Tokens stored in localStorage** — readable by XSS; keep access
  tokens in memory and refresh tokens in httpOnly cookies.
- **Long expirations** — keep access tokens short-lived (5–15 min).
- **Sensitive data in payload** — base64 is encoding, not encryption.
- **No revocation path** — maintain a server-side denylist for critical
  invalidations and keep access-token lifetimes short.

### Refresh-token rotation

The auth server issues a fresh refresh token on every use and
invalidates the old one. If a previously-rotated refresh token is
presented again, treat it as theft and invalidate the entire token
family.

### Anti-patterns

- Building your own crypto or password hashing.
- Storing JWT access tokens in localStorage on a SPA.
- Using `HS256` for tokens consumed by clients you don't control.
- Granting blanket scopes or permissions because "it's easier".
- Returning different errors for "user not found" vs "wrong password"
  (enables enumeration).
- Using GET requests for state-changing operations (CSRF surface).
