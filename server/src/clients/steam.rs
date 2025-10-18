use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::clients::{download_asset, ApiClient, GameMetadata};

#[derive(Debug, Serialize, Deserialize)]
struct SteamApiResponse {
    #[serde(flatten)]
    apps: HashMap<String, AppResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppResponse {
    success: bool,
    data: Option<AppData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppData {
    name: String,
    detailed_description: Option<String>,
    short_description: Option<String>,
    header_image: Option<String>,
    capsule_image: Option<String>,
    background: Option<String>,
    background_raw: Option<String>,
    screenshots: Option<Vec<Screenshot>>,
    movies: Option<Vec<Movie>>,
    genres: Option<Vec<Genre>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Genre {
    id: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Screenshot {
    path_full: String,
    path_thumbnail: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Movie {
    name: String,
    thumbnail: String,
    mp4: Option<HashMap<String, String>>,
    webm: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub name: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub genres: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameAssets {
    pub data_langs: HashMap<String, GameData>,
    pub header_image: Option<String>,
    pub capsule_image: Option<String>,
    pub background_raw: Option<String>,
    pub background: Option<String>,
    pub screenshots: Vec<String>,
    pub movies: Vec<MovieAsset>,
    // Additional CDN assets
    pub library_hero: String,
    pub library_600x900: String,
    pub library_capsule_231x87: String,
    pub library_capsule_616x353: String,
    pub icon: Option<String>,
    pub logo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieAsset {
    pub name: String,
    pub thumbnail: String,
    pub mp4_urls: HashMap<String, String>,
    pub webm_urls: HashMap<String, String>,
}

pub struct SteamClient {
    client: ReqwestClient,
    api_key: Option<String>,
    languages: Vec<String>,
    userids: [u64; 6],
}

impl SteamClient {
    pub fn new(api_key: String, languages: Vec<String>) -> Self {
        Self {
            client: ReqwestClient::new(),
            api_key: Some(api_key),
            languages,
            userids: [
                76561198017975643,
                76561198028121353,
                76561198355953202,
                76561197979911851,
                76561198002410826,
                76561198879997583,
            ],
        }
    }

    pub async fn try_get_icon(
        &self,
        app_id: u32,
        user_id: u64,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(api_key) = &self.api_key {
            let url = format!(
                "https://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&include_appinfo=true&appids_filter[0]={}",
                api_key, user_id, app_id
            );

            #[derive(Deserialize)]
            struct OwnedGamesResponse {
                response: OwnedGames,
            }

            #[derive(Deserialize)]
            struct OwnedGames {
                games: Option<Vec<OwnedGame>>,
            }

            #[derive(Deserialize)]
            struct OwnedGame {
                img_icon_url: Option<String>,
                img_logo_url: Option<String>, // note: not used currently
            }

            let response = self
                .client
                .get(&url)
                .send()
                .await?
                .json::<OwnedGamesResponse>()
                .await?;

            if let Some(games) = response.response.games {
                if let Some(game) = games.first() {
                    return Ok(game.img_icon_url.clone());
                }
            }
        }
        Ok(None)
    }

    pub async fn search_icon(
        &self,
        app_id: u32,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        for uid in self.userids {
            if let Some(icon_hash) = self.try_get_icon(app_id, uid).await? {
                println!(
                    "Found icon hash for app {} with user {}: {}",
                    app_id, uid, icon_hash
                );
                return Ok(Some(format!(
                    "https://media.steampowered.com/steamcommunity/public/images/apps/{}/{}.jpg",
                    app_id, icon_hash
                )));
            }
        }
        Ok(None)
    }

    pub async fn get_game_assets(
        &self,
        app_id: u32
    ) -> Result<GameAssets, Box<dyn std::error::Error>> {
        let mut app_datas: HashMap<&str, AppData> = HashMap::new();

        for lang in self.languages.iter() {
            let url = format!(
                "https://store.steampowered.com/api/appdetails?appids={}&l={}",
                app_id, lang
            );

            let response = self
                .client
                .get(&url)
                .send()
                .await?
                .json::<SteamApiResponse>()
                .await?;

            let app_response = response
                .apps
                .get(&app_id.to_string())
                .ok_or("App not found in response")?;

            if !app_response.success {
                return Err("Steam API returned success: false".into());
            }

            let app_data = app_response.data.as_ref().unwrap().clone();

            app_datas.insert(lang, app_data);
        }

        let app_data = app_datas.get(&self.languages[0].as_str())
            .ok_or("No app data found for preferred language")?;
        
        let assets = GameAssets {
            data_langs: app_datas.clone().into_iter().map(|(k, v)| (k.to_string(), GameData {
                name: v.name.clone(),
                description: v.detailed_description.clone(),
                short_description: v.short_description.clone(),
                genres: v
                    .genres
                    .as_ref()
                    .map(|genres| genres.iter().map(|g| g.description.clone()).collect())
                    .unwrap_or_default(),
            })).collect(),
            header_image: app_data.header_image.clone(),
            capsule_image: app_data.capsule_image.clone(),
            background: app_data.background.clone(),
            background_raw: app_data.background_raw.clone(),
            screenshots: app_data
                .screenshots
                .as_ref()
                .map(|screenshots| screenshots.iter().map(|s| s.path_full.clone()).collect())
                .unwrap_or_default(),
            movies: app_data
                .movies
                .as_ref()
                .map(|movies| {
                    movies
                        .iter()
                        .map(|m| MovieAsset {
                            name: m.name.clone(),
                            thumbnail: m.thumbnail.clone(),
                            mp4_urls: m.mp4.clone().unwrap_or_default(),
                            webm_urls: m.webm.clone().unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            // CDN asset URLs
            library_hero: format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_hero.jpg",
                app_id
            ),
            library_600x900: format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg",
                app_id
            ),
            library_capsule_231x87: format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/capsule_231x87.jpg",
                app_id
            ),
            library_capsule_616x353: format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/capsule_616x353.jpg",
                app_id
            ),
            icon: self.search_icon(app_id).await?,
            logo: format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/logo.png",
                app_id
            ),
        };

        Ok(assets)
    }

    async fn download_assets(
        &self,
        resources_path: &PathBuf,
        assets: &GameAssets,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(header_image) = &assets.header_image {
            download_asset(header_image, &resources_path.join("header_image.jpg")).await?;
        }

        if let Some(capsule_image) = &assets.capsule_image {
            download_asset(capsule_image, &resources_path.join("capsule_image.jpg")).await?;
        }

        if let Some(background_raw) = &assets.background_raw {
            download_asset(background_raw, &resources_path.join("background_raw.jpg")).await?;
        }

        if let Some(background) = &assets.background {
            download_asset(background, &resources_path.join("background.jpg")).await?;
        }

        for (index, screenshot) in assets.screenshots.iter().enumerate() {
            let file_path = resources_path.join(format!("screenshot_{}.jpg", index + 1));
            download_asset(screenshot, &file_path).await?;
        }

        for (index, movie) in assets.movies.iter().enumerate() {
            if let Some(url) = movie.mp4_urls.get("max") {
                let file_path = resources_path.join(format!("movie_{}.mp4", index + 1));
                download_asset(url, &file_path).await?;
            } else if let Some(url) = movie.webm_urls.get("max") {
                let file_path = resources_path.join(format!("movie_{}.webm", index + 1));
                download_asset(url, &file_path).await?;
            }
        }

        download_asset(&assets.library_hero, &resources_path.join("library_hero.jpg")).await?;
        download_asset(&assets.library_600x900, &resources_path.join("library_600x900.jpg")).await?;
        download_asset(&assets.library_capsule_231x87, &resources_path.join("library_capsule_231x87.jpg")).await?;
        download_asset(&assets.library_capsule_616x353, &resources_path.join("library_capsule_616x353.jpg")).await?;
        if let Some(icon_url) = &assets.icon {
            download_asset(icon_url, &resources_path.join("icon.jpg")).await?;
        }
        download_asset(&assets.logo, &resources_path.join("logo.png")).await?;

        Ok(())
    }

    pub async fn fetch_download_data(&self, resources_path: &PathBuf, app_id: u32, lang: &str) -> Option<GameMetadata> {
        let assets = self.get_game_assets(app_id).await.ok()?;
        let metadata = GameMetadata {
            api: ApiClient {
                id: app_id,
                client: "steam".to_string(),
            },
            store_pages: Some(vec![format!("https://store.steampowered.com/app/{}", app_id)]),
            name: Some(assets.data_langs.get(lang).map_or(assets.data_langs.values().next()?.name.clone(), |d| d.name.clone())),
            icon: resources_path.join("icon.jpg").to_string_lossy().to_string().into(),
            logo: Some(resources_path.join("logo.png").to_string_lossy().to_string()),
            hero: Some(resources_path.join("library_hero.jpg").to_string_lossy().to_string()),
            cover: Some(resources_path.join("library_600x900.jpg").to_string_lossy().to_string()),
            description: assets.data_langs.get(lang).and_then(|d| d.description.clone()),
            short_description: assets.data_langs.get(lang).and_then(|d| d.short_description.clone()),
            screenshots: if !assets.screenshots.is_empty() {
                Some(assets.screenshots.iter().enumerate().map(|(i, s)| resources_path.join(format!("screenshot_{}.jpg", i + 1)).to_string_lossy().to_string()).collect())
            } else {
                None
            },
            movies: if !assets.movies.is_empty() {
                Some(
                    assets
                        .movies
                        .iter().enumerate()
                        .map(|(i, m)| {
                            resources_path.join(format!("movie_{}.{}", i, m.mp4_urls.get("max").unwrap_or(&"webm".into()))).to_string_lossy().to_string()
                        })
                        .collect(),
                )
            } else {
                None
            },
            movies_thumbnails: if !assets.movies.is_empty() {
                Some(assets.movies.iter().map(|m| m.thumbnail.clone()).collect())
            } else {
                None
            },
            tags: if !assets.data_langs.get(lang).map_or(vec![], |d| d.genres.clone()).is_empty() {
                Some(assets.data_langs.get(lang).map_or(vec![], |d| d.genres.clone()))
            } else {
                None
            },
        };

        self.download_assets(resources_path, &assets).await.ok()?;

        let metadata_path = resources_path.join("data.json");
        let metadata_json = serde_json::to_string_pretty(&metadata).ok()?;
        tokio::fs::write(metadata_path, metadata_json).await.ok()?;

        Some(metadata)
    }
}