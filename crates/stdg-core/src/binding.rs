use std::path::{Path, PathBuf};

/// A path that may need translation when crossing the container boundary
/// (pressure-vessel). Never a raw `String` once a path can survive that
/// crossing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValue {
    /// Valid on the host side only — no known/needed translation.
    Host(PathBuf),
    /// The layer that produced this path already knows how it appears on
    /// both sides of the container boundary.
    Translated { host: PathBuf, guest: PathBuf },
}

impl PathValue {
    pub fn host(&self) -> &Path {
        match self {
            PathValue::Host(p) => p,
            PathValue::Translated { host, .. } => host,
        }
    }

    /// The path to use from the point of view of the process that consumes
    /// this value: `guest` when translated, `host` otherwise.
    pub fn effective(&self) -> &Path {
        match self {
            PathValue::Host(p) => p,
            PathValue::Translated { guest, .. } => guest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindMode {
    ReadOnly,
    ReadWrite,
}

/// Free-form label used for `explain` output and logs (e.g. "steamapi-emu-dll").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindPurpose(pub String);

/// Something a layer declares it needs to cross the container boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub source: PathValue,
    pub mode: BindMode,
    pub purpose: BindPurpose,
}
