use anyhow::Result;
use rusty_ytdl::Video;
use serde::{Deserialize, Serialize, de};
use std::path::PathBuf;
use tokio::fs;
use tokio::task::JoinHandle;
use tracing::{debug, info};

mod igdb;
mod steam;

use crate::clients::igdb::IgdbClient;
use crate::clients::steam::SteamClient;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiClient {
    pub id: u32,
    pub client: String,
}

#[derive(Debug, Clone)]
pub struct GameAsset {
    pub url: String,
    pub file_path: PathBuf,
}

impl GameAsset {
    pub fn new(url: String, file_path: PathBuf) -> Self {
        Self { url, file_path }
    }

    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Downloading asset from {}", self.url);
        if self.url.find("youtube.com").is_some() {
            let video = Video::new(&self.url)?;
            video.download(&self.file_path).await?;
        } else {
            let response = reqwest::get(self.url.as_str()).await?;

            if !response.status().is_success() {
                debug!("Failed to download asset: HTTP {}", response.status());
                return Err(format!("Failed to download asset: HTTP {}", response.status()).into());
            }

            let bytes = response.bytes().await?;
            fs::create_dir_all(self.file_path.parent().unwrap()).await?;
            tokio::fs::write(&self.file_path, bytes).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GameAssetList {
    pub assets: Vec<GameAsset>,
}

impl GameAssetList {
    pub fn new() -> Self {
        Self { assets: Vec::new() }
    }

    pub fn add_some_asset(&mut self, url: String, file_path: PathBuf) -> Option<usize> {
        self.assets.push(GameAsset::new(url, file_path));
        Some(self.assets.len() - 1)
    }

    pub fn add_asset(&mut self, url: String, file_path: PathBuf) -> usize {
        self.assets.push(GameAsset::new(url, file_path));
        self.assets.len() - 1
    }

    pub async fn download_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for asset in &self.assets {
            let asset_cloned = asset.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = asset_cloned.download().await {
                    eprintln!("Error downloading asset {}: {}", asset_cloned.url, e);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            debug!("Waiting for asset download to complete");
            handle.await?;
        }

        Ok(())
    }

    pub fn get_asset_paths(&self) -> Vec<String> {
        self.assets
            .iter()
            .map(|asset| asset.file_path.to_string_lossy().into_owned())
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameMetadata {
    pub store_pages: Option<Vec<String>>,
    pub name: Option<String>,
    pub icon: Option<usize>,
    pub logo: Option<usize>,
    pub hero: Option<usize>,
    pub cover: Option<usize>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub screenshots: Option<Vec<usize>>,
    pub movies: Option<Vec<usize>>,
    pub movies_thumbnails: Option<Vec<usize>>,
    pub tags: Option<Vec<String>>,
    pub assets: Vec<String>,
}

pub struct ApiClients {
    resources_path: PathBuf,
    igdb: IgdbClient,
    steam: SteamClient,
}

impl ApiClients {
    pub async fn new(resources: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let igdb_id = std::env::var("IGDB_ID").expect("IGDB_ID not set");
        let igdb_secret = std::env::var("IGDB_SECRET").expect("IGDB_SECRET not set");
        let steam_key = std::env::var("STEAM_KEY").expect("STEAM_KEY not set");

        Ok(Self {
            resources_path: PathBuf::from(resources),
            igdb: IgdbClient::new(&igdb_id, &igdb_secret).await?,
            steam: SteamClient::new(steam_key, vec!["french".to_string(), "english".to_string()]),
        })
    }

    pub async fn fetch_game_metadata(
        &mut self,
        api: ApiClient,
        lang: &str,
    ) -> Option<GameMetadata> {
        let resources_path = PathBuf::from(self.resources_path.clone())
            .join(api.client.as_str())
            .join(api.id.to_string());

        info!("Fetching metadata for {} (ID: {})", api.client, api.id);

        let data_json_path = resources_path.join("data.json");
        if data_json_path.exists() {
            let data = fs::read_to_string(data_json_path).await.ok()?;
            debug!(
                "Loaded metadata from cache for {} (ID: {})",
                api.client, api.id
            );
            let metadata: GameMetadata = serde_json::from_str(&data).ok()?;
            Some(metadata)
        } else {
            debug!(
                "Fetching metadata from API for {} (ID: {})",
                api.client, api.id
            );
            match api.client.to_lowercase().as_str() {
                "igdb" => {
                    self.igdb
                        .fetch_download_data(&resources_path, api.id, lang)
                        .await
                }
                "steam" => {
                    self.steam
                        .fetch_download_data(&resources_path, api.id, lang)
                        .await
                }
                _ => None,
            }
        }
    }
}
