# OWASP Top 10 Checklist

Practical checklist covering each OWASP Top 10 (2021) category with secure code
patterns and prevention steps.

## A01: Broken Access Control

**What it is:** Users act outside their intended permissions — accessing other users'
data, elevating privileges, or bypassing access controls.

**Secure pattern:**

```python
@app.get("/api/users/{user_id}/profile")
def get_profile(user_id: int, current_user: User = Depends(get_current_user)):
    if current_user.id != user_id and "admin" not in current_user.roles:
        raise HTTPException(status_code=403, detail="Forbidden")
    return db.query(User).filter(User.id == user_id).first()
```

**Prevention:**
- [ ] Deny by default — require explicit grants for every endpoint.
- [ ] Check authorization on every request, not just in the UI.
- [ ] Use indirect object references (UUIDs) instead of sequential IDs.
- [ ] Log and alert on access control failures.
- [ ] Implement resource-level ownership checks.

## A02: Cryptographic Failures

**What it is:** Sensitive data exposed due to weak cryptography, missing encryption,
or poor key management.

**Secure pattern:**

```python
# Using bcrypt with automatic salting
from passlib.context import CryptContext
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")
password_hash = pwd_context.hash(password)
# Verify: pwd_context.verify(password, password_hash)
```

**Prevention:**
- [ ] Use bcrypt or argon2id for passwords (never MD5, SHA1, or plain SHA256).
- [ ] Encrypt sensitive data at rest (AES-256-GCM).
- [ ] Enforce TLS 1.2+ for all data in transit.
- [ ] Never store secrets in source code or logs.
- [ ] Rotate encryption keys periodically.

## A03: Injection

**What it is:** Untrusted data sent to an interpreter as part of a command or query.
Includes SQL injection, command injection, LDAP injection, and similar.

**Secure pattern (SQL):**

```python
# Always use parameterized queries
cursor.execute("SELECT * FROM users WHERE email = %s", (email,))
```

**Secure pattern (commands):**

```python
# Always use argument lists, never string interpolation with shell
subprocess.run(["ping", "-c", "1", hostname], check=True, capture_output=True)
```

**Prevention:**
- [ ] Use parameterized queries for all database operations.
- [ ] Use ORM methods instead of raw SQL when possible.
- [ ] Use subprocess with argument lists, never shell=True with user input.
- [ ] Validate and sanitize all input against an allowlist.
- [ ] Apply least-privilege database accounts.

## A04: Insecure Design

**What it is:** Missing or ineffective security controls due to design flaws, not
implementation bugs.

**Secure pattern:**

```python
from slowapi import Limiter
limiter = Limiter(key_func=get_remote_address)

@app.post("/reset-password")
@limiter.limit("3/hour")
def reset_password(email: str):
    send_reset_email(email)
    # Generic message prevents email enumeration
    return {"message": "If the email exists, a reset link was sent."}
```

**Prevention:**
- [ ] Implement rate limiting on auth-related endpoints.
- [ ] Use generic error messages (don't reveal if email/username exists).
- [ ] Design with least privilege from the start.
- [ ] Conduct threat modeling during design phase.
- [ ] Establish secure defaults for all configurable options.

## A05: Security Misconfiguration

**What it is:** Insecure default configurations, open cloud storage, verbose error
messages, unnecessary features enabled.

**Secure pattern:**

```python
app = Flask(__name__)
app.config["DEBUG"] = False
app.config["TESTING"] = False

@app.errorhandler(500)
def internal_error(error):
    # Never expose stack traces to users
    return {"error": {"code": "INTERNAL_ERROR", "message": "An error occurred."}}, 500
```

**Prevention:**
- [ ] Disable debug mode and verbose errors in production.
- [ ] Remove default credentials and sample data.
- [ ] Disable unnecessary HTTP methods, ports, and services.
- [ ] Set security headers (CSP, HSTS, X-Frame-Options).
- [ ] Review cloud storage bucket permissions.
- [ ] Automate configuration auditing.

## A06: Vulnerable and Outdated Components

**What it is:** Using libraries or frameworks with known vulnerabilities.

**Prevention:**
- [ ] Run `npm audit` / `pip-audit` / `cargo audit` in CI.
- [ ] Pin dependencies with lock files.
- [ ] Monitor for CVEs with Dependabot, Snyk, or similar.
- [ ] Remove unused dependencies.
- [ ] Update vulnerable dependencies promptly.
- [ ] Maintain a software bill of materials (SBOM).

## A07: Identification and Authentication Failures

**What it is:** Broken authentication — weak passwords, credential stuffing, session
fixation, missing MFA.

**Secure pattern:**

```python
from limits import RateLimiter
login_limiter = RateLimiter("5/minute", key_func=lambda: request.remote_addr)

@app.post("/login")
def login(username: str, password: str):
    if not login_limiter.check():
        return {"error": "Too many attempts. Try again later."}, 429

    user = db.get_user(username)
    if user and verify_password(password, user.password_hash):
        session.regenerate()  # Prevent session fixation
        return create_session(user)

    login_limiter.hit()
    return {"error": "Invalid credentials"}, 401
```

**Prevention:**
- [ ] Enforce strong passwords (min 12 chars, check against breach lists).
- [ ] Implement rate limiting and account lockout.
- [ ] Regenerate session IDs after login.
- [ ] Support and encourage MFA.
- [ ] Hash passwords with bcrypt/argon2id.

## A08: Software and Data Integrity Failures

**What it is:** Code and infrastructure not protected against integrity violations —
unsigned updates, untrusted CI/CD pipelines, insecure deserialization.

**Secure pattern:**

```python
import json
# Use JSON for external data — safe, no code execution
user_data = json.loads(request.data)
# Validate with schema
UserSchema(**user_data)
```

**Prevention:**
- [ ] Never deserialize untrusted data with formats that allow code execution.
- [ ] Use JSON or other data-only formats for external input.
- [ ] Verify signatures on software updates and packages.
- [ ] Protect CI/CD pipelines with access controls and audit logs.
- [ ] Use Subresource Integrity (SRI) for external scripts.

## A09: Security Logging and Monitoring Failures

**What it is:** Insufficient logging of security events, enabling attackers to
remain undetected.

**Prevention:**
- [ ] Log all authentication events (success and failure).
- [ ] Log access control failures.
- [ ] Log input validation failures with source IP.
- [ ] Never log secrets, tokens, passwords, or full credit card numbers.
- [ ] Set up alerting on anomalous patterns (spike in 401s, etc.).
- [ ] Retain logs for at least 90 days.
- [ ] Use structured logging (JSON) for machine parsing.

## A10: Server-Side Request Forgery (SSRF)

**What it is:** Application fetches a remote resource using user-supplied URL without
validation, allowing access to internal services.

**Secure pattern:**

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
    # Block internal IPs
    try:
        ip = ipaddress.ip_address(parsed.hostname)
        if ip.is_private or ip.is_loopback:
            raise ValueError("Internal addresses not allowed")
    except ValueError:
        pass  # Hostname is not an IP — check allowlist above
    return url

@app.post("/fetch-url")
def fetch_url(url: str):
    safe_url = validate_url(url)
    response = requests.get(safe_url, timeout=5)
    return response.text
```

**Prevention:**
- [ ] Allowlist permitted URLs, hosts, or IP ranges.
- [ ] Block requests to private/internal IP ranges (10.x, 172.16.x, 192.168.x, 169.254.x).
- [ ] Disable HTTP redirects or validate redirect targets.
- [ ] Use network-level segmentation (firewall rules) as defense in depth.
