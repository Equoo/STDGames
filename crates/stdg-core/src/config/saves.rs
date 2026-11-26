//! The `[saves]` section of a game config.

use serde::Deserialize;

/// Where a game keeps its saves and configs, and which profile to use.
///
/// Locations come from the Ludusavi manifest first, then `paths`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct SavesConfig {
    /// Ludusavi manifest entry; looked up by `app_id` when unset.
    pub manifest_name: Option<String>,
    /// Extra locations in Ludusavi syntax, e.g. `<winAppData>/Studio/Game`.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Profile to use; the game's current profile when unset.
    pub profile: Option<String>,
}
