# Security Skills

Skills for application security, authentication, and threat analysis.

---

### dependency-audit

> Audits project dependencies for vulnerabilities and outdated packages. Use when checking security posture or planning dependency updates.

**Triggers:** Checking dependencies, auditing security, updating packages, verifying dependency health before a release.
**Tools:** None
**References:** None

Key capabilities:

- Multi-ecosystem audit tool selection (cargo audit, pip-audit, npm audit, govulncheck)
- Severity-based triage: critical/high (fix immediately), medium (this sprint), low (when convenient)
- Update strategy: one dependency at a time, full test suite after each, changelog review
- Outdated package detection (cargo outdated, pip list --outdated, npm outdated)
- Ongoing maintenance: monthly reviews, Dependabot/Renovate automation, pinned version documentation

??? example "Example usage"
    User asks "Are my dependencies secure?" The agent runs the appropriate audit
    tool for the project's package manager, summarizes findings by severity, and
    recommends specific version bumps for vulnerable packages. Flags any
    dependencies with no maintained alternatives.

---

### secret-management

> Guides secure handling of secrets -- env vars, .env files, vault patterns. Use when dealing with API keys, passwords, tokens, or credentials.

**Triggers:** Handling API keys, passwords, tokens, database credentials, or asking where to store secrets and how to manage credentials.
**Tools:** None
**References:** None

Key capabilities:

- Git secret prevention: .gitignore setup, history scanning, immediate rotation if committed
- Local development: .env files with dotenv pattern, .env.example with placeholders
- CI/CD: platform secret stores (GitHub Secrets, GitLab CI Variables), OIDC tokens over long-lived credentials
- Production: secrets managers (Vault, AWS/GCP Secret Manager), 90-day rotation, least-privilege access
- Code patterns: environment variable reads, no hardcoding, secret redaction in logs, per-environment isolation

??? example "Example usage"
    User needs to add an API key for a payment provider. The agent adds
    `PAYMENT_API_KEY=` to `.env.example`, updates `.gitignore` to include `.env`,
    reads the key from `os.environ["PAYMENT_API_KEY"]` in code, and documents
    the required variable.

---

### auth-patterns

> Authentication and authorization patterns including OAuth2, JWT, session management, and RBAC/ABAC. Use when implementing login flows, securing APIs, managing tokens, or designing permission systems.

**Triggers:** Implementing OAuth2 login flows, working with JWTs, designing session management, building RBAC or ABAC permission systems, securing API endpoints, reviewing authentication code.
**Tools:** `Bash` `Read` `Write`
**References:** `oauth-flows.md`, `jwt-reference.md`

Key capabilities:

- OAuth2 flow selection by client type (Authorization Code, PKCE, Client Credentials, Device Authorization)
- JWT best practices: validation (signature, exp, iss, aud), short expiry, RS256/ES256, JWKS rotation
- Token refresh pattern with single-use rotating refresh tokens
- Session management: cryptographic IDs, server-side storage, HttpOnly/Secure/SameSite cookies, idle and absolute timeouts
- RBAC with permission-to-role mapping and role-to-user assignment
- ABAC with policy evaluation based on subject, resource, action, and context attributes
- API key patterns: prefixed keys, hashed storage, scoped permissions
- CORS configuration with explicit origins (never wildcard with credentials)
- Security review checklist (8-point verification for auth implementations)

??? example "Example usage"
    Add Google OAuth login to a React SPA. The agent implements the full PKCE flow:
    generates code_verifier and code_challenge, redirects to Google, exchanges the
    authorization code for tokens, stores the access token in memory (not
    localStorage), sets up silent refresh, and adds logout with token revocation.

---

### secure-coding

> Secure coding practices based on OWASP Top 10. Injection prevention, XSS mitigation, CSRF protection, input validation, and security headers. Use when reviewing code for security, implementing auth, or hardening web applications.

**Triggers:** Reviewing code for security vulnerabilities, implementing input validation or output encoding, adding security headers, preventing injection attacks, implementing CSRF protection, auditing for secrets exposure, hardening before production.
**Tools:** `Bash` `Read` `Write`
**References:** `owasp-checklist.md`

Key capabilities:

- Input validation: allowlist over denylist, type/length/range/format checks, server-side enforcement
- Context-aware output encoding (HTML body, attributes, JavaScript, URL, CSS)
- Injection prevention: parameterized SQL queries, subprocess argument lists (no shell=True)
- CSRF protection: anti-CSRF tokens, SameSite cookies, Origin/Referer verification
- Security headers: CSP, HSTS, X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Permissions-Policy
- Secrets management: environment variables, .gitignore patterns, per-environment isolation
- Dependency security: regular audits, pinned versions, minimized dependency tree
- Security review checklist (10-point verification covering input, output, auth, headers, secrets, deps)

??? example "Example usage"
    Review an Express.js app for security issues. The agent identifies SQL queries
    built with string concatenation, user input rendered without escaping, missing
    CSRF tokens, absent security headers, and a hardcoded API key. Writes fixes
    for each issue including parameterized queries, output encoding, csurf
    middleware, helmet with strict CSP, and environment variable migration.

---

### threat-modeling

> Threat modeling using STRIDE methodology. Data flow diagrams, trust boundaries, attack surface mapping, and risk assessment. Use when analyzing system security, designing secure architectures, or conducting security reviews.

**Triggers:** Designing a new system handling sensitive data, reviewing architecture for security, conducting threat assessments, identifying trust boundaries and attack surfaces, prioritizing security work by risk.
**Tools:** None
**References:** None

Key capabilities:

- STRIDE methodology: Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege
- Data flow diagram creation with processes, data stores, data flows, external entities, and trust boundaries
- Trust boundary identification across network segments, privilege levels, and service boundaries
- Systematic STRIDE analysis applied to each component and data flow
- Risk assessment matrix (likelihood x impact) with Critical/High/Medium/Low ratings
- Mitigation strategies mapped to each STRIDE category (MFA, encryption, audit logging, rate limiting, etc.)
- Attack surface mapping across network, application, authentication, data, infrastructure, and human vectors
- Structured output format with system description, DFD, assets, threat table, and prioritized recommendations

??? example "Example usage"
    Pre-launch security review focused on highest risks. The agent identifies
    missing webhook signature verification in the payment flow (Critical), no rate
    limiting on registration (High), admin panel accessible without VPN (Critical),
    and file uploads without type validation (High). Produces a prioritized punch
    list with specific mitigations.
