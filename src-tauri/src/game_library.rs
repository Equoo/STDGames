
use serde::Serialize;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigPath {
	pub original: String,
	pub user: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameInfo {
	pub name: String,
	pub display_name: String,
	pub cover: String,
	pub genres: String,
	pub launch_type: String,
	pub env: HashMap<String, String>,
	pub proton:	String,
	pub exec_path:	String,
	pub config: Vec<ConfigPath>,
}

pub struct GameLibrary {
	pub games: Vec<GameInfo>,
}

impl GameLibrary {
	pub fn new() -> Result<GameLibrary, Box<dyn std::error::Error>> {
        let file_content = match fs::read_to_string("/sgoinfre/dderny/games.json") {
            Ok(content) => content,
            Err(e) => {
                println!("Error reading file: {}", e);
                return Err(Box::new(e));
            }
        };
        let games: Vec<GameInfo> = serde_json::from_str(&file_content)?;
        Ok(GameLibrary { games })
    }

	pub fn get_game(&self, name: &str) -> Option<&GameInfo> {
        self.games.iter().find(|game| game.name == name)
    }
}
