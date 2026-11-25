//! What kind of game is being launched.

use serde::{Deserialize, Serialize};

/// The target kind determines the `Runner` — never the mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum TargetKind {
    /// A native Linux executable.
    NativeLinux,
    /// A Windows executable, run through a Compat layer.
    Windows,
    /// A ROM, run by an emulator for this platform.
    Rom(PlatformId),
}

/// ROM platform id, defined by emulator manifests.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlatformId(pub String);
