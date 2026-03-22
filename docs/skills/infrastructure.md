# Infrastructure Skills

Skills for containers, orchestration, networking, system administration, and CI/CD.

---

### dockerfile-review

> Dockerfile best practices review -- layer optimization, caching, security, image size. Use when writing or reviewing Dockerfiles.

**Triggers:** When the user asks to review a Dockerfile, optimize an image, or says "why is my image so big?", "is this Dockerfile correct?", or "help me with Docker".
**Tools:** None
**References:** None

Key capabilities:

- Layer optimization: combine related `RUN` commands, order from least to most frequently changing
- Caching: copy dependency manifests first, install, then copy source
- Security: don't run as root, never `COPY` secrets, pin base images with digest, remove package caches
- Size reduction: slim/alpine base images, multi-stage builds, `--no-install-recommends`
- Correctness: use `COPY` over `ADD`, set `WORKDIR` instead of `cd`, exec form for `CMD`/`ENTRYPOINT`

??? example "Example usage"
    User says "Review my Dockerfile." The agent reads it and identifies that dependency installation and source copy are in the same layer (cache-busting), apt lists aren't cleaned up, and the container runs as root. Provides specific fixes for each issue.

---

### ci-cd-setup

> CI/CD pipeline setup -- GitHub Actions, testing, linting, deployment. Use when setting up or improving continuous integration and deployment.

**Triggers:** When the user asks to "set up CI", "add GitHub Actions", "automate tests", "add deployment", or wants to improve their build pipeline.
**Tools:** None
**References:** None

Key capabilities:

- Pipeline stages in order: lint (fastest feedback), test (unit then integration), build, deploy
- GitHub Actions basics: trigger on push/PR, specific action versions, cache dependencies, set timeouts
- Testing in CI: same commands as local dev, matrix builds when needed, fail fast
- Security: use GitHub Secrets, `permissions` key for token scope, pin third-party actions to SHA
- Best practices: keep CI under 5 minutes for PRs, require CI pass before merge, run expensive checks only on main

??? example "Example usage"
    User asks to set up CI for a Rust project. The agent creates `.github/workflows/ci.yml` with lint (clippy), test (cargo test), and build steps, with cargo caching for faster runs.

---

### kubernetes-basics

> Kubernetes cluster management, resource definitions, networking, storage, Helm, and troubleshooting. Use when working with Kubernetes manifests, kubectl commands, Helm charts, or debugging pod/service issues.

**Triggers:** When writing or editing Kubernetes YAML manifests, running kubectl or helm commands, debugging pods/services/networking/storage, or managing Helm charts.
**Tools:** `Bash(kubectl:*)`, `Bash(helm:*)`, `Bash(k9s:*)`, `Read`, `Write`
**References:** `references/resource-cheatsheet.md`, `references/cluster-architecture.md`, `references/troubleshooting.md`

Key capabilities:

- Cluster context management: confirm active cluster and namespace before changes
- Core resources: Deployments (stateless), StatefulSets (stateful), DaemonSets (per-node), Jobs/CronJobs
- Networking: ClusterIP, NodePort, LoadBalancer services; Ingress for HTTP routing; Network Policies
- Storage: PersistentVolumes, PersistentVolumeClaims, StorageClasses, `volumeClaimTemplates`
- Configuration: ConfigMaps and Secrets, prefer volume mounts over env vars
- Helm: repo management, install/upgrade/rollback, `helm template` for inspection, pin chart versions
- Troubleshooting workflow: events, describe, logs, exec, top
- Safe changes: `kubectl diff` before `apply`, `--dry-run=client` for validation, `rollout undo` for rollback

??? example "Example usage"
    User has a pod stuck in CrashLoopBackOff. The agent runs `kubectl describe pod` to check events for OOM or probe failures, `kubectl logs --previous` to see logs from the crashed container, and inspects the pod YAML for resource limits and command issues.

---

### dns-networking

> DNS resolution, IP addressing, subnetting, network protocols, and diagnostic tools. Use when configuring DNS records, debugging connectivity, setting up networking, or troubleshooting network issues.

**Triggers:** When setting up or modifying DNS records, debugging DNS resolution or connectivity, configuring firewalls, analyzing HTTP/TLS issues, calculating subnets, or diagnosing latency and routing problems.
**Tools:** `Bash(dig:*)`, `Bash(nslookup:*)`, `Bash(traceroute:*)`, `Bash(curl:*)`, `Bash(ss:*)`, `Read`, `Write`
**References:** `references/protocol-reference.md`, `references/troubleshooting-tools.md`

Key capabilities:

- DNS fundamentals: record types (A, AAAA, CNAME, MX, TXT, SRV, NS, SOA, PTR), TTL management
- IP addressing and subnetting: CIDR notation, private ranges (RFC 1918), quick subnet math
- Common protocols: TCP vs UDP, HTTP/HTTPS, DNS, SSH, SMTP/IMAP
- TLS and HTTPS: handshake process, debugging expired certificates and hostname mismatches
- Port management: listing listeners with `ss`, well-known vs ephemeral port ranges
- Firewall basics with `ufw` and `iptables`
- Load balancing concepts: round-robin DNS, reverse proxy (Layer 7), Layer 4 LB
- Diagnostic workflow: DNS resolution, reachability, route tracing, port checks, application testing

??? example "Example usage"
    User reports a DNS record not propagating after a change. The agent checks the current authoritative answer vs cached answer with `dig @ns1.provider.com` and `dig @8.8.8.8`, runs `dig +trace` for the full resolution chain, and advises waiting for the old TTL to expire if the authoritative server shows the new value.

---

### terraform-basics

> Infrastructure-as-code with Terraform/OpenTofu. Resources, providers, state, modules, and plan/apply workflow. Use when writing Terraform configs, managing cloud infrastructure, or reviewing IaC code.

**Triggers:** When writing or editing `.tf` files, planning/applying/destroying infrastructure, managing state and backends, creating modules, or reviewing IaC for best practices.
**Tools:** `Bash(terraform:*)`, `Bash(tofu:*)`, `Read`, `Write`
**References:** None

Key capabilities:

- Resources and providers: pin provider versions in `required_providers`, descriptive resource names
- Variables with types, descriptions, and validation; outputs with descriptions
- Data sources to reference existing infrastructure without managing it
- State management: remote backends (S3 + DynamoDB), never edit state manually, `terraform import`
- Modules for reuse: focused on single concern, pin versions in production
- Plan/apply/destroy workflow: `init`, `fmt`, `validate`, `plan -out`, `apply`
- Best practices: one state per environment, `prevent_destroy` on critical resources, tag all resources, `moved` blocks for refactoring

??? example "Example usage"
    User needs to provision an EC2 instance. The agent writes Terraform config with a security group, AMI data source, typed variables, and outputs. Uses `terraform plan -out=tfplan` followed by `terraform apply tfplan` for safe deployment.

---

### container-orchestration

> Docker Compose patterns for multi-service architectures. Health checks, networking, volumes, and service dependencies. Use when designing docker-compose files, debugging container networking, or managing multi-container applications.

**Triggers:** When writing Docker Compose files, designing multi-service architectures, debugging container networking, setting up health checks, managing volumes, or configuring environment variables.
**Tools:** `Bash(docker:*)`, `Bash(docker-compose:*)`, `Read`, `Write`
**References:** `references/compose-patterns.md`

Key capabilities:

- Compose file structure: services, networks, and volumes at top level
- Build vs image: `build` for developed services, `image` for third-party
- Health checks with `depends_on: condition: service_healthy` for proper startup ordering
- Networking: custom networks for traffic isolation, service name as hostname, `expose` vs `ports`
- Volume strategies: named volumes (persistent), bind mounts (dev), tmpfs (sensitive/cache)
- Environment management: `.env` files with overrides, never put secrets in `docker-compose.yml`
- Profiles for optional services (e.g., debug tools)
- Common commands: `up -d`, `logs -f`, `exec`, `down -v`, `config` for validation

??? example "Example usage"
    User has a service that cannot connect to the database. The agent checks both services are on the same network, verifies the db is healthy with `docker compose exec db pg_isready`, tests connectivity from the app container, and inspects environment variables for correct database host configuration.

---

### linux-administration

> Essential Linux system administration for developers. File permissions, process management, systemd, journald, cron, and disk management. Use when managing Linux servers, debugging system issues, or writing system scripts.

**Triggers:** When managing file permissions, investigating processes, creating systemd services, querying logs with journalctl, setting up cron jobs, diagnosing disk issues, managing users, or installing packages.
**Tools:** `Bash`, `Read`, `Write`
**References:** `references/commands-cheatsheet.md`, `references/systemd-reference.md`

Key capabilities:

- File permissions and ownership: rwx model, numeric and symbolic notation, special bits (setuid, setgid, sticky)
- Process management: `ps aux`, `pgrep`, `kill` with SIGTERM before SIGKILL, `lsof`, `ss`
- User and group management: `useradd`, `usermod -aG`, prefer `useradd` for scripting
- Systemd services: create unit files, `systemctl` commands, always `daemon-reload` after edits
- Journald logging: `journalctl -u`, filter by time and priority, manage journal size
- Cron jobs: crontab format, common schedules, always redirect output
- Disk and filesystem management: `df -h`, `du -sh`, `lsblk`, emergency disk space cleanup
- Package management with apt (Debian/Ubuntu) and dnf (RHEL/Fedora)

??? example "Example usage"
    User needs to diagnose high disk usage on a server. The agent runs `df -h` to check filesystem usage, `du -sh /* | sort -rh` to find the largest directories, drills into the biggest directory, and cleans up old logs with `journalctl --vacuum-size` and `apt autoremove`.

---

### shell-scripting

> Bash scripting best practices including error handling, argument parsing, and shellcheck compliance. Use when writing shell scripts, reviewing bash code, or automating tasks with shell commands.

**Triggers:** When writing new shell scripts, reviewing bash code, adding error handling, parsing command-line arguments, or fixing shellcheck warnings.
**Tools:** `Bash(shellcheck:*)`, `Bash(bash:*)`, `Read`, `Write`
**References:** `references/bash-patterns.md`

Key capabilities:

- Script header: shebang (`#!/usr/bin/env bash`) and strict mode (`set -euo pipefail`)
- Variable quoting: always double-quote expansions, `${var:-default}` for defaults, `${var:?error}` for required
- Argument parsing with positional args and `getopts` for options
- Functions with `local` variables, return values via stdout
- Error handling and cleanup with `trap cleanup EXIT`
- Arrays for safe file path handling with `find -print0` and `read -d ''`
- Common pitfalls: never parse `ls`, use `[[ ]]` over `[ ]`, `$(command)` over backticks
- Shellcheck compliance: run on every script, disable warnings locally with comments, never globally

??? example "Example usage"
    User needs a script with argument parsing. The agent writes a script with shebang, strict mode, a usage function, `getopts` for options, input validation, and a trap for temp file cleanup on exit.

---

### dependency-audit

> Audits project dependencies for vulnerabilities and outdated packages. Use when checking security posture or planning dependency updates.

**Triggers:** When the user asks to "check dependencies", "audit security", "update packages", "are my dependencies safe?", or before a release to verify dependency health.
**Tools:** None
**References:** None

Key capabilities:

- Identify the package manager and run its audit tool: `cargo audit`, `pip-audit`, `npm audit`, `govulncheck`
- Review findings by severity: critical/high (fix immediately), medium (plan for current sprint), low (track)
- Update strategy: one dependency at a time, full test suite after each, check changelogs for breaking changes
- Check for outdated packages: `cargo outdated`, `pip list --outdated`, `npm outdated`
- Ongoing maintenance: monthly reviews, Dependabot/Renovate for automated PRs, document pinned versions

??? example "Example usage"
    User asks "Are my dependencies secure?" The agent runs the appropriate audit tool, summarizes findings by severity, and recommends specific version bumps for vulnerable packages. Flags any dependencies with no maintained alternatives.

---

### secret-management

> Guides secure handling of secrets -- env vars, .env files, vault patterns. Use when dealing with API keys, passwords, tokens, or credentials.

**Triggers:** When the user needs to handle API keys, passwords, tokens, database credentials, or asks "where should I put this secret?", "is this safe?", or "how do I manage credentials?".
**Tools:** None
**References:** None

Key capabilities:

- Never commit secrets to git: add `.env` to `.gitignore`, check history for leaked secrets, rotate if compromised
- Local development: `.env` files with dotenv pattern, `.env.example` with placeholder values committed
- CI/CD: platform secret stores (GitHub Secrets, GitLab CI Variables), OIDC tokens over long-lived credentials
- Production: secrets managers (Vault, AWS Secrets Manager), rotate on 90-day schedule, short-lived tokens, least privilege
- Code patterns: read from environment variables, never hardcode, never log secrets, separate secrets per environment

??? example "Example usage"
    User needs to add an API key for the payment provider. The agent adds `PAYMENT_API_KEY=` to `.env.example`, updates `.gitignore` to include `.env`, reads the key from `os.environ["PAYMENT_API_KEY"]` in code, and documents the required variable.
