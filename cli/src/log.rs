//! Structured append-only command log.
//!
//! Every aibox command appends one NDJSON line to `.aibox/aibox.log`
//! in the current project directory (if a project root can be found).
//! This log is gitignored (`.aibox/` is in the project's `.gitignore`).
//!
//! Log format (one JSON object per line):
//!   {"ts":"2026-04-10T14:32:11Z","cmd":"sync","version":"0.17.5","exit_code":0,"duration_ms":4230,"msg":"..."}
//!
//! Rotation: if `aibox.log` exceeds 1 MB, it is renamed to `aibox.log.1`
//! (overwriting any prior `.1`), and a new `aibox.log` is started.
//! Only one backup is kept to bound disk use.

use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;

const MAX_LOG_SIZE: u64 = 1_048_576; // 1 MB

/// One structured log entry appended to `.aibox/aibox.log`.
#[derive(Serialize)]
pub struct LogEntry {
    /// ISO 8601 UTC timestamp.
    pub ts: String,
    /// Command name (e.g. "sync", "build", "start", "init", "reset").
    pub cmd: String,
    /// aibox CLI version.
    pub version: String,
    /// 0 for success, 1 for error.
    pub exit_code: i32,
    /// Wall-clock milliseconds elapsed.
    pub duration_ms: u64,
    /// Short human-readable summary.
    pub msg: String,
}

/// Append a structured log entry to `.aibox/aibox.log` under `project_root`.
///
/// Silently swallows all I/O errors — logging must never crash the CLI.
/// Use: `let _ = append_log(root, &entry);`
pub fn append_log(project_root: &Path, entry: &LogEntry) -> Result<()> {
    let aibox_dir = project_root.join(".aibox");
    fs::create_dir_all(&aibox_dir)?;

    let log_path = aibox_dir.join("aibox.log");

    // Rotate if the log exceeds 1 MB.
    if log_path.exists() {
        let size = fs::metadata(&log_path)?.len();
        if size > MAX_LOG_SIZE {
            let rotated = aibox_dir.join("aibox.log.1");
            fs::rename(&log_path, &rotated)?;
        }
    }

    let json_line = serde_json::to_string(entry)?;
    let mut line = json_line;
    line.push('\n');

    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    file.write_all(line.as_bytes())?;

    Ok(())
}

/// Measures wall-clock time for a command and appends a log entry on finish.
pub struct LogTimer {
    cmd: String,
    start: std::time::Instant,
}

impl LogTimer {
    /// Start timing a command.
    pub fn start(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            start: std::time::Instant::now(),
        }
    }

    /// Record elapsed time and append one log entry. Silently swallows errors.
    pub fn finish(self, root: &Path, exit_code: i32, msg: &str) {
        let entry = LogEntry {
            ts: chrono::Utc::now().to_rfc3339(),
            cmd: self.cmd,
            version: env!("CARGO_PKG_VERSION").to_string(),
            exit_code,
            duration_ms: self.start.elapsed().as_millis() as u64,
            msg: msg.to_string(),
        };
        let _ = append_log(root, &entry);
    }
}
