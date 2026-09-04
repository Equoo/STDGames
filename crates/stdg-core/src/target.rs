use serde::{Deserialize, Serialize};

/// The target kind determines the `Runner` — never the mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum TargetKind {
    NativeLinux,
    Windows,
    Rom(PlatformId),
}

/// Open ROM platform identifier (not a closed enum): adding an emulator
/// means adding a manifest file, not a code variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlatformId(pub String);
