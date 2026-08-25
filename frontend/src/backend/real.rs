use super::GameSource;
use crate::data::game::{ApiClient, Game};
use crate::data::GameDisplay;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

// Same CDN this app's mock data already points `hero`/`icon`/etc URLs at (see
// `data/mock_data.rs`) — except here the URLs come from the actual `/api/data` response instead
// of a guessed `cdn/steam/<appid>/<file>` naming convention, ported from
// `librarybackend/metadata.rs`'s `load_api_data`.
const API_URL: &str = "http://37.59.106.4:2356/api/data";
const CDN_PREFIX: &str = "http://37.59.106.4:2356/cdn/";

#[derive(Debug, Deserialize)]
struct GamesFile {
    games: Vec<Game>,
}

/// Shape of one entry in the `/api/data` response: asset fields are indices into `assets`
/// rather than URLs, mirroring `librarybackend/metadata.rs`'s `GameMetadataAPI`.
#[derive(Debug, Deserialize, Default)]
struct ApiMetadata {
    name: Option<String>,
    icon: Option<usize>,
    logo: Option<usize>,
    hero: Option<usize>,
    cover: Option<usize>,
    description: Option<String>,
    short_description: Option<String>,
    screenshots: Option<Vec<usize>>,
    movies: Option<Vec<usize>>,
    movies_thumbnails: Option<Vec<usize>>,
    tags: Option<Vec<String>>,
    #[serde(default)]
    assets: Vec<String>,
}

/// Real backend: reads the on-disk `games.toml` (see `RealGameSource::default_path`), then
/// enriches entries that carry a `metadata.api` (Steam/IGDB id) with real name/description/
/// artwork fetched from the CDN's `/api/data` endpoint. Games actually launching (Proton/overlay
/// execution) is out of scope here — that needs this app's own host environment (Proton-GE
/// installs, overlay mounts) to test against, so Play/Kill stay simulated like `MockGameSource`
/// until that's wired in separately.
pub struct RealGameSource {
    toml_path: PathBuf,
    running: Mutex<Option<String>>,
}

impl RealGameSource {
    pub fn new(toml_path: PathBuf) -> Self {
        Self { toml_path, running: Mutex::new(None) }
    }

    /// `$STDGAMES_LIBRARY_TOML`, else `<config dir>/games.toml` if the user has dropped a real
    /// one there, else the bundled `games.toml.exemple` (this repo's real library, despite the
    /// name — the `.exemple` suffix marks it as a template for the *file format*, not fake data).
    pub fn default_path() -> PathBuf {
        if let Ok(p) = std::env::var("STDGAMES_LIBRARY_TOML") {
            return PathBuf::from(p);
        }
        if let Some(dirs) = directories::ProjectDirs::from("dev", "STDGames", "stdgames-launcher") {
            let user_path = dirs.config_dir().join("games.toml");
            if user_path.exists() {
                return user_path;
            }
        }
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/games.toml.exemple"))
    }
}

#[async_trait]
impl GameSource for RealGameSource {
    async fn fetch_library(&self) -> Vec<GameDisplay> {
        fetch_library(&self.toml_path)
    }

    async fn launch_game(&self, slug: &str) -> bool {
        *self.running.lock().unwrap() = Some(slug.to_string());
        true
    }

    async fn get_running_game(&self) -> Option<String> {
        self.running.lock().unwrap().clone()
    }

    async fn kill_running_game(&self) {
        *self.running.lock().unwrap() = None;
    }

    fn add_desktop_icon(&self) {
        eprintln!("[DEV] Real backend: add desktop icon not implemented yet");
    }

    fn open_url(&self, url: &str) {
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(5)).timeout(Duration::from_secs(10)).build()
}

fn load_games(path: &Path) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(toml::from_str::<GamesFile>(&content)?.games)
}

/// Best-effort: on any network/parse failure, returns an empty map so callers fall back to
/// whatever metadata each `Game` already carries inline in the TOML.
fn fetch_api_metadata(games: &[Game]) -> HashMap<String, ApiMetadata> {
    let body: HashMap<&str, &ApiClient> =
        games.iter().filter_map(|g| g.metadata.api.as_ref().map(|api| (g.slug.as_str(), api))).collect();

    if body.is_empty() {
        return HashMap::new();
    }

    match agent().post(API_URL).send_json(&body) {
        Ok(res) => match res.into_json::<HashMap<String, ApiMetadata>>() {
            Ok(data) => data,
            Err(err) => {
                eprintln!("[real backend] failed to parse /api/data response: {err}");
                HashMap::new()
            }
        },
        Err(err) => {
            eprintln!("[real backend] /api/data request failed: {err}");
            HashMap::new()
        }
    }
}

fn resolve_asset(idx: Option<usize>, assets: &[String]) -> Option<String> {
    idx.and_then(|i| assets.get(i)).cloned()
}

fn resolve_list(idxs: &Option<Vec<usize>>, assets: &[String]) -> Option<Vec<String>> {
    idxs.as_ref().map(|list| list.iter().filter_map(|i| assets.get(*i).cloned()).collect())
}

fn non_empty(v: Option<Vec<String>>) -> Option<Vec<String>> {
    v.filter(|v| !v.is_empty())
}

/// Inline TOML fields always win over the API's, matching `load_api_data`'s
/// `game.metadata.x.or(data.x)` merge order.
fn merge(game: Game, api: Option<&ApiMetadata>) -> GameDisplay {
    let assets: Vec<String> =
        api.map(|a| a.assets.iter().map(|s| s.replace("resources/", CDN_PREFIX)).collect()).unwrap_or_default();
    let m = game.metadata;

    GameDisplay {
        slug: game.slug,
        name: m.name.or_else(|| api.and_then(|a| a.name.clone())),
        icon: m.icon.or_else(|| api.and_then(|a| resolve_asset(a.icon, &assets))),
        logo: m.logo.or_else(|| api.and_then(|a| resolve_asset(a.logo, &assets))),
        hero: m.hero.or_else(|| api.and_then(|a| resolve_asset(a.hero, &assets))),
        cover: m.cover.or_else(|| api.and_then(|a| resolve_asset(a.cover, &assets))),
        description: m.description.or_else(|| api.and_then(|a| a.description.clone())),
        short_description: m.short_description.or_else(|| api.and_then(|a| a.short_description.clone())),
        screenshots: non_empty(m.screenshots).or_else(|| api.and_then(|a| resolve_list(&a.screenshots, &assets))),
        movies: non_empty(m.movies).or_else(|| api.and_then(|a| resolve_list(&a.movies, &assets))),
        movies_thumbnails: non_empty(m.movies_thumbnails)
            .or_else(|| api.and_then(|a| resolve_list(&a.movies_thumbnails, &assets))),
        tags: non_empty(m.tags).or_else(|| api.and_then(|a| a.tags.clone())),
    }
}

fn fetch_library(path: &Path) -> Vec<GameDisplay> {
    let games = match load_games(path) {
        Ok(games) => games,
        Err(err) => {
            eprintln!("[real backend] failed to load {}: {err}", path.display());
            return Vec::new();
        }
    };

    let api_data = fetch_api_metadata(&games);
    games.into_iter().map(|g| { let api = api_data.get(&g.slug); merge(g, api) }).collect()
}
