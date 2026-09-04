use serde::Deserialize;

/// A single emulator, entirely declarative. Adding an emulator means adding
/// one of these files, never a code change.
#[derive(Debug, Clone, Deserialize)]
pub struct EmulatorManifest {
    pub id: String,
    pub platforms: Vec<String>,
    pub binary: String,
    /// Argument template. The literal token `{rom}` is replaced by the
    /// resolved rom path; every other token is passed through as-is.
    pub args: Vec<String>,
    #[serde(default)]
    pub detect_paths: Vec<String>,
}

impl EmulatorManifest {
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.iter().any(|p| p == platform)
    }
}
