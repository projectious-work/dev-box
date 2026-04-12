---
name: dns-networking
description: |
  DNS records, IP addressing, subnetting, common protocols, and diagnostic tools. Use when configuring DNS records, debugging resolution or connectivity, planning subnets, analyzing HTTPS/TLS errors, or diagnosing latency and routing issues.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-dns-networking
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
---

# DNS & Networking

## Intro

Networking problems resolve bottom-up: name, reachability, route, port,
application. Know which record type, which protocol, and which tool
answers each question, and most incidents collapse to a handful of
`dig`/`ss`/`curl` commands.

## Overview

### DNS fundamentals

DNS maps names to addresses through a hierarchical chain: stub
resolver, recursive resolver, root, TLD, authoritative server. The
record types you need day-to-day:

- **A / AAAA** — IPv4 / IPv6 address mapping
- **CNAME** — alias to another name (cannot coexist with other records
  at the same name; cannot be used at the zone apex)
- **MX** — mail server with priority (lower = higher priority)
- **TXT** — arbitrary text; used for SPF, DKIM, domain verification
- **SRV** — service discovery (priority, weight, port, target)
- **NS** — delegates a zone to nameservers
- **SOA** — zone metadata (primary NS, serial, refresh/retry/expire)
- **PTR** — reverse DNS (IP back to hostname)

**TTL:** time-to-live in seconds. Lower the TTL before a migration
(300s), raise it after (3600–86400s). Always check the current TTL
before making changes: `dig +noall +answer example.com A`.

### IP addressing and subnetting

CIDR notation: `192.168.1.0/24` means a 24-bit network prefix, leaving
8 bits for hosts (254 usable after subtracting network and broadcast).

Private ranges (RFC 1918):

- `10.0.0.0/8` — 16M addresses; large networks and cloud VPCs
- `172.16.0.0/12` — 1M addresses; Docker default bridge range
- `192.168.0.0/16` — 65K addresses; home and office networks

Quick subnet math: /24 = 256, /25 = 128, /26 = 64, /27 = 32, /28 = 16
(subtract 2 for usable hosts).

### Common protocols

TCP gives reliable ordered delivery with a 3-way handshake. UDP is
connectionless and cheap — good when speed matters more than
guaranteed delivery.

- **HTTP / HTTPS** — TCP 80 / 443; HTTPS adds TLS
- **DNS** — UDP 53 for queries, TCP 53 for zone transfers and large
  responses
- **SSH** — TCP 22 (key-based auth preferred)
- **SMTP / IMAP** — TCP 25/587, 143/993

### TLS and HTTPS

Handshake order: ClientHello (supported ciphers, SNI), ServerHello
(chosen cipher, certificate), key exchange, encrypted session. Common
failure modes are expired certs, hostname mismatch, missing
intermediate CA, and protocol version mismatch. Debug with
`curl -vI https://example.com` or
`openssl s_client -connect example.com:443`.

### Port management

```bash
ss -tlnp                       # TCP listeners with process names
ss -ulnp                       # UDP listeners with process names
ss -tlnp sport = :443          # what is listening on 443?
```

Well-known ports: 0–1023 (root). Registered: 1024–49151. Ephemeral:
49152–65535.

### Firewall basics

```bash
# ufw (Ubuntu/Debian)
ufw allow 80/tcp
ufw allow from 10.0.0.0/8 to any port 5432
ufw status numbered

# iptables (direct)
iptables -A INPUT -p tcp --dport 443 -j ACCEPT
iptables -A INPUT -s 192.168.1.0/24 -j ACCEPT
iptables -L -n --line-numbers
```

Always allow SSH (port 22) before enabling a firewall. Test rules
before persisting.

### Load balancing concepts

- **Round-robin DNS** — multiple A records, no health checking
- **Reverse proxy** (nginx/HAProxy) — L7, routes by path/header,
  health-checks backends
- **Layer 4 LB** — TCP-level forwarding, lowest overhead, no content
  inspection

Health checks should verify application readiness, not just TCP
connectivity.

### Diagnostic workflow

When something does not work, go bottom-up:

1. **Name** — `dig example.com` — does it resolve?
2. **Reachability** — `ping -c 3 <ip>` — is the host up?
3. **Route** — `traceroute <ip>` — where does the path break?
4. **Port** — `curl -v telnet://<ip>:<port>` — is the port open?
5. **Application** — `curl -vI https://example.com` — does the service
   respond correctly?

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **Not lowering TTL before a DNS migration.** Changing an A record with a 24-hour TTL means clients that cached the old value before the change will continue using the old IP for up to 24 hours. Lower the TTL to 300 seconds at least an hour before the change, complete the migration, verify, then raise it back. Skipping this step makes rollback window unpredictably long.
- **Using a CNAME at the zone apex.** A CNAME at the zone root (`example.com`, not `www.example.com`) conflicts with SOA and MX records and violates the DNS specification — many resolvers will silently drop one or the other. Use an ALIAS or ANAME record if your DNS provider supports it, or an A record pointing at a load balancer IP for the apex.
- **Diagnosing DNS issues from a cached resolver instead of the authoritative server.** `dig example.com` queries your default recursive resolver which may return a cached (and outdated) answer. Always compare `dig @8.8.8.8 example.com` (public cache) against `dig @ns1.provider.com example.com` (authoritative) to know what the actual current record is versus what is cached.
- **Enabling a firewall without first allowing SSH.** Adding a firewall rule that blocks all inbound traffic before ensuring port 22 (or the configured SSH port) is explicitly allowed will lock you out of the server. Always verify SSH access persists after adding firewall rules before saving them permanently.
- **Assuming `* * *` in a traceroute means the host is down.** A hop that shows `* * *` means that router rate-limits or drops ICMP TTL-exceeded messages — it is not forwarding the probe, but it may still be forwarding actual application traffic. Only conclude the path is broken if `* * *` continues from that hop onward with no subsequent hops responding.
- **Testing TCP port reachability from outside the server without checking whether the service is listening inside.** A port scan from an external client failing does not distinguish between "service not listening", "firewall blocking", or "routing problem". Add a check from inside the server itself (`ss -tlnp sport = :<port>`) to confirm the service is bound before concluding the issue is network-related.
- **Diagnosing TLS errors without checking the full certificate chain.** A TLS error may be caused by an expired leaf certificate, a missing intermediate CA certificate, or a hostname mismatch — all of which look similar in a browser error. Use `openssl s_client -connect host:443 -showcerts` to see the full chain, not just the end-entity certificate, so you know which certificate in the chain is the problem.

## Full reference

### DNS record syntax

```
; A and AAAA
example.com.    3600  IN  A     93.184.216.34
example.com.    3600  IN  AAAA  2606:2800:220:1:248:1893:25c8:1946

; CNAME
www.example.com.  3600  IN  CNAME  example.com.

; MX (lower priority tried first)
example.com.  3600  IN  MX  10  mail1.example.com.
example.com.  3600  IN  MX  20  mail2.example.com.

; TXT (SPF, DKIM, verification)
example.com.  3600  IN  TXT  "v=spf1 include:_spf.google.com ~all"
selector._domainkey.example.com.  3600  IN  TXT  "v=DKIM1; k=rsa; p=MIGf..."

; SRV: _service._proto.name TTL IN SRV priority weight port target
_sip._tcp.example.com.  3600  IN  SRV  10 60 5060 sip1.example.com.

; SOA
example.com.  86400  IN  SOA  ns1.provider.com. admin.example.com. (
    2024010101  ; serial
    3600        ; refresh
    900         ; retry
    604800      ; expire
    300         ; negative cache TTL
)

; PTR (reverse DNS, lives in in-addr.arpa)
34.216.184.93.in-addr.arpa.  3600  IN  PTR  example.com.
```

PTR records must be set at your IP provider (hosting/cloud), not your
domain registrar. CNAMEs cannot live at the zone apex — use ALIAS/ANAME
if your provider supports it, or a plain A record.

### TCP vs UDP at a glance

| Property     | TCP                         | UDP                      |
|--------------|-----------------------------|--------------------------|
| Connection   | 3-way handshake (SYN/ACK)   | Connectionless           |
| Reliability  | Guaranteed, ordered         | Best-effort, unordered   |
| Flow control | Yes (window-based)          | None                     |
| Overhead     | Higher (20-byte header min) | Lower (8-byte header)    |
| Use cases    | HTTP, SSH, SMTP, databases  | DNS, video, VoIP, gaming |

### HTTP status codes worth remembering

| Code | Meaning |
|---|---|
| 200 | OK |
| 201 | Created (POST/PUT) |
| 204 | No Content (DELETE) |
| 301 | Moved Permanently (cacheable) |
| 302 | Found (temporary redirect) |
| 304 | Not Modified (client cache valid) |
| 400 | Bad Request (malformed syntax) |
| 401 | Unauthorized (authentication required) |
| 403 | Forbidden (authenticated but not authorized) |
| 404 | Not Found |
| 429 | Too Many Requests (check `Retry-After`) |
| 500 | Internal Server Error |
| 502 | Bad Gateway (upstream invalid response) |
| 503 | Service Unavailable |
| 504 | Gateway Timeout (upstream did not respond) |

### TLS handshake (1.3 vs 1.2)

- **TLS 1.3** — single round-trip: ClientHello → ServerHello (cipher,
  key share, cert) → Finished. Encrypted communication begins.
- **TLS 1.2** — two round-trips: ClientHello → ServerHello, then cert
  + key exchange, client verification, Finished.

Common TLS problems and checks:

```bash
# Expiration and dates
openssl s_client -connect host:443 < /dev/null 2>/dev/null \
  | openssl x509 -noout -dates
# Full chain (verify intermediates are sent)
openssl s_client -connect host:443 -showcerts < /dev/null
# Force a specific version
openssl s_client -connect host:443 -tls1_3
# SNI virtual-host workaround
curl --resolve host:443:IP https://host/
```

Enforce TLS 1.2+ minimum — 1.0/1.1 are obsolete.

### `dig` recipes

```bash
dig example.com                      # default A lookup
dig example.com AAAA                 # IPv6
dig example.com MX                   # mail servers
dig example.com TXT                  # SPF/DKIM/verification
dig @8.8.8.8 example.com             # query a specific resolver
dig @ns1.provider.com example.com    # query authoritative directly
dig +trace example.com               # full resolution chain from root
dig +short example.com               # terse answer
dig +noall +answer example.com       # clean output with TTL
dig -x 93.184.216.34                 # reverse DNS lookup
dig @ns1.example.com example.com AXFR  # zone transfer (if permitted)
```

### Path and reachability tools

```bash
# traceroute / tracepath
traceroute example.com
traceroute -T -p 443 example.com     # TCP trace (bypasses ICMP blocks)
traceroute -n example.com            # skip DNS lookups
tracepath example.com                # no root needed, discovers MTU

# mtr — live traceroute + packet loss stats
mtr example.com
mtr -rw -c 100 example.com           # report mode: 100 pkts, wide output
mtr -T -P 443 example.com            # TCP mode on port 443

# ping
ping -c 5 example.com
ping -c 5 -s 1472 example.com        # test large packets (MTU issues)
```

Reading traceroute: `* * *` at a single hop often means that router
rate-limits ICMP and is not a real problem. `* * *` from a hop onward
with no recovery means traffic is being dropped there. Increasing
latency with distance is normal; sudden jumps indicate congestion.

### `curl` debugging recipes

```bash
# Connection debugging
curl -v https://example.com                  # DNS, TCP, TLS, headers
curl -vI https://example.com                 # verbose + HEAD only
curl -v --trace-time https://example.com     # add timestamps

# DNS/routing control
curl --resolve example.com:443:93.184.216.34 https://example.com
curl -H "Host: example.com" http://10.0.0.5:80/
curl -4 https://example.com                  # force IPv4
curl -6 https://example.com                  # force IPv6

# TLS debugging
curl -vI https://example.com 2>&1 | grep -E 'SSL|TLS|subject|issuer|expire'
curl -k https://self-signed.example.com      # skip verification
curl --cacert /path/ca.pem https://internal  # custom CA

# Timing breakdown
curl -o /dev/null -s -w "\
  DNS:        %{time_namelookup}s\n\
  Connect:    %{time_connect}s\n\
  TLS:        %{time_appconnect}s\n\
  First byte: %{time_starttransfer}s\n\
  Total:      %{time_total}s\n" https://example.com
```

### Socket statistics and packet capture

```bash
# ss (modern netstat replacement)
ss -tlnp                             # TCP listeners with processes
ss -ulnp                             # UDP listeners
ss -tlnp sport = :443                # who owns port 443?
ss -tn state established             # established TCP connections
ss -tn dst 10.0.0.5                  # connections to a host
ss -s                                # summary statistics

# tcpdump (requires root)
sudo tcpdump -i any port 53 -n                       # DNS traffic
sudo tcpdump -i any host 10.0.0.5 -n                 # host traffic
sudo tcpdump -i any port 80 -A -n                    # HTTP, ASCII dump
sudo tcpdump -i any -w capture.pcap port 443         # save for Wireshark
sudo tcpdump -i any 'tcp[tcpflags] & (tcp-syn) != 0' # SYN packets
sudo tcpdump -i any 'tcp[tcpflags] & (tcp-rst) != 0' # RST packets
```

### Diagnostic workflow summary

| Step | Question                   | Tool                             |
|------|----------------------------|----------------------------------|
| 1    | Does the name resolve?     | `dig +short example.com`         |
| 2    | Is the host reachable?     | `ping -c 3 <ip>`                 |
| 3    | What path do packets take? | `mtr -rw -c 20 <ip>`             |
| 4    | Is the port open?          | `ss -tlnp sport = :PORT`         |
| 5    | Does the service respond?  | `curl -vI https://example.com`   |
| 6    | What is on the wire?       | `sudo tcpdump -i any port PORT`  |

### Common scenarios

**DNS record not propagating after change.** Compare authoritative
answer vs cached answer; verify the change actually reached the
authoritative server; then wait for the old TTL to expire:

```bash
dig @8.8.8.8 example.com A              # public resolver (may be cached)
dig @ns1.provider.com example.com A     # authoritative (should be new)
dig +trace example.com A                # full resolution chain
```

**Service unreachable on a specific port.** Isolate DNS vs network vs
application:

```bash
dig api.example.com                      # verify resolution
curl -v telnet://api.example.com:443     # TCP connectivity
curl -vI https://api.example.com         # full HTTP/TLS stack
ss -tlnp sport = :443                    # on the server: is it listening?
```

**DNS + TLS for a new subdomain.** Add the record, verify, then
issue the certificate:

```bash
dig +short staging.example.com           # verify resolution
curl -I http://staging.example.com       # HTTP works before TLS
certbot certonly --nginx -d staging.example.com
curl -vI https://staging.example.com     # verify HTTPS
```
