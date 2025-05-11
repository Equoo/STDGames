
use serde::Serialize;
use serde::Deserialize;
use serde_json::Number;
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
	pub igdb: Number,
	pub launch_type: String,
	pub env: HashMap<String, String>,
	pub proton:	String,
	pub workdir:	String,
	pub exec_path:	String,
	pub config: Vec<ConfigPath>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameData {
	pub cover: String,
	pub name: String,
	pub genres: Vec<String>,
	pub publisher: String,
	pub developer: String,
	pub summary: String,
	pub rating: f32,
	pub release_dates: Vec<Number>,
	pub screenshots: Vec<String>,
	pub videos: Vec<String>,
	pub artworks: Vec<String>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameLibrary {
	pub games: Vec<GameInfo>,
	pub gamesdata: Vec<GameData>,
}

fn fetch_igdb(category: &str, query: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.igdb.com/v4/{}", category);

    let res = client
        .post(&url)
        .header("Accept", "application/json")
        .header("Client-ID", "rggouo5m4dsiowf6upejcgzyskt2vj")
        .header("Authorization", "Bearer nziqmeslg7vw0q6lhuz7kp3cv2rjkp")
        .body(query.to_string())
        .send()
        .map_err(|e| e.to_string())?;

    let json = res.json::<serde_json::Value>().map_err(|e| e.to_string())?;
    Ok(json)
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

		let gamesdata: Vec<GameData> = games.iter().map(|game| {
			let name = game.display_name.clone();
			let genres = vec!["Action".to_string(), "Adventure".to_string()]; // Placeholder
			let publisher = "Unknown".to_string(); // Placeholder
			let developer
			= "Unknown".to_string(); // Placeholder
			let summary = "No summary available".to_string(); // Placeholder
			let rating = 0.0; // Placeholder
			let release_dates = vec![0.into()]; // Placeholder
			let screenshots = vec!["https://example.com/screenshot.jpg".to_string()]; // Placeholder
			let videos = vec!["https://example.com/video.mp4".to_string()]; // Placeholder
			let artworks = vec!["https://example.com/artwork.jpg".to_string()]; // Placeholder

			let cover_rq = fetch_igdb("covers", &format!("fields *; where game = {};", game.igdb)).unwrap()
				.get(0)
				.and_then(|v| v.get("url"))
				.and_then(|v| v.as_str())
				.unwrap_or("")
				.to_string();
			let cover = cover_rq.replace("t_thumb", "t_cover_big_2x");

			GameData {
				cover,
				name,
				genres,
				publisher,
				developer,
				summary,
				rating,
				release_dates,
				screenshots,
				videos,
				artworks,
			}
		}).collect();

        Ok(GameLibrary { games, gamesdata })
    }

	pub fn get_game(&self, name: &str) -> Option<&GameInfo> {
        self.games.iter().find(|game| game.name == name)
    }
}
