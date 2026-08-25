use serde::{Deserialize, Serialize};

// `Game`/`GameMetadata`/`GameLaunchData`/`ApiClient` mirror `lib/types/game.ts`'s Tauri-side
// shape (the real backend's `get_game_library` return type, mapped down to `GameDisplay` by
// `games.ts`) and, not by coincidence, the `[[games]]` shape in `games.toml.exemple` — this is
// what `backend::real::RealGameSource` deserializes each entry into. `environs` accepts the
// TOML's `environ` key (singular) via `alias` since the field elsewhere follows the plural
// `GameMetadata`/`GameDisplay` convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClient {
    pub id: i64,
    pub client: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    pub api: Option<ApiClient>,
    pub store_pages: Option<Vec<String>>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub hero: Option<String>,
    pub cover: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub screenshots: Option<Vec<String>>,
    pub movies: Option<Vec<String>>,
    pub movies_thumbnails: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameLaunchData {
    pub proton: Option<String>,
    pub winetricks: Option<Vec<String>>,
    pub noruntime: Option<bool>,
    pub epicgame: Option<bool>,
    #[serde(alias = "environ")]
    pub environs: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub overlays: Vec<String>,
    #[serde(default)]
    pub start: Vec<String>,
    pub prestart: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub slug: String,
    pub status: String,
    #[serde(default)]
    pub metadata: GameMetadata,
    #[serde(default)]
    pub launch: GameLaunchData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameDisplay {
    pub slug: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub hero: Option<String>,
    pub cover: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub screenshots: Option<Vec<String>>,
    pub movies: Option<Vec<String>>,
    pub movies_thumbnails: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl GameDisplay {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.slug)
    }

    /// `hero`, falling back to the portrait `cover` art — the same fallback `AppState::to_card`
    /// uses to fill `GameCardData::hero`/`hero_thumb`. Centralized here so
    /// `AppState::card_image_urls` requests the exact same source URL it later looks up.
    pub fn hero_source(&self) -> Option<&str> {
        self.hero.as_deref().or(self.cover.as_deref()).filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}
