# OAuth2 Flows Reference

## Authorization Code Flow

**Use for:** Server-side web applications with a backend that can keep secrets.

```
  Browser              App Server           Auth Server
    |                      |                     |
    |-- Click "Login" ---->|                     |
    |                      |-- Redirect -------->|
    |<-- 302 to auth URL --|                     |
    |                                            |
    |-- User authenticates ---------------------->|
    |<-- 302 to callback with ?code=xyz ---------|
    |                                            |
    |-- GET /callback?code=xyz -->|              |
    |                             |-- POST /token (code + client_secret) -->|
    |                             |<-- access_token + refresh_token --------|
    |                             |              |
    |<-- Set session cookie ------|              |
```

**Parameters for authorization request:**

```
GET /authorize?
  response_type=code
  &client_id=CLIENT_ID
  &redirect_uri=https://app.example.com/callback
  &scope=openid profile email
  &state=RANDOM_STATE_VALUE
```

**Token exchange:**

```
POST /token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=AUTHORIZATION_CODE
&redirect_uri=https://app.example.com/callback
&client_id=CLIENT_ID
&client_secret=CLIENT_SECRET
```

## Authorization Code + PKCE

**Use for:** SPAs, mobile apps, native apps — any public client that cannot store secrets.

```
  SPA / Mobile           Auth Server           API
    |                        |                  |
    |-- Generate:            |                  |
    |   code_verifier (random 43-128 chars)     |
    |   code_challenge = BASE64URL(SHA256(code_verifier))
    |                        |                  |
    |-- Redirect to /authorize                  |
    |   + code_challenge     |                  |
    |   + code_challenge_method=S256            |
    |                        |                  |
    |-- User authenticates ->|                  |
    |<-- code=xyz -----------|                  |
    |                        |                  |
    |-- POST /token          |                  |
    |   + code + code_verifier                  |
    |   (NO client_secret)   |                  |
    |<-- access_token -------|                  |
    |                        |                  |
    |-- API call + Bearer token --------------->|
    |<-- Resource -------------------------------|
```

**PKCE generation (JavaScript):**

```javascript
function generateCodeVerifier() {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64urlEncode(array);
}

async function generateCodeChallenge(verifier) {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest("SHA-256", data);
  return base64urlEncode(new Uint8Array(digest));
}
```

## Client Credentials Flow

**Use for:** Service-to-service communication where no user is involved.

```
  Service A              Auth Server           Service B
    |                        |                     |
    |-- POST /token          |                     |
    |   grant_type=client_credentials              |
    |   client_id=SERVICE_A  |                     |
    |   client_secret=SECRET |                     |
    |   scope=read:data      |                     |
    |                        |                     |
    |<-- access_token -------|                     |
    |                                              |
    |-- GET /api/data + Bearer token ------------->|
    |<-- Response ---------------------------------|
```

**Token request:**

```
POST /token
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
&client_id=SERVICE_A_CLIENT_ID
&client_secret=SERVICE_A_SECRET
&scope=read:data
```

Cache the token until near expiry. Do not request a new token per API call.

## Device Authorization Flow

**Use for:** CLIs, smart TVs, IoT devices — limited input capability.

```
  Device                 Auth Server           User's Browser
    |                        |                     |
    |-- POST /device/code -->|                     |
    |<-- device_code,        |                     |
    |    user_code="ABCD-1234"                     |
    |    verification_uri    |                     |
    |                        |                     |
    |-- Display to user:     |                     |
    |   "Go to https://auth.example.com/device"    |
    |   "Enter code: ABCD-1234"                    |
    |                        |                     |
    |                        |    User visits URL and enters code -->|
    |                        |<-- User authorizes ---|
    |                        |                     |
    |-- Poll POST /token     |                     |
    |   grant_type=          |                     |
    |   urn:ietf:params:     |                     |
    |   oauth:grant-type:    |                     |
    |   device_code          |                     |
    |   &device_code=DEVICE_CODE                   |
    |                        |                     |
    |<-- access_token -------|                     |
```

Poll the token endpoint at the interval specified in the device code response.
Handle `authorization_pending` and `slow_down` errors gracefully.

## Token Refresh

**All flows that issue refresh tokens use the same refresh mechanism:**

```
POST /token
Content-Type: application/x-www-form-urlencoded

grant_type=refresh_token
&refresh_token=CURRENT_REFRESH_TOKEN
&client_id=CLIENT_ID
```

**Refresh token rotation** — the server issues a new refresh token with each use
and invalidates the old one. If a revoked refresh token is used, invalidate the
entire token family (indicates token theft).

## Flow Selection Decision Tree

```
Is a user involved?
  |
  +-- No  --> Client Credentials
  |
  +-- Yes
       |
       Can the client store secrets securely?
         |
         +-- Yes (server-side) --> Authorization Code
         |
         +-- No (SPA / mobile / native)
              |
              +-- Auth Code + PKCE
              |
              Does the device have limited input?
                |
                +-- Yes --> Device Authorization
```
