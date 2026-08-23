// Step-by-step launch tracing. Writes to stderr (visible when run from a
// terminal) and appends to CONFIG.log_file (visible afterwards even when
// launched from a desktop entry with no terminal attached). Best-effort:
// logging failures never abort the actual game launch.

use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::CONFIG;

fn timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs_today = elapsed.as_secs() % 86400;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        secs_today / 3600,
        (secs_today % 3600) / 60,
        secs_today % 60,
        elapsed.subsec_millis()
    )
}

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD_RED: &str = "\x1b[1;31m";

/// One consistent color per pipeline stage, so a scrolling terminal stays
/// scannable at a glance. Falls back to no color for any unlisted tag.
fn tag_color(tag: &str) -> &'static str {
    match tag {
        "setup" => "\x1b[36m",          // cyan
        "replace_vars" => "\x1b[35m",   // magenta
        "junest_cmd" => "\x1b[34m",     // blue
        "build_command" => "\x1b[33m",  // yellow
        "manager" => "\x1b[32m",        // green
        "cli" => "\x1b[37m",            // white
        _ => "",
    }
}

fn looks_like_failure(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("failed") || lower.contains("error") || lower.contains("unauthorized")
}

/// Logs one step of the launch pipeline under `tag` (e.g. "setup",
/// "junest_cmd", "build_command"). Use this at every point where a path is
/// resolved, a subprocess is built/spawned, or a subprocess exits, so a
/// failed launch can be traced step by step from `log_file`.
pub fn log_step(tag: &str, msg: impl std::fmt::Display) {
    let msg = msg.to_string();
    let ts = timestamp();
    let plain_line = format!("[{ts}] [{tag}] {msg}");

    // Color only the terminal line: the tag in its stage color, the message
    // in bold red when it looks like a failure so problems jump out
    // regardless of which stage they came from. The log file stays
    // plain-text so it's clean to grep/cat/open in an editor later.
    let color = tag_color(tag);
    let colored_line = if looks_like_failure(&msg) {
        format!("{DIM}[{ts}]{RESET} {color}[{tag}]{RESET} {BOLD_RED}{msg}{RESET}")
    } else {
        format!("{DIM}[{ts}]{RESET} {color}[{tag}]{RESET} {msg}")
    };
    eprintln!("{colored_line}");

    if let Some(parent) = std::path::Path::new(&CONFIG.log_file).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&CONFIG.log_file)
    {
        let _ = writeln!(file, "{plain_line}");
    }
}
