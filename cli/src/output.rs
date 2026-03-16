//! Colored terminal output helpers using ANSI escape codes.

/// Print an info message in cyan bold.
pub fn info(msg: &str) {
    eprintln!("\x1b[1;36m==> {}\x1b[0m", msg);
}

/// Print a success message in green bold.
pub fn ok(msg: &str) {
    eprintln!("\x1b[1;32m  \u{2713} {}\x1b[0m", msg);
}

/// Print a warning message in yellow bold.
pub fn warn(msg: &str) {
    eprintln!("\x1b[1;33m  ! {}\x1b[0m", msg);
}

/// Print an error message in red bold.
pub fn error(msg: &str) {
    eprintln!("\x1b[1;31mERR {}\x1b[0m", msg);
}
