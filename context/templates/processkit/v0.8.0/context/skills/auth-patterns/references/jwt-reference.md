# JWT Reference

## JWT Structure

A JWT consists of three base64url-encoded parts separated by dots:

```
HEADER.PAYLOAD.SIGNATURE

eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.
eyJzdWIiOiJ1c2VyXzEyMyIsImVtYWlsIjoiYWxpY2VAZXhhbXBsZS5jb20iLC
JpYXQiOjE3MDk1MDAwMDAsImV4cCI6MTcwOTUwMDkwMCwiaXNzIjoiaHR0cHM6L
y9hdXRoLmV4YW1wbGUuY29tIiwiYXVkIjoiaHR0cHM6Ly9hcGkuZXhhbXBsZS5
jb20ifQ.
SIGNATURE_BYTES
```

### Header

```json
{
  "alg": "RS256",
  "typ": "JWT",
  "kid": "key-2024-01"
}
```

| Field | Description                                            |
|-------|--------------------------------------------------------|
| alg   | Signing algorithm (RS256, ES256, HS256)                |
| typ   | Token type — always "JWT"                              |
| kid   | Key ID — identifies which key to use for verification  |

### Payload (Claims)

```json
{
  "sub": "user_123",
  "email": "alice@example.com",
  "roles": ["editor", "viewer"],
  "iat": 1709500000,
  "exp": 1709500900,
  "nbf": 1709500000,
  "iss": "https://auth.example.com",
  "aud": "https://api.example.com",
  "jti": "unique-token-id-abc"
}
```

### Registered Claims

| Claim | Name            | Description                              | Required |
|-------|-----------------|------------------------------------------|----------|
| sub   | Subject         | User or entity identifier                | Yes      |
| iat   | Issued At       | Unix timestamp of token creation         | Yes      |
| exp   | Expiration      | Unix timestamp when token expires        | Yes      |
| nbf   | Not Before      | Token is not valid before this time      | Optional |
| iss   | Issuer          | Who issued the token (URL)               | Yes      |
| aud   | Audience        | Intended recipient (URL or string)       | Yes      |
| jti   | JWT ID          | Unique identifier for the token          | Optional |

### Signature

```
SIGNATURE = ALGORITHM(
  base64urlEncode(header) + "." + base64urlEncode(payload),
  secret_or_private_key
)
```

## Signing Algorithms

| Algorithm | Type       | Use Case                                     |
|-----------|------------|----------------------------------------------|
| RS256     | Asymmetric | Production APIs — sign with private, verify with public |
| ES256     | Asymmetric | Smaller tokens — ECDSA, faster verification  |
| HS256     | Symmetric  | Internal services only — shared secret       |
| EdDSA     | Asymmetric | Modern alternative — Ed25519, high performance |

**Never use** `alg: none` — always reject tokens with no algorithm.

**Prefer asymmetric** algorithms for public APIs: the API server only needs the
public key, reducing blast radius if the server is compromised.

## Validation Checklist

When receiving a JWT, verify ALL of the following:

1. **Decode** the token and parse header and payload.
2. **Check `alg`** — reject `none` and any unexpected algorithm.
3. **Verify signature** using the correct key (look up via `kid` in JWKS).
4. **Check `exp`** — reject expired tokens. Allow 30-60 seconds clock skew.
5. **Check `nbf`** — reject tokens used before their not-before time.
6. **Check `iss`** — must match your expected issuer URL exactly.
7. **Check `aud`** — must include your API's identifier.
8. **Check `sub`** — ensure the subject exists and is active.
9. **Check custom claims** — validate roles, scopes, tenant_id as needed.

```python
import jwt
from jwt import PyJWKClient

JWKS_URL = "https://auth.example.com/.well-known/jwks.json"
ISSUER = "https://auth.example.com"
AUDIENCE = "https://api.example.com"

jwks_client = PyJWKClient(JWKS_URL)

def validate_token(token: str) -> dict:
    signing_key = jwks_client.get_signing_key_from_jwt(token)
    payload = jwt.decode(
        token,
        signing_key.key,
        algorithms=["RS256"],       # Explicitly list allowed algorithms
        issuer=ISSUER,
        audience=AUDIENCE,
        options={
            "require": ["exp", "iss", "aud", "sub"],
            "verify_exp": True,
            "verify_iss": True,
            "verify_aud": True,
        },
    )
    return payload
```

## Common Vulnerabilities

### 1. Algorithm Confusion (CVE-2015-9235)

**Attack:** Attacker changes `alg` from RS256 to HS256 and signs with the public key.

**Fix:** Always specify allowed algorithms explicitly. Never trust the `alg` header.

```python
# WRONG — trusts the token's alg claim
jwt.decode(token, key)

# RIGHT — explicitly list allowed algorithms
jwt.decode(token, key, algorithms=["RS256"])
```

### 2. Missing Signature Validation

**Attack:** Set `alg: none` and remove the signature.

**Fix:** Reject tokens with `alg: none`. Use a library that validates by default.

### 3. Token Stored in localStorage

**Attack:** XSS can read localStorage and steal the token.

**Fix:** Store access tokens in memory. Use httpOnly cookies for refresh tokens.

### 4. No Expiration or Long Expiration

**Attack:** Stolen token works indefinitely.

**Fix:** Set short expiry (5-15 min for access, hours-days for refresh).

### 5. Sensitive Data in Payload

**Attack:** Base64 is encoding, not encryption — anyone can read the payload.

**Fix:** Never put passwords, credit card numbers, or secrets in JWT claims.

### 6. No Token Revocation

**Attack:** Compromised token cannot be invalidated before expiry.

**Fix:** Maintain a server-side denylist for critical revocations. Keep access token
lifetime short to limit the window.

## Access Token vs Refresh Token

| Property        | Access Token            | Refresh Token              |
|-----------------|-------------------------|----------------------------|
| Lifetime        | 5-15 minutes            | Hours to days              |
| Storage         | Memory (SPA) / variable | httpOnly cookie / secure storage |
| Sent with       | Every API request       | Only to token endpoint     |
| Contains        | User claims, scopes     | Opaque or minimal claims   |
| Revocable       | Not easily (short-lived)| Yes (server-side tracking) |

## JWKS (JSON Web Key Set)

Publish public keys at a well-known endpoint for token verification:

```
GET https://auth.example.com/.well-known/jwks.json

{
  "keys": [
    {
      "kty": "RSA",
      "kid": "key-2024-01",
      "use": "sig",
      "alg": "RS256",
      "n": "0vx7agoebGcQSuuPiLJXZptN9...",
      "e": "AQAB"
    }
  ]
}
```

Clients cache JWKS and refresh when they encounter an unknown `kid`.
