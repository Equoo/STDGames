use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use stdg_core::{SessionId, SessionInfo};

/// Generates a fresh session id and its dedicated tmp directory path (not
/// created yet: creating it on disk is `SessionTmpDir`'s job, so that the
/// same RAII guard type is used whether the caller is a layer or the
/// executor itself).
pub fn new_session_info(base_tmp_dir: &std::path::Path) -> SessionInfo {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let id = SessionId(format!("{pid:x}-{stamp:x}"));
    let tmp_dir: PathBuf = base_tmp_dir.join(format!("stdgames-{}", id.0));

    SessionInfo { id, tmp_dir }
}
