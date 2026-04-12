---
name: linux-administration
description: |
  Linux sysadmin for developers — permissions, processes, systemd, journald, cron, disks. Use when managing Linux servers, writing systemd units, querying journalctl, debugging disk or process issues, or wiring up cron and timers.
metadata:
  processkit:
    apiVersion: processkit.projectious.work/v1
    id: SKILL-linux-administration
    version: "1.1.0"
    created: 2026-04-06T00:00:00Z
    category: devops
---

# Linux Administration

## Intro

Day-to-day Linux admin comes down to a small set of tools: `chmod`,
`ps`/`ss`, `systemctl`/`journalctl`, `cron` or timers, and `df`/`du`.
Get comfortable with each and most production incidents resolve with
a handful of commands.

## Overview

### File permissions and ownership

Linux uses the rwx model for owner, group, and others. Use numeric
(`chmod 755`) or symbolic (`chmod u+x`) notation:

```bash
chmod 755 script.sh          # rwxr-xr-x
chmod u+x,g+r,o-w file       # symbolic mode
chown app:app /srv/data      # change owner and group
chown -R app:app /srv/data   # recursive
```

Key values: 644 (files, rw-r--r--), 755 (executables/dirs), 600
(secrets, owner-only), 700 (private dirs). Directories need execute
permission for traversal. Special bits: setuid (4000), setgid (2000),
sticky (1000). Inspect concisely with `stat -c '%a %U:%G %n' file`.

### Process management

```bash
ps aux                         # all processes
ps aux --sort=-%mem            # sort by memory
pgrep -af "pattern"            # find by name
kill -TERM <pid>               # graceful termination
kill -KILL <pid>               # force kill (last resort)
top -b -n 1 -o %MEM            # batch snapshot sorted by memory
```

Send `SIGTERM` (15) first; reach for `SIGKILL` (9) only when the
process refuses to exit. Check open files with `lsof -p <pid>` and
listening ports with `ss -tlnp`.

### User and group management

```bash
useradd -m -s /bin/bash -G sudo newuser  # create user with home dir
usermod -aG docker existinguser           # append to group (keep existing)
passwd username                           # set password
userdel -r username                       # remove user and home dir
groups username                           # list user's groups
id username                               # uid, gid, groups
```

Prefer `useradd` over `adduser` for scripts (consistent across
distros). Always use `-aG` (append) with `usermod` — without `-a` you
replace the user's entire group list.

### Systemd services

Create unit files in `/etc/systemd/system/`. Always run
`daemon-reload` after editing:

```bash
systemctl start myapp
systemctl stop myapp
systemctl restart myapp
systemctl enable myapp            # start on boot
systemctl enable --now myapp      # enable and start immediately
systemctl status myapp            # status with recent logs
systemctl daemon-reload           # reload after editing unit files
```

### Journald

```bash
journalctl -u myapp -f                       # follow service logs
journalctl -u myapp --since "1 hour ago"     # recent
journalctl -u myapp -p err                   # errors and above
journalctl -b -1                             # previous boot
journalctl --disk-usage                      # journal size
journalctl --vacuum-size=500M                # reclaim space
```

### Cron

Edit with `crontab -e`. Format: `minute hour day month weekday command`.

```cron
0 * * * *    /usr/local/bin/hourly-task.sh
0 2 * * *    /usr/local/bin/nightly-backup.sh
*/5 * * * *  /usr/local/bin/health-check.sh
0 0 * * 0    /usr/local/bin/weekly-cleanup.sh
```

Always redirect cron output: `command >> /var/log/task.log 2>&1`.
Consider systemd timers as a modern alternative.

### Disk and filesystem

```bash
df -h                    # filesystem usage
du -sh /var/log/*        # directory sizes
lsblk                    # block device tree
mount | column -t        # current mounts
findmnt --target /data   # find mount point for a path
```

For disk-space emergencies: check `/var/log`, `/tmp`, old kernels
(`apt autoremove`), Docker (`docker system prune`), and journal logs
(`journalctl --vacuum-size`).

### Package management

**Debian/Ubuntu (apt):**

```bash
apt update && apt upgrade -y
apt install -y --no-install-recommends package
apt search keyword
apt autoremove -y
dpkg -l | grep package
```

**RHEL/Fedora (dnf):**

```bash
dnf check-update
dnf install -y package
dnf autoremove
```

## Gotchas

Agent-specific failure modes — provider-neutral pause-and-self-check items:

- **`usermod -G` without `-a` replaces the entire group list.** `usermod -G docker username` removes the user from every group except `docker` — including `sudo`. Always use `usermod -aG docker username` (append). Losing `sudo` access on a remote server that requires sudo for SSH key management can lock you out.
- **Editing unit files in `/lib/systemd/system/` instead of `/etc/systemd/system/`.** Files in `/lib/systemd/system/` are owned by the package manager and will be overwritten on the next package upgrade. Override units by copying them to `/etc/systemd/system/` or by creating a drop-in file at `/etc/systemd/system/unit.d/override.conf`. Admin files in `/etc/` always win.
- **Forgetting `systemctl daemon-reload` after editing a unit file.** systemd caches unit file contents. Editing a `.service` file and then restarting the service with `systemctl restart` will restart the service using the old cached definition. Always run `daemon-reload` after any unit file change.
- **`kill -9` (SIGKILL) as the first option.** SIGKILL cannot be caught or ignored by the process, so it terminates immediately without flushing buffers, releasing file locks, or running shutdown hooks. This can corrupt data or leave stale lock files. Always try `kill -TERM <pid>` (graceful) first; wait a few seconds; use SIGKILL only if the process refuses to exit.
- **Not redirecting cron job output.** A cron job that produces output sends it to the local mail spool by default, which fills up silently on systems without a mail server configured. Add `>> /var/log/jobname.log 2>&1` to every cron job so output is captured to a file and errors are included. Alternatively use `systemd` timers which capture output to the journal automatically.
- **`rm -rf /var/log/*` to free disk space.** Deleting log files while the daemons that own them are running does not actually free disk space — the file descriptors remain open and the blocks are still allocated until the process closes them. The correct approach is log rotation (`logrotate`) or `journalctl --vacuum-size` for journal logs. Files that must be cleared immediately can be truncated with `> /var/log/file.log` while the daemon continues writing.
- **Setting permissions with `chmod 777` to fix a permission problem.** `chmod 777` (world-readable and world-writable) is almost never the correct fix and makes files accessible to every user and every process on the system. Diagnose which user and group actually need access, set `chown` to the correct owner, and use `chmod 640` or `chmod 755` as appropriate. `777` is a security problem, not a solution.

## Full reference

### Systemd unit file reference

```ini
[Unit]
Description=My Application Service
Documentation=https://example.com/docs
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=app
Group=app
WorkingDirectory=/srv/myapp
ExecStart=/srv/myapp/bin/server --config /etc/myapp/config.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5
TimeoutStartSec=30
TimeoutStopSec=30

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/srv/myapp/data /var/log/myapp
PrivateTmp=yes

# Environment
Environment=NODE_ENV=production
EnvironmentFile=/etc/myapp/env

[Install]
WantedBy=multi-user.target
```

**Service types:**

- `simple` (default) — ExecStart is the main process
- `exec` — like simple, but systemd waits for the binary to execute
- `forking` — process forks and parent exits (use `PIDFile=`)
- `oneshot` — runs once and exits (combine with `RemainAfterExit=yes`)
- `notify` — process sends `sd_notify` when ready
- `idle` — delayed until other jobs finish

**Restart policies:** `no`, `on-failure`, `on-abnormal`, `always`,
`on-success`. Use `RestartSec=` for delay between restarts and
`StartLimitIntervalSec=` / `StartLimitBurst=` to prevent restart
loops.

**Dependencies:** `After=`/`Before=` for ordering only, `Requires=`
for hard dependencies, `Wants=` for soft dependencies, `BindsTo=` to
stop when the dependency stops.

### Systemd commands

```bash
systemctl status myapp             # status with recent logs
systemctl show myapp               # all properties
systemctl cat myapp                # show unit file content
systemctl list-dependencies myapp  # dependency tree
systemctl is-active myapp          # for scripts
systemctl is-enabled myapp
systemctl is-failed myapp
systemctl list-units --failed      # show failed units
systemctl list-timers              # show all timers
systemctl list-units --type=service --state=running
```

### Journalctl patterns

```bash
journalctl -u myapp --since "2024-01-15 09:00" --until "2024-01-15 10:00"
journalctl -u myapp -p warning..err  # warnings through errors
journalctl -u myapp -o json-pretty   # structured JSON
journalctl -u myapp -o short-precise # precise timestamps
journalctl -u myapp -o cat           # message only
journalctl -b                        # current boot
journalctl --list-boots              # all recorded boots
journalctl -k                        # kernel messages
journalctl --vacuum-time=30d         # keep only last 30 days
```

### Systemd timers (cron alternative)

```ini
# /etc/systemd/system/backup.timer
[Unit]
Description=Run backup nightly

[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true
RandomizedDelaySec=300

[Install]
WantedBy=timers.target
```

```ini
# /etc/systemd/system/backup.service
[Unit]
Description=Backup job

[Service]
Type=oneshot
ExecStart=/usr/local/bin/backup.sh
User=backup
```

Calendar syntax examples:

```
*-*-* 02:00:00          # daily at 2 AM
Mon *-*-* 09:00:00      # every Monday at 9 AM
*-*-01 00:00:00         # first day of every month
*-*-* *:00/15:00        # every 15 minutes
hourly / daily / weekly # shorthands
```

Verify expressions with `systemd-analyze calendar "Mon *-*-* 09:00:00"`.
Enable with `systemctl enable --now backup.timer`; list with
`systemctl list-timers`.

Monotonic timers use relative intervals instead:

```ini
[Timer]
OnBootSec=5min          # 5 minutes after boot
OnUnitActiveSec=1h      # 1 hour after last activation
```

### Command cheatsheet by task

**Files and directories:**

```bash
ls -la
ls -lah --sort=size
stat file
find /path -name "*.log" -mtime +30 -delete
find /path -type f -size +100M
rsync -avz src/ dest/
tar czf archive.tar.gz dir/
tar xzf archive.tar.gz
```

**Text processing:**

```bash
grep -rn "pattern" /path/
grep -rli "pattern" /path/
sort file | uniq -c | sort -rn
cut -d: -f1 /etc/passwd
awk '{print $1, $NF}' file
sed -i 's/old/new/g' file
diff -u file1 file2
```

**Network:**

```bash
ss -tlnp                  # listening TCP with processes
ss -tunap                 # all TCP/UDP connections
ip addr show              # interfaces and IPs
ip route show             # routing table
nc -zv host port          # test TCP port reachability
dig example.com           # DNS lookup
```

**Disk:**

```bash
df -h                     # filesystem usage
df -ih                    # inode usage
du -sh /path/* | sort -rh # directories sorted by size
lsblk -f                  # block devices with filesystems
blkid                     # partition UUIDs and types
iostat -x 1 5             # disk I/O statistics
```

**System info:**

```bash
uname -a
hostnamectl
uptime
free -h
nproc
lscpu
dmesg -T | tail -50
```

### Worked examples

**Diagnose high disk usage:**

```bash
df -h
du -sh /* 2>/dev/null | sort -rh | head -20
du -sh /var/log/* | sort -rh | head -10
journalctl --vacuum-size=200M
apt autoremove -y
```

**Create a systemd service for a Python app:**

```bash
cat > /etc/systemd/system/myapp.service << 'EOF'
[Unit]
Description=My Python Application
After=network.target

[Service]
Type=simple
User=app
WorkingDirectory=/srv/myapp
ExecStart=/srv/myapp/.venv/bin/python -m myapp
Restart=on-failure
RestartSec=5
Environment=PYTHONUNBUFFERED=1

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now myapp
systemctl status myapp
```

**Investigate a memory-hungry process:**

```bash
ps aux --sort=-%mem | head -10
PID=12345
ls -la /proc/$PID/fd | wc -l           # open file descriptors
ss -tlnp | grep $PID                   # listening ports
cat /proc/$PID/status | grep -i vmrss  # resident memory
kill -TERM $PID
sleep 2 && ps -p $PID || echo "Process terminated"
```

### Anti-patterns

- **Editing package-installed unit files in `/lib/systemd/system/`** —
  drop your override in `/etc/systemd/system/` instead (admin wins)
- **Forgetting `daemon-reload` after unit edits** — systemd keeps
  serving the old definition until you reload
- **`usermod -G` without `-a`** — this replaces the user's group list
  instead of appending
- **Bare `kill -9`** — always try SIGTERM first so processes can
  clean up
- **Not redirecting cron output** — silent failures with no logs
- **`rm -rf /var/log/*` to reclaim disk** — breaks running daemons
  that have log files open; use `journalctl --vacuum-*` and rotate
  logs instead
