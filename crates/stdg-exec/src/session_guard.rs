//! Session-scoped scratch directories.

use std::path::{Path, PathBuf};

use stdg_core::SessionGuard;

/// Session scratch directory, removed on drop.
#[derive(Debug)]
pub struct SessionTmpDir {
    path: PathBuf,
}

impl SessionTmpDir {
    /// Creates `session_dir/name` (and its parents).
    pub fn create(session_dir: &Path, name: &str) -> std::io::Result<Self> {
        let path = session_dir.join(name);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// The directory's path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SessionTmpDir {
    fn drop(&mut self) {
        // Drop can't propagate errors, so log them.
        if let Err(e) = std::fs::remove_dir_all(&self.path) {
            tracing::warn!("could not remove {}: {e}", self.path.display());
        }
    }
}

impl SessionGuard for SessionTmpDir {
    fn label(&self) -> &'static str {
        "session-tmp-dir"
    }
}
