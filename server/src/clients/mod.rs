use std::path::PathBuf;
use tokio::fs;
use serde::{Deserialize, Serialize};

mod igdb;
mod steam;

use crate::clients::igdb::IgdbClient;
use crate::clients::steam::SteamClient;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiClient {
    pub id: u32,
    pub client: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameMetadata {
    pub api: ApiClient,
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

pub async fn download_asset(url: &String, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
	let response = reqwest::get(url.as_str()).await?;

	if !response.status().is_success() {
		return Err(format!("Failed to download asset: HTTP {}", response.status()).into());
	}

	let bytes = response.bytes().await?;
	fs::create_dir_all(file_path.parent().unwrap()).await?;
	tokio::fs::write(file_path, bytes).await?;

	Ok(())
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
			igdb: IgdbClient::new(igdb_id, igdb_secret).await?,
			steam: SteamClient::new(steam_key, vec!["french".to_string(), "english".to_string()]),
		})
	}

	pub async fn fetch_game_metadata(&self, api: ApiClient, lang: &str) -> Option<GameMetadata> {
		let resources_path = PathBuf::from(self.resources_path.clone()).join(api.client.as_str()).join(api.id.to_string());

		let data_json_path = resources_path.join("data.json");
		if data_json_path.exists() {
			let data = fs::read_to_string(data_json_path).await.ok()?;
			let metadata: GameMetadata = serde_json::from_str(&data).ok()?;
			Some(metadata)
		} else {
			match api.client.to_lowercase().as_str() {
				"igdb" => self.igdb.fetch_download_data(&resources_path, api.id, lang).await,
				"steam" => self.steam.fetch_download_data(&resources_path, api.id, lang).await,
				_ => None,
			}
		}
	}
}

