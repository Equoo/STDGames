//! Container paths and bind mounts.

use std::path::{Path, PathBuf};

/// A path that may differ inside the container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValue {
    /// Valid on the host side only — no known/needed translation.
    Host(PathBuf),
    /// Known on both sides of the container boundary.
    Translated {
        /// Where the file lives on the host.
        host: PathBuf,
        /// Where the containerised process sees it.
        guest: PathBuf,
    },
}

impl PathValue {
    /// The host-side path, whichever variant this is.
    pub fn host(&self) -> &Path {
        match self {
            PathValue::Host(p) => p,
            PathValue::Translated { host, .. } => host,
        }
    }

    /// The path as seen by the consuming process.
    pub fn effective(&self) -> &Path {
        match self {
            PathValue::Host(p) => p,
            PathValue::Translated { guest, .. } => guest,
        }
    }
}

/// How a path is mounted into the container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindMode {
    /// Mounted read-only.
    ReadOnly,
    /// Mounted read-write.
    ReadWrite,
}

/// Label shown in `explain` and logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindPurpose(pub String);

/// Something a layer declares it needs to cross the container boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// What to mount, and where it appears inside the container.
    pub source: PathValue,
    /// Read-only or read-write.
    pub mode: BindMode,
    /// Why the layer needs it, for diagnostics.
    pub purpose: BindPurpose,
}
