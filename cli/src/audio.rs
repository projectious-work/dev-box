//! Host-side audio diagnostics and setup for PulseAudio over TCP.
//!
//! Checks whether PulseAudio is installed, running, has the TCP module loaded,
//! is configured for persistence, and is listening on the expected port.
//! On macOS, can create a launchd plist for auto-start with KeepAlive.

use crate::output;
use anyhow::Result;
use std::process::Command;

const DEFAULT_PORT: u16 = 4714;
const PLIST_LABEL: &str = "com.aibox.pulseaudio";

/// Outcome counters for the summary line.
struct Tally {
    pass: u32,
    warn: u32,
    fail: u32,
}

impl Tally {
    fn new() -> Self {
        Self {
            pass: 0,
            warn: 0,
            fail: 0,
        }
    }
    fn pass(&mut self, msg: &str) {
        self.pass += 1;
        output::ok(msg);
    }
    fn warn(&mut self, msg: &str) {
        self.warn += 1;
        output::warn(msg);
    }
    fn fail(&mut self, msg: &str) {
        self.fail += 1;
        output::error(msg);
    }
}

/// Run a command and return its stdout as a trimmed string, or None on failure.
fn run(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Check if a command exists on PATH.
fn has_cmd(cmd: &str) -> bool {
    crate::runtime::command_exists(cmd)
}

// ── Public entry points ──────────────────────────────────────────────────────

/// `aibox audio check` — diagnose host audio readiness.
pub fn cmd_audio_check(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let mut t = Tally::new();

    let os = std::env::consts::OS; // "macos" or "linux"

    // 1. Platform
    output::info(&format!("Platform: {}", os_label(os)));

    // 2. PulseAudio installation
    output::info("Checking PulseAudio installation...");
    check_pulseaudio_installed(&mut t);

    // 3. Daemon running
    output::info("Checking PulseAudio daemon...");
    let pa_running = check_pulseaudio_running(&mut t);

    // 4. TCP module
    output::info(&format!("Checking TCP module (port {port})..."));
    let tcp_loaded = if pa_running {
        check_tcp_module(&mut t, port)
    } else {
        t.fail("Cannot check TCP module — PulseAudio not running");
        false
    };

    // 5. Persistence
    output::info("Checking TCP module persistence...");
    check_tcp_persistence(&mut t, port, os);

    // 6. Port listening
    output::info(&format!("Checking port {port}..."));
    check_port_listening(&mut t, port, tcp_loaded);

    // 7. macOS launchd
    if os == "macos" {
        output::info("Checking macOS auto-start...");
        check_macos_launchd(&mut t);
    }

    // 8. Connectivity test
    if tcp_loaded {
        output::info("Testing connectivity...");
        check_connectivity(&mut t, port);
    }

    // Summary
    println!();
    if t.fail == 0 && t.warn == 0 {
        output::ok(&format!("Audio ready ({} checks passed)", t.pass));
    } else if t.fail == 0 {
        output::warn(&format!(
            "{} passed, {} warnings — audio should work, review warnings above",
            t.pass, t.warn
        ));
    } else {
        output::error(&format!(
            "{} passed, {} warnings, {} failed — audio not ready",
            t.pass, t.warn, t.fail
        ));
    }

    Ok(())
}

/// `aibox audio setup` — install and configure PulseAudio on the host.
pub fn cmd_audio_setup(port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(DEFAULT_PORT);
    let os = std::env::consts::OS;

    if os != "macos" {
        output::info("Automated setup is currently macOS-only.");
        output::info("On Linux, install pulseaudio and add to your PA config:");
        output::info(&format!(
            "  load-module module-native-protocol-tcp port={port} auth-ip-acl=127.0.0.1;172.16.0.0/12;10.0.0.0/8;192.168.0.0/16"
        ));
        return Ok(());
    }

    // 1. Install PulseAudio
    if !has_cmd("pulseaudio") {
        output::info("Installing PulseAudio via Homebrew...");
        let status = Command::new("brew")
            .args(["install", "pulseaudio"])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to install PulseAudio via Homebrew");
        }
        output::ok("PulseAudio installed");
    } else {
        output::ok("PulseAudio already installed");
    }

    // 2. Configure TCP module persistence
    let pa_conf = user_pa_config_path();
    let tcp_line = format!("load-module module-native-protocol-tcp port={port} auth-ip-acl=127.0.0.1;172.16.0.0/12;10.0.0.0/8;192.168.0.0/16");

    if let Ok(content) = std::fs::read_to_string(&pa_conf) {
        if content.contains(&format!("port={port}"))
            && content.contains("module-native-protocol-tcp")
        {
            output::ok("TCP module already configured in PA config");
        } else {
            append_tcp_to_config(&pa_conf, &tcp_line)?;
        }
    } else {
        // Config doesn't exist — copy system default as base, then append
        if let Some(system_conf) = find_system_pa_config() {
            if let Ok(system_content) = std::fs::read_to_string(&system_conf) {
                if let Some(parent) = std::path::Path::new(&pa_conf).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&pa_conf, system_content)?;
                output::info(&format!("Copied system PA config to {pa_conf}"));
            }
        } else if let Some(parent) = std::path::Path::new(&pa_conf).parent() {
            std::fs::create_dir_all(parent)?;
        }
        append_tcp_to_config(&pa_conf, &tcp_line)?;
    }

    // 3. Create launchd plist with KeepAlive
    setup_launchd_plist(port)?;

    // 4. Verify
    output::info("Verifying setup...");
    std::thread::sleep(std::time::Duration::from_secs(1));
    cmd_audio_check(Some(port))?;

    Ok(())
}

// ── Check functions ──────────────────────────────────────────────────────────

fn check_pulseaudio_installed(t: &mut Tally) {
    if has_cmd("pulseaudio") {
        if let Some(version) = run("pulseaudio", &["--version"]) {
            t.pass(&format!("pulseaudio found: {version}"));
        } else {
            t.pass("pulseaudio found");
        }
    } else if has_cmd("pipewire") && has_cmd("pactl") {
        t.pass("PipeWire with PulseAudio compatibility detected");
    } else {
        t.fail("PulseAudio not installed");
        if std::env::consts::OS == "macos" {
            output::info("  Install: brew install pulseaudio");
        } else {
            output::info("  Install: apt install pulseaudio (Debian/Ubuntu)");
        }
    }

    if has_cmd("pactl") {
        t.pass("pactl available");
    } else {
        t.fail("pactl not found");
    }
}

fn check_pulseaudio_running(t: &mut Tally) -> bool {
    // pulseaudio --check returns 0 if running
    if Command::new("pulseaudio")
        .arg("--check")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        t.pass("PulseAudio daemon is running");
        return true;
    }

    // PipeWire compatibility — pactl info succeeds even when pulseaudio --check fails
    if has_cmd("pactl")
        && Command::new("pactl")
            .arg("info")
            .output()
            .is_ok_and(|o| o.status.success())
    {
        t.pass("PulseAudio server responding (PipeWire compatibility)");
        return true;
    }

    t.fail("PulseAudio daemon is not running");
    output::info("  Start: pulseaudio --start");
    false
}

fn check_tcp_module(t: &mut Tally, port: u16) -> bool {
    if !has_cmd("pactl") {
        t.fail("Cannot check TCP module — pactl not available");
        return false;
    }

    let output = run("pactl", &["list", "modules", "short"]);
    if let Some(modules) = output {
        for line in modules.lines() {
            if line.contains("module-native-protocol-tcp") {
                if line.contains(&format!("port={port}")) {
                    t.pass(&format!("module-native-protocol-tcp loaded on port {port}"));
                } else {
                    t.warn("module-native-protocol-tcp loaded but possibly on a different port");
                    output::info(&format!("  {line}"));
                }
                return true;
            }
        }
    }

    t.fail("module-native-protocol-tcp not loaded");
    output::info(&format!(
        "  Load: pactl load-module module-native-protocol-tcp port={port} auth-ip-acl=127.0.0.1;172.16.0.0/12;10.0.0.0/8;192.168.0.0/16"
    ));
    false
}

fn check_tcp_persistence(t: &mut Tally, port: u16, os: &str) {
    let pa_conf = if os == "macos" {
        user_pa_config_path()
    } else {
        // Check user config first, fall back to system
        let user = user_pa_config_path();
        if std::path::Path::new(&user).exists() {
            user
        } else {
            "/etc/pulse/default.pa".to_string()
        }
    };

    if let Ok(content) = std::fs::read_to_string(&pa_conf) {
        let has_tcp = content.lines().any(|line| {
            !line.trim_start().starts_with('#') && line.contains("module-native-protocol-tcp")
        });

        if has_tcp {
            let has_port = content.lines().any(|line| {
                !line.trim_start().starts_with('#')
                    && line.contains("module-native-protocol-tcp")
                    && line.contains(&format!("port={port}"))
            });
            if has_port {
                t.pass(&format!("Persistent TCP config in {pa_conf}"));
            } else {
                t.warn(&format!(
                    "TCP module in {pa_conf} but port may differ from {port}"
                ));
            }
        } else {
            t.fail(&format!(
                "TCP module not in {pa_conf} (won't survive reboot)"
            ));
            output::info("  Run: aibox audio setup");
        }
    } else {
        t.fail(&format!("Config not found: {pa_conf}"));
        output::info("  Run: aibox audio setup");
    }
}

fn check_port_listening(t: &mut Tally, port: u16, tcp_loaded: bool) {
    let port_str = port.to_string();

    // Try lsof first (macOS + Linux)
    if has_cmd("lsof")
        && let Some(out) = run("lsof", &["-i", &format!(":{port}"), "-sTCP:LISTEN"])
        && !out.is_empty()
    {
        t.pass(&format!("Port {port} is listening"));
        return;
    }

    // Try ss (Linux)
    if has_cmd("ss")
        && let Some(out) = run("ss", &["-tlnp"])
        && out.contains(&format!(":{port_str}"))
    {
        t.pass(&format!("Port {port} is listening"));
        return;
    }

    if tcp_loaded {
        t.warn(&format!(
            "TCP module loaded but port {port} not detected — may bind to localhost only"
        ));
    } else {
        t.fail(&format!("Nothing listening on port {port}"));
    }
}

fn check_macos_launchd(t: &mut Tally) {
    if let Some(list) = run("launchctl", &["list"]) {
        if list.contains(PLIST_LABEL) {
            t.pass("aibox PulseAudio launch agent loaded (auto-starts, KeepAlive)");
            return;
        }
        if list.contains("homebrew.mxcl.pulseaudio") {
            t.pass("Homebrew PulseAudio launch agent loaded");
            output::info("  Consider: aibox audio setup (for KeepAlive support)");
            return;
        }
    }

    t.warn("PulseAudio does not auto-start on login");
    output::info("  Fix: aibox audio setup");
}

fn check_connectivity(t: &mut Tally, port: u16) {
    if !has_cmd("pactl") {
        return;
    }

    let output = Command::new("pactl")
        .arg("info")
        .env("PULSE_SERVER", format!("tcp:127.0.0.1:{port}"))
        .output();

    match output {
        Ok(o) if o.status.success() => {
            t.pass(&format!("pactl connects to tcp:127.0.0.1:{port}"));
        }
        _ => {
            t.warn(&format!("pactl cannot connect to tcp:127.0.0.1:{port}"));
            output::info("  Check auth settings (auth-anonymous=1 or auth-ip-acl)");
        }
    }
}

// ── Setup helpers ────────────────────────────────────────────────────────────

fn user_pa_config_path() -> String {
    if let Some(home) = dirs::home_dir() {
        home.join(".config/pulse/default.pa")
            .to_string_lossy()
            .into_owned()
    } else {
        "~/.config/pulse/default.pa".to_string()
    }
}

fn find_system_pa_config() -> Option<String> {
    // macOS Homebrew locations
    for prefix in ["/opt/homebrew", "/usr/local"] {
        let path = format!("{prefix}/etc/pulse/default.pa");
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    // Linux system location
    let system = "/etc/pulse/default.pa";
    if std::path::Path::new(system).exists() {
        return Some(system.to_string());
    }
    None
}

fn append_tcp_to_config(path: &str, tcp_line: &str) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file)?;
    writeln!(file, "# aibox: enable PulseAudio TCP for container audio")?;
    writeln!(file, "{tcp_line}")?;
    output::ok(&format!("Added TCP module config to {path}"));
    Ok(())
}

fn setup_launchd_plist(port: u16) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home dir"))?;
    let plist_path = home
        .join("Library/LaunchAgents")
        .join(format!("{PLIST_LABEL}.plist"));

    // Find pulseaudio binary path
    let pa_bin = run("which", &["pulseaudio"])
        .ok_or_else(|| anyhow::anyhow!("Cannot find pulseaudio binary"))?;

    let agents_dir = home.join("Library/LaunchAgents");
    std::fs::create_dir_all(&agents_dir)?;

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{PLIST_LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{pa_bin}</string>
    <string>--exit-idle-time=-1</string>
    <string>--log-target=syslog</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardErrorPath</key><string>/tmp/pulseaudio-aibox.err</string>
</dict>
</plist>"#
    );

    std::fs::write(&plist_path, &plist_content)?;

    // Unload brew services version if present to avoid conflicts
    let brew_plist = home.join("Library/LaunchAgents/homebrew.mxcl.pulseaudio.plist");
    if brew_plist.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", &brew_plist.to_string_lossy()])
            .output();
        output::info("Unloaded brew services agent to avoid conflicts");
    }

    // Load our plist
    let _ = Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .output();

    let status = Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .status()?;

    if status.success() {
        output::ok(&format!("Launch agent created: {}", plist_path.display()));
        output::ok("PulseAudio will auto-start on login and restart if it crashes");
    } else {
        output::warn("Failed to load launch agent — load manually:");
        output::info(&format!("  launchctl load {}", plist_path.display()));
    }

    // Also load the TCP module right now if not already loaded
    let modules = run("pactl", &["list", "modules", "short"]).unwrap_or_default();
    if !modules.contains("module-native-protocol-tcp") {
        let _ = Command::new("pactl")
            .args([
                "load-module",
                "module-native-protocol-tcp",
                &format!("port={port}"),
                "auth-ip-acl=127.0.0.1;172.16.0.0/12;10.0.0.0/8;192.168.0.0/16",
            ])
            .output();
    }

    Ok(())
}

fn os_label(os: &str) -> &str {
    match os {
        "macos" => "macOS",
        "linux" => "Linux",
        other => other,
    }
}
