//! Declarative emulator descriptions.

use serde::Deserialize;

/// A single emulator, entirely declarative.
#[derive(Debug, Clone, Deserialize)]
pub struct EmulatorManifest {
    /// Unique emulator id.
    pub id: String,
    /// Platform ids this emulator runs.
    pub platforms: Vec<String>,
    /// Path to the emulator executable.
    pub binary: String,
    /// Argument template.
    pub args: Vec<String>,
    /// Where the emulator may be installed (informational).
    #[serde(default)]
    pub detect_paths: Vec<String>,
}

impl EmulatorManifest {
    /// Parses a manifest from TOML.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Whether this emulator runs `platform`.
    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.iter().any(|p| p == platform)
    }
}
