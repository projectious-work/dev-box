---
sidebar_position: 3
title: "Audio Support"
---

# Audio Support

The dev-box base image includes audio support for Claude Code's voice features. Audio is bridged from the container to the host via PulseAudio over TCP.

## Why Audio Matters

Claude Code supports voice interaction. For this to work inside a container, audio output (and optionally input) must be forwarded to the host's sound system. dev-box handles this by installing PulseAudio client utilities in the container and connecting them to a PulseAudio server running on the host.

## Architecture

```
Container                          Host
┌─────────────────────┐     ┌─────────────────────┐
│  Claude Code        │     │  PulseAudio Server   │
│       │             │     │       │              │
│  pulseaudio-utils   │────>│  TCP :4714           │
│  sox                │     │       │              │
│  .asoundrc          │     │  Speakers / Mic      │
└─────────────────────┘     └─────────────────────┘
```

The container sets `PULSE_SERVER` to point at the host's PulseAudio TCP module. Audio data flows over the network socket.

## Configuration in dev-box.toml

```toml
[audio]
enabled = true
pulse_server = "tcp:host.docker.internal:4714"
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Whether to set up audio environment variables in the container |
| `pulse_server` | `tcp:host.docker.internal:4714` | PulseAudio server address |

When `enabled = true`, the generated `docker-compose.yml` sets `PULSE_SERVER` in the container environment.

## Host Setup

The fastest way to set up audio on your host is the built-in CLI command:

```bash
# Check if your host is ready
dev-box audio check

# Automatic setup (macOS: installs PulseAudio, configures TCP, creates launchd agent)
dev-box audio setup
```

`dev-box audio setup` handles:

- Installing PulseAudio via Homebrew (macOS) if not present
- Configuring `~/.config/pulse/default.pa` with the TCP module on port 4714
- Creating a launchd agent with `KeepAlive` so PulseAudio auto-starts and restarts on crash (macOS)
- Loading the TCP module immediately

`dev-box audio check` diagnoses: PulseAudio installation, daemon status, TCP module, persistence config, port listening, launchd agent (macOS), and connectivity.

Both commands accept `--port` to override the default port (4714).

### Manual setup

If you prefer manual configuration:

#### macOS

1. Install PulseAudio:
   ```bash
   brew install pulseaudio
   ```

2. Enable the TCP module. Add to `~/.config/pulse/default.pa`:
   ```
   load-module module-native-protocol-tcp port=4714 auth-anonymous=1
   ```

3. Start PulseAudio:
   ```bash
   pulseaudio --start
   ```

4. Verify it is listening:
   ```bash
   lsof -i :4714
   ```

Docker Desktop and OrbStack provide `host.docker.internal` automatically. For Podman, check your machine's network configuration — you may need to use the host IP directly:

```toml
[audio]
enabled = true
pulse_server = "tcp:192.168.64.1:4714"
```

#### Linux

1. PulseAudio is likely already running. Enable the TCP module:
   ```bash
   pactl load-module module-native-protocol-tcp port=4714 auth-ip-acl=127.0.0.1;172.16.0.0/12;10.0.0.0/8;192.168.0.0/16
   ```

   To make this persistent, add to `~/.config/pulse/default.pa`:
   ```
   load-module module-native-protocol-tcp port=4714 auth-ip-acl=127.0.0.1;172.16.0.0/12;10.0.0.0/8;192.168.0.0/16
   ```

2. Use `host.docker.internal` (Docker 20.10+) or the Docker bridge IP:
   ```toml
   [audio]
   enabled = true
   pulse_server = "tcp:host.docker.internal:4714"
   ```

:::warning Firewall

Ensure port 4714 is accessible from the container network. On systems with strict firewalls, you may need to allow traffic from the Docker/Podman bridge interface.

:::

## Claude Code OAuth in Containers

When running `claude auth` inside a container with bridge networking (OrbStack, Docker Desktop), the OAuth callback may fail. Claude Code starts a temporary HTTP server on a random ephemeral port to receive the callback, but that port isn't forwarded to the host browser.

**Workaround:** Use `claude setup-token` to authenticate manually, or authenticate on the host first — the `.claude` directory is bind-mounted into the container, so credentials are shared.

:::info Upstream tracking

This is tracked at [anthropics/claude-code#14528](https://github.com/anthropics/claude-code/issues/14528). A fix on the Claude Code side (e.g., configurable callback port) would resolve this properly without compromising container network isolation.

:::

## The .asoundrc File

The base image includes an `.asoundrc` file at `/root/.asoundrc`. This configures ALSA to route through PulseAudio, so applications that use ALSA (rather than PulseAudio directly) also get audio output.

## Troubleshooting

### No sound output

1. Verify PulseAudio is running on the host:
   ```bash
   pulseaudio --check && echo "running" || echo "not running"
   ```

2. Verify the TCP module is loaded:
   ```bash
   pactl list modules | grep module-native-protocol-tcp
   ```

3. Test from inside the container:
   ```bash
   paplay /usr/share/sounds/freedesktop/stereo/bell.oga
   ```
   If the file does not exist, use `sox` to generate a test tone:
   ```bash
   play -n synth 0.5 sine 440
   ```

### Connection refused

The `PULSE_SERVER` address is not reachable from the container.

- Check that the PulseAudio TCP module is listening on the correct port
- Check that `host.docker.internal` resolves from inside the container:
  ```bash
  # From inside the container
  getent hosts host.docker.internal
  ```
- Try using the host's explicit IP address instead

### Audio works but is choppy

This is usually a network or resource issue. PulseAudio over TCP adds latency. Ensure the container has sufficient CPU resources and the host is not under heavy load.

### Disabling audio

If you do not need audio, set `enabled = false` in `dev-box.toml`:

```toml
[audio]
enabled = false
```

This removes the `PULSE_SERVER` environment variable from the container. The audio packages (`sox`, `pulseaudio-utils`) remain installed in the base image but are inert without a server to connect to.
