//! Recursive copy for profiles.

use std::path::Path;

use crate::SavesError;

/// Copies `from` into `to` (created), keeping symlinks as links.
///
/// Sockets, fifos and devices (e.g. overlay whiteouts) are skipped with a warning.
pub(crate) fn copy_tree(from: &Path, to: &Path) -> Result<(), SavesError> {
    std::fs::create_dir_all(to).map_err(SavesError::io("creating", to))?;
    let entries = std::fs::read_dir(from).map_err(SavesError::io("reading", from))?;
    for entry in entries {
        let entry = entry.map_err(SavesError::io("reading", from))?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        let file_type = entry.file_type().map_err(SavesError::io("reading", &src))?;
        if file_type.is_dir() {
            copy_tree(&src, &dst)?;
        } else if file_type.is_symlink() {
            let target = std::fs::read_link(&src).map_err(SavesError::io("reading", &src))?;
            std::os::unix::fs::symlink(&target, &dst).map_err(SavesError::io("creating", &dst))?;
        } else if file_type.is_file() {
            std::fs::copy(&src, &dst).map_err(SavesError::io("copying", &src))?;
        } else {
            tracing::warn!("skipping special file {}", src.display());
        }
    }
    Ok(())
}
