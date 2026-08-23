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

/// Logs one step of the launch pipeline under `tag` (e.g. "setup",
/// "junest_cmd", "build_command"). Use this at every point where a path is
/// resolved, a subprocess is built/spawned, or a subprocess exits, so a
/// failed launch can be traced step by step from `log_file`.
pub fn log_step(tag: &str, msg: impl std::fmt::Display) {
    let line = format!("[{}] [{tag}] {msg}", timestamp());
    eprintln!("{line}");

    if let Some(parent) = std::path::Path::new(&CONFIG.log_file).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&CONFIG.log_file)
    {
        let _ = writeln!(file, "{line}");
    }
}
