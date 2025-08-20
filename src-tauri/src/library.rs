use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Games {
pub     games: Vec<Game>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Game {
	pub slug: String,
	pub status: String,
    pub metadata: GameMetadata,
    pub launch: GameLaunchData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameMetadata {
    pub idgb_id: Option<i32>,
    pub store_pages: Option<Vec<String>>,
	pub name: Option<String>,
	pub cover: Option<String>,
	pub icon: Option<String>,
	pub logo: Option<String>,
	pub description: Option<String>,
	pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameLaunchData {
    pub flags: String,
	pub environs: Option<HashMap<String, String>>,
	pub overlays: Vec<String>,
	pub start: Vec<String>,
	pub prestart: Option<Vec<String>>,
}

pub fn load_library(path: &str) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Games = toml::from_str(&content)?;
    Ok(config.games)
}