use std::path::{Path, PathBuf};

use stdg_core::SessionGuard;

/// Generic RAII primitive for the "never write into the game folder" rule:
/// a layer that needs scratch space (a symlink farm for DLL injection, a
/// generated config file...) creates one of these under the session's tmp
/// dir instead. `Drop` removes it, including when the game crashes or a
/// panic unwinds through the pipeline.
pub struct SessionTmpDir {
    path: PathBuf,
}

impl SessionTmpDir {
    pub fn create(session_dir: &Path, name: &str) -> std::io::Result<Self> {
        let path = session_dir.join(name);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SessionTmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

impl SessionGuard for SessionTmpDir {
    fn label(&self) -> &str {
        "session-tmp-dir"
    }
}
