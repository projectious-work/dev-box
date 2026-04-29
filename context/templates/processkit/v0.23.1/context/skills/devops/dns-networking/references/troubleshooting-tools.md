# Troubleshooting Tools

## dig

The primary DNS diagnostic tool. Returns authoritative answers, TTLs, and full resolution details.

### Basic queries

```bash
dig example.com                    # Default A record lookup
dig example.com AAAA               # IPv6 address
dig example.com MX                 # Mail servers
dig example.com TXT                # TXT records (SPF, DKIM, verification)
dig example.com ANY                # All record types (some servers refuse this)
```

### Targeted queries

```bash
dig @8.8.8.8 example.com          # Query specific resolver (Google DNS)
dig @ns1.provider.com example.com  # Query authoritative nameserver directly
dig +trace example.com             # Show full resolution chain from root
dig +short example.com             # Terse output — just the answer
dig +noall +answer example.com     # Clean output with TTL visible
```

### Reverse DNS

```bash
dig -x 93.184.216.34               # PTR lookup for an IP address
```

### Checking zone transfer (AXFR)

```bash
dig @ns1.example.com example.com AXFR   # Full zone transfer (if permitted)
```

## nslookup

Simpler than dig, available on most systems including Windows.

```bash
nslookup example.com               # Default lookup
nslookup example.com 8.8.8.8       # Use specific DNS server
nslookup -type=MX example.com      # Query specific record type
```

Prefer `dig` for scripting and detailed analysis. Use `nslookup` for quick checks.

## traceroute / tracepath

Shows the network path (each router hop) between you and a destination.

```bash
traceroute example.com              # ICMP/UDP trace (may need sudo)
traceroute -T -p 443 example.com   # TCP trace on port 443 (bypasses ICMP blocks)
traceroute -n example.com          # Skip DNS lookups for faster output
tracepath example.com              # No root needed, also discovers MTU
```

### Reading traceroute output

```
 1  gateway (192.168.1.1)      1.234 ms
 2  isp-router (10.0.0.1)     5.678 ms
 3  * * *                               <-- hop not responding (filtered ICMP)
 4  peer.example.net (203.0.113.1)  25.123 ms
```

- **Consistent high latency from a hop onward** -- congestion or distance at that hop
- **`* * *` at one hop but later hops respond** -- that router blocks ICMP, not a problem
- **`* * *` from a hop onward with no recovery** -- traffic is being dropped there
- **Increasing latency** is normal across long distances; sudden jumps indicate congestion

## mtr (My Traceroute)

Combines ping and traceroute into a live, continuously updating display.

```bash
mtr example.com                     # Interactive mode
mtr -rw -c 100 example.com         # Report mode: 100 packets, wide output
mtr -T -P 443 example.com          # TCP mode on port 443
```

Output shows packet loss % and latency stats per hop. Look for loss that persists through subsequent hops (not just at one hop, which may be rate-limited ICMP).

## ping

Basic reachability and latency test.

```bash
ping -c 5 example.com              # Send 5 packets then stop
ping -c 5 -s 1472 example.com     # Test with large packets (MTU issues)
ping -W 2 example.com              # 2-second timeout per packet
ping6 -c 5 example.com            # Force IPv6
```

No response does not always mean the host is down -- ICMP may be blocked.

## curl

### Connection debugging

```bash
curl -v https://example.com         # Verbose: shows DNS, TCP, TLS, HTTP headers
curl -vI https://example.com        # Verbose + HEAD request only (no body)
curl -v --trace-time https://example.com  # Add timestamps to verbose output
```

### DNS and routing control

```bash
# Force resolution to a specific IP (bypass DNS)
curl --resolve example.com:443:93.184.216.34 https://example.com

# Connect to a different host but send correct Host header
curl -H "Host: example.com" http://10.0.0.5:80/

# Force IPv4 or IPv6
curl -4 https://example.com
curl -6 https://example.com
```

### TLS debugging

```bash
curl -vI https://example.com 2>&1 | grep -E 'SSL|TLS|subject|issuer|expire'
curl -k https://self-signed.example.com   # Skip certificate verification
curl --cacert /path/to/ca.pem https://internal.example.com  # Custom CA
```

### Timing breakdown

```bash
curl -o /dev/null -s -w "\
  DNS:        %{time_namelookup}s\n\
  Connect:    %{time_connect}s\n\
  TLS:        %{time_appconnect}s\n\
  First byte: %{time_starttransfer}s\n\
  Total:      %{time_total}s\n" https://example.com
```

This reveals where latency is: DNS resolution, TCP connect, TLS handshake, or server processing.

## ss (Socket Statistics)

Modern replacement for netstat. Shows listening ports and active connections.

```bash
ss -tlnp                            # TCP listeners with process names
ss -ulnp                            # UDP listeners with process names
ss -tlnp sport = :443               # What is listening on port 443?
ss -tn state established            # All established TCP connections
ss -tn dst 10.0.0.5                 # Connections to a specific host
ss -s                               # Summary statistics
```

### netstat (legacy, still common)

```bash
netstat -tlnp                       # TCP listeners (same as ss -tlnp)
netstat -an | grep :80              # All connections on port 80
```

## tcpdump

Packet capture for deep network analysis. Requires root.

```bash
sudo tcpdump -i any port 53 -n                          # Capture DNS traffic
sudo tcpdump -i any host 10.0.0.5 -n                    # Traffic to/from a host
sudo tcpdump -i any port 80 -A -n                       # HTTP traffic, show ASCII
sudo tcpdump -i any -w capture.pcap port 443             # Save to file for Wireshark
sudo tcpdump -i any -c 100 port 8080 -n                 # Limit to 100 packets
sudo tcpdump -i any 'tcp[tcpflags] & (tcp-syn) != 0'    # SYN packets (new connections)
sudo tcpdump -i any 'tcp[tcpflags] & (tcp-rst) != 0'    # RST packets (resets)
```

## openssl

```bash
# Check certificate expiration
openssl s_client -connect example.com:443 < /dev/null 2>/dev/null | openssl x509 -noout -dates
# Check full certificate chain
openssl s_client -connect example.com:443 -showcerts < /dev/null
# Test specific TLS version
openssl s_client -connect example.com:443 -tls1_3
```

## Diagnostic Workflow Summary

| Step | Question                  | Tool                              |
|------|---------------------------|-----------------------------------|
| 1    | Does the name resolve?    | `dig +short example.com`          |
| 2    | Is the host reachable?    | `ping -c 3 <ip>`                  |
| 3    | What path do packets take?| `mtr -rw -c 20 <ip>`             |
| 4    | Is the port open?         | `ss -tlnp sport = :PORT`          |
| 5    | Does the service respond? | `curl -vI https://example.com`    |
| 6    | What is on the wire?      | `sudo tcpdump -i any port PORT`   |
