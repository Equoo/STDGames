//! Session identity.

use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use stdg_core::{SessionId, SessionInfo};

/// New session id and tmp dir path (not created).
pub fn new_session_info(base_tmp_dir: &std::path::Path) -> SessionInfo {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // Pre-1970 clock: the pid still keeps the id unique.
        .map_or(0, |d| d.as_nanos());
    let pid = std::process::id();
    let id = SessionId(format!("{pid:x}-{stamp:x}"));
    let tmp_dir: PathBuf = base_tmp_dir.join(format!("stdgames-{}", id.0));

    SessionInfo { id, tmp_dir }
}
