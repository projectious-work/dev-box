# Protocol Reference

## DNS Record Types

### A Record (Address)

Maps a hostname to an IPv4 address.

```
example.com.    3600  IN  A  93.184.216.34
```

Multiple A records for the same name provide round-robin DNS.

### AAAA Record (IPv6 Address)

Maps a hostname to an IPv6 address.

```
example.com.    3600  IN  AAAA  2606:2800:0220:0001:0248:1893:25c8:1946
```

### CNAME Record (Canonical Name)

Alias from one name to another. The resolver follows the chain to find the final A/AAAA.

```
www.example.com.    3600  IN  CNAME  example.com.
app.example.com.    3600  IN  CNAME  myapp.herokuapp.com.
```

**Restrictions:** Cannot coexist with other record types at the same name. Cannot be used at the zone apex (use ALIAS/ANAME if your provider supports it, or use an A record).

### MX Record (Mail Exchange)

Directs email to mail servers. Lower priority number = tried first.

```
example.com.    3600  IN  MX  10  mail1.example.com.
example.com.    3600  IN  MX  20  mail2.example.com.
```

### TXT Record (Text)

Arbitrary text, commonly used for verification and email security.

```
; SPF — authorize mail senders
example.com.  3600  IN  TXT  "v=spf1 include:_spf.google.com ~all"

; DKIM — email signing
selector._domainkey.example.com.  3600  IN  TXT  "v=DKIM1; k=rsa; p=MIGf..."

; Domain verification
example.com.  3600  IN  TXT  "google-site-verification=abc123..."
```

### SRV Record (Service)

Service discovery with priority, weight, port, and target.

Format: `_service._proto.name TTL IN SRV priority weight port target`

```
_sip._tcp.example.com.  3600  IN  SRV  10 60 5060 sip1.example.com.
_sip._tcp.example.com.  3600  IN  SRV  10 40 5060 sip2.example.com.
```

Weight distributes load among records with equal priority (60/40 split above).

### NS Record (Name Server)

Delegates authority for a zone to specific nameservers.

```
example.com.    86400  IN  NS  ns1.provider.com.
example.com.    86400  IN  NS  ns2.provider.com.
```

### SOA Record (Start of Authority)

Zone metadata. One per zone. Controls zone transfer timing.

```
example.com.  86400  IN  SOA  ns1.provider.com. admin.example.com. (
    2024010101  ; serial (increment on changes, often YYYYMMDDNN)
    3600        ; refresh — secondary checks primary
    900         ; retry — retry after failed refresh
    604800      ; expire — secondary stops serving if no refresh
    300         ; minimum TTL for negative caching
)
```

### PTR Record (Pointer)

Reverse DNS: maps an IP address back to a hostname. Lives in the `in-addr.arpa` (IPv4) or `ip6.arpa` (IPv6) zone.

```
34.216.184.93.in-addr.arpa.  3600  IN  PTR  example.com.
```

Set up via your IP provider (hosting/cloud), not your domain registrar.

## TCP vs UDP

| Property        | TCP                        | UDP                      |
|-----------------|----------------------------|--------------------------|
| Connection      | 3-way handshake (SYN/ACK)  | Connectionless           |
| Reliability     | Guaranteed, ordered         | Best-effort, unordered   |
| Flow control    | Yes (window-based)          | None                     |
| Overhead        | Higher (20-byte header min) | Lower (8-byte header)    |
| Use cases       | HTTP, SSH, SMTP, databases  | DNS, video, VoIP, gaming |

TCP retransmits lost segments. UDP drops them silently -- the application must handle loss.

## HTTP Status Codes

### 2xx Success
- **200 OK** -- Standard success
- **201 Created** -- Resource created (POST/PUT)
- **204 No Content** -- Success with no body (DELETE)

### 3xx Redirection
- **301 Moved Permanently** -- URL changed, update bookmarks (cacheable)
- **302 Found** -- Temporary redirect (not cacheable by default)
- **304 Not Modified** -- Client cache is still valid

### 4xx Client Error
- **400 Bad Request** -- Malformed request syntax
- **401 Unauthorized** -- Authentication required
- **403 Forbidden** -- Authenticated but not authorized
- **404 Not Found** -- Resource does not exist
- **429 Too Many Requests** -- Rate limited; check `Retry-After` header

### 5xx Server Error
- **500 Internal Server Error** -- Generic server failure
- **502 Bad Gateway** -- Upstream server gave invalid response (reverse proxy issue)
- **503 Service Unavailable** -- Server overloaded or in maintenance
- **504 Gateway Timeout** -- Upstream server did not respond in time

## TLS Handshake Overview

**TLS 1.3 (current standard, single round-trip):**

1. **ClientHello** -- Client sends supported cipher suites, key share, SNI (server name)
2. **ServerHello** -- Server selects cipher, sends key share and certificate
3. **Finished** -- Both sides derive session keys; encrypted communication begins

**TLS 1.2 (legacy, two round-trips):**

1. ClientHello -> ServerHello (cipher negotiation)
2. Server sends certificate + key exchange parameters
3. Client verifies certificate chain, sends key material
4. Both sides compute session key, exchange Finished messages

**Common TLS problems:**
- Expired certificate -- check with `openssl s_client -connect host:443 | openssl x509 -noout -dates`
- Missing intermediate CA -- server must send full chain, not just leaf cert
- SNI mismatch -- virtual hosts need correct SNI; test with `curl --resolve host:443:IP https://host/`
- Protocol version -- some clients reject TLS 1.0/1.1; enforce TLS 1.2+ minimum

## Common Port Numbers

| Port   | Protocol    | Description                          |
|--------|-------------|--------------------------------------|
| 22     | SSH         | Secure shell, SCP, SFTP              |
| 25     | SMTP        | Mail transfer (server-to-server)     |
| 53     | DNS         | Domain name resolution (UDP + TCP)   |
| 80     | HTTP        | Unencrypted web traffic              |
| 443    | HTTPS       | TLS-encrypted web traffic            |
| 587    | SMTP        | Mail submission (client-to-server)   |
| 993    | IMAPS       | Encrypted IMAP mail retrieval        |
| 3306   | MySQL       | MySQL/MariaDB database               |
| 5432   | PostgreSQL  | PostgreSQL database                  |
| 6379   | Redis       | Redis key-value store                |
| 8080   | HTTP-alt    | Common dev server / proxy port       |
| 8443   | HTTPS-alt   | Alternative HTTPS port               |
| 27017  | MongoDB     | MongoDB database                     |
