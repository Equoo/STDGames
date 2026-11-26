//! Errors of the save manager.

use std::path::PathBuf;

/// Something the save manager could not do.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SavesError {
    /// A filesystem operation failed.
    #[error("{action} {}", path.display())]
    Io {
        /// What was being done, e.g. `reading`.
        action: &'static str,
        /// The path it was done to.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// A profile or game name that is not a plain file name.
    #[error("invalid name {0:?}: use letters, digits, `.`, `_` or `-`, not starting with `.`")]
    InvalidName(String),
    /// The profile already exists.
    #[error("profile `{0}` already exists")]
    ProfileExists(String),
    /// Another launch or command is using the profile.
    #[error("profile `{0}` is in use by another launch")]
    ProfileInUse(String),
    /// The profile does not exist.
    #[error("profile `{0}` does not exist")]
    NoSuchProfile(String),
    /// The Ludusavi manifest could not be parsed.
    #[error("parsing the Ludusavi manifest")]
    Manifest(#[source] serde_yaml_ng::Error),
    /// The manifest index could not be read or written.
    #[error("manifest index {}", path.display())]
    Index {
        /// The index file.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: serde_json::Error,
    },
    /// `tar` failed or could not be run.
    #[error("tar {action} failed: {reason}")]
    Tar {
        /// `export` or `import`.
        action: &'static str,
        /// Exit status or spawn error.
        reason: String,
    },
    /// An imported archive does not hold exactly one profile.
    #[error("{} must contain exactly one profile directory", .0.display())]
    BadArchive(PathBuf),
}

impl SavesError {
    pub(crate) fn io(
        action: &'static str,
        path: &std::path::Path,
    ) -> impl FnOnce(std::io::Error) -> Self {
        let path = path.to_path_buf();
        move |source| SavesError::Io {
            action,
            path,
            source,
        }
    }
}
