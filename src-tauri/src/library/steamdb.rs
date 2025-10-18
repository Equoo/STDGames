use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct Genre {
    id: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Screenshot {
    path_full: String,
    path_thumbnail: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    name: String,
    thumbnail: String,
    mp4: Option<HashMap<String, String>>,
    webm: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameAssets {
    pub name: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub genres: Vec<String>,
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
    pub icon: String,
    pub logo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieAsset {
    pub name: String,
    pub thumbnail: String,
    pub mp4_urls: HashMap<String, String>,
    pub webm_urls: HashMap<String, String>,
}

pub struct SteamAssetsClient {
    client: ReqwestClient,
    api_key: Option<String>,
    language: String,
    userids: [u64; 6],
}

impl SteamAssetsClient {
    pub fn new(api_key: Option<String>, lang: String) -> Self {
        Self {
            client: ReqwestClient::new(),
            api_key,
            language: lang,
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

    pub async fn get_icon_hash(
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

    pub async fn get_game_assets(
        &self,
        app_id: u32,
    ) -> Result<GameAssets, Box<dyn std::error::Error>> {
        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={}&l={}",
            app_id, self.language
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

        let app_data = app_response
            .data
            .as_ref()
            .ok_or("No app data in response")?;

        let assets = GameAssets {
            name: app_data.name.clone(),
            description: app_data.detailed_description.clone(),
            short_description: app_data.short_description.clone(),
            genres: app_data
                .genres
                .as_ref()
                .map(|genres| genres.iter().map(|g| g.description.clone()).collect())
                .unwrap_or_default(),
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
            icon: format!(
                "https://cdn.akamai.steamstatic.com/steamcommunity/public/images/apps/{}/icon.jpg",
                app_id
            ), // Placeholder - use get_icon_with_hash for actual icon
            logo: format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/logo.png",
                app_id
            ),
        };

        Ok(assets)
    }

    //pub async fn check_asset_availability(&self, url: &str) -> bool {
    //    match self.client.head(url).send().await {
    //        Ok(response) => response.status().is_success(),
    //        Err(_) => false,
    //    }
    //}

    //pub async fn download_asset(&self, url: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    //    let response = self.client.get(url).send().await?;

    //    if !response.status().is_success() {
    //        return Err(format!("Failed to download asset: HTTP {}", response.status()).into());
    //    }

    //    let bytes = response.bytes().await?;
    //    tokio::fs::write(file_path, bytes).await?;

    //    Ok(())
    //}

    pub async fn get_icon_with_hash(
        &self,
        app_id: u32,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        for uid in self.userids {
            if let Some(icon_hash) = self.get_icon_hash(app_id, uid).await? {
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

    pub async fn get_game_assets_with_icons(
        &self,
        app_id: u32,
    ) -> Result<GameAssets, Box<dyn std::error::Error>> {
        let mut assets = self.get_game_assets(app_id).await?;

        // Get the real icon URL with hash
        //if let Some(real_icon) = self.get_icon_with_hash(app_id).await? {
        //    assets.icon = real_icon;
        //}

        Ok(assets)
    }
}

// Additional utility functions
//impl GameAssets {
//    pub fn get_all_image_urls(&self) -> Vec<String> {
//        let mut urls = Vec::new();

//        if let Some(header) = &self.header_image {
//            urls.push(header.clone());
//        }
//        if let Some(capsule) = &self.capsule_image {
//            urls.push(capsule.clone());
//        }
//        if let Some(bg) = &self.background {
//            urls.push(bg.clone());
//        }
//        if let Some(bg_raw) = &self.background_raw {
//            urls.push(bg_raw.clone());
//        }

//        // Add screenshots
//        urls.extend(self.screenshots.iter().cloned());

//        // Add CDN assets
//        urls.push(self.library_hero.clone());
//        urls.push(self.library_600x900.clone());
//        urls.push(self.library_capsule_231x87.clone());
//        urls.push(self.library_capsule_616x353.clone());
//        urls.push(self.icon.clone());
//        urls.push(self.logo.clone());

//        urls
//    }

//    pub fn get_video_urls(&self) -> Vec<String> {
//        let mut urls = Vec::new();

//        for movie in &self.movies {
//            urls.push(movie.thumbnail.clone());
//            urls.extend(movie.mp4_urls.values().cloned());
//            urls.extend(movie.webm_urls.values().cloned());
//        }

//        urls
//    }
//}
