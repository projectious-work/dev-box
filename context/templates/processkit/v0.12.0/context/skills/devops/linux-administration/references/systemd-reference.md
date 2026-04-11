# Systemd Reference

Unit file format, service management commands, journalctl patterns, and timer units.

## Unit File Format

Service unit files live in `/etc/systemd/system/` (admin-created) or `/lib/systemd/system/` (package-installed). Admin files take priority.

### Basic Service Unit

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

### Service Types

- **simple** (default) -- ExecStart is the main process
- **exec** -- like simple, but systemd waits for the binary to execute
- **forking** -- process forks and parent exits (use PIDFile=)
- **oneshot** -- runs once and exits (combine with RemainAfterExit=yes)
- **notify** -- process sends sd_notify when ready
- **idle** -- delayed until all jobs finish

### Restart Policies

- **no** -- never restart (default)
- **on-failure** -- restart on non-zero exit, signal, or timeout
- **on-abnormal** -- restart on signal or timeout only
- **always** -- restart regardless of exit reason
- **on-success** -- restart only on clean exit (code 0)

Use `RestartSec=` to add delay between restarts. Use `StartLimitIntervalSec=` and `StartLimitBurst=` to prevent restart loops.

### Dependencies

- **After=** / **Before=** -- ordering only (no activation)
- **Requires=** -- hard dependency, fails if dependency fails
- **Wants=** -- soft dependency, continues if dependency fails
- **BindsTo=** -- like Requires, also stops when dependency stops

## Common Commands

```bash
# Service lifecycle
systemctl start myapp
systemctl stop myapp
systemctl restart myapp
systemctl reload myapp           # if ExecReload is defined
systemctl enable myapp           # start on boot
systemctl enable --now myapp     # enable + start
systemctl disable myapp          # remove from boot

# Inspection
systemctl status myapp           # status with recent log lines
systemctl show myapp             # all properties
systemctl cat myapp              # show unit file content
systemctl list-dependencies myapp  # dependency tree
systemctl is-active myapp        # active/inactive (for scripts)
systemctl is-enabled myapp       # enabled/disabled
systemctl is-failed myapp        # failed check

# System-wide
systemctl daemon-reload          # reload unit files after changes
systemctl list-units --failed    # show failed units
systemctl list-timers            # show all timers
systemctl list-units --type=service --state=running   # running services
```

## Journalctl Patterns

```bash
# Follow logs for a service
journalctl -u myapp -f

# Recent logs with timestamps
journalctl -u myapp --since "1 hour ago"
journalctl -u myapp --since "2024-01-15 09:00" --until "2024-01-15 10:00"

# Filter by priority
journalctl -u myapp -p err                # errors and above
journalctl -u myapp -p warning..err       # warnings through errors

# Output formats
journalctl -u myapp -o json-pretty        # structured JSON
journalctl -u myapp -o short-precise      # precise timestamps
journalctl -u myapp -o cat                # message only, no metadata

# Boot logs
journalctl -b                             # current boot
journalctl -b -1                          # previous boot
journalctl --list-boots                   # list all recorded boots

# Kernel messages
journalctl -k                             # kernel messages (like dmesg)

# Space management
journalctl --disk-usage                   # check journal size
journalctl --vacuum-size=500M             # shrink to 500MB
journalctl --vacuum-time=30d              # keep only last 30 days
```

## Timer Units

Systemd timers are the modern alternative to cron. Create a `.timer` file alongside your `.service` file.

### Calendar Timer (like cron)

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

### Monotonic Timer (relative intervals)

```ini
[Timer]
OnBootSec=5min              # 5 minutes after boot
OnUnitActiveSec=1h          # 1 hour after last activation
```

### Calendar Syntax Examples

```
*-*-* 02:00:00              # daily at 2 AM
Mon *-*-* 09:00:00          # every Monday at 9 AM
*-*-01 00:00:00             # first day of every month
*-*-* *:00/15:00            # every 15 minutes
hourly                       # shorthand for *-*-* *:00:00
daily                        # shorthand for *-*-* 00:00:00
weekly                       # shorthand for Mon *-*-* 00:00:00
```

Verify calendar expressions with `systemd-analyze calendar "Mon *-*-* 09:00:00"`.

Enable timers with `systemctl enable --now backup.timer`. List active timers with `systemctl list-timers`.
