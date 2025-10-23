use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::clients::{ApiClient, GameAssetList, GameMetadata};

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
pub struct GameDataAssets {
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

    pub async fn get_game_data_assets(
        &self,
        app_id: u32
    ) -> Result<GameDataAssets, Box<dyn std::error::Error>> {
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
        
        let assets = GameDataAssets {
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

    pub async fn fetch_download_data(&self, resources_path: &PathBuf, app_id: u32, lang: &str) -> Option<GameMetadata> {
        let mut assets = GameAssetList::new();
        let data = self.get_game_data_assets(app_id).await.ok()?;
        let lang_data = data.data_langs.get(lang).or_else(|| data.data_langs.values().next())?;

        let metadata = GameMetadata {
            store_pages: Some(vec![format!("https://store.steampowered.com/app/{}", app_id)]),
            name: Some(lang_data.name.clone()),
            icon: if let Some(icon) = data.icon { assets.add_some_asset(icon, resources_path.join("icon.jpg")) } else { None },
            logo: assets.add_some_asset(data.logo, resources_path.join("logo.png")),
            hero: assets.add_some_asset(data.library_hero, resources_path.join("library_hero.jpg")),
            cover: assets.add_some_asset(data.library_600x900, resources_path.join("library_600x900.jpg")),
            description: lang_data.description.clone(),
            short_description: lang_data.short_description.clone(),
            screenshots: if !data.screenshots.is_empty() {
                Some(data.screenshots.iter().enumerate().map(|(i, s)| {
                    assets.add_asset(s.clone(), resources_path.join(format!("screenshot_{}.jpg", i)))
                }).collect())
            } else {
                None
            },
            movies: if !data.movies.is_empty() {
                Some(
                    data
                        .movies
                        .iter().enumerate()
                        .map(|(i, m)| {
                            assets.add_asset(
                                m.mp4_urls.get("max").unwrap_or(&m.webm_urls.get("max").unwrap_or(&"".to_string())).clone(),
                                resources_path.join(format!(
                                    "movie_{}.{}",
                                    i,
                                    if m.mp4_urls.contains_key("max") { "mp4" } else { "webm" }
                                )),
                            )
                        })
                        .collect(),
                )
            } else {
                None
            },
            movies_thumbnails: if !data.movies.is_empty() {
                Some(data.movies.iter().map(|m| {
                    assets.add_asset(m.thumbnail.clone(), resources_path.join(format!("movie_thumbnail_{}.jpg", m.name.replace(" ", "_"))))
                }).collect())
            } else {
                None
            },
            tags: if !lang_data.genres.clone().is_empty() {
                Some(lang_data.genres.clone())
            } else {
                None
            },
            assets: assets.get_asset_paths(),
        };

        assets.download_all().await.ok()?;

        let metadata_path = resources_path.join("data.json");
        let metadata_json = serde_json::to_string_pretty(&metadata).ok()?;
        tokio::fs::write(metadata_path, metadata_json).await.ok()?;

        Some(metadata)
    }
}
