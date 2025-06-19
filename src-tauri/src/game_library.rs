use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigPath {
    pub original: String,
    pub user: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rconf {
    pub src: String,
    pub dest: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameInfo {
    pub name: String,
    pub igdb: Number,
    pub launch_type: String,
    pub env: HashMap<String, String>,
    pub proton: String,
    pub workdir: Option<String>,
    pub exec_path: String,
    pub config: Option<Vec<ConfigPath>>,
    pub prestart: Option<String>,
    pub disabled: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub r_conf: Option<Vec<Rconf>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameData {
    pub cover: String,
    pub icon: String,
    pub name: String,
    pub displayname: String,
    pub gamemode: String,
    pub genres: Vec<String>,
    pub publisher: String,
    pub developer: String,
    pub summary: String,
    pub rating: f32,
    pub release_dates: Vec<Number>,
    pub screenshots: Vec<String>,
    pub videos: Vec<String>,
    pub artworks: Vec<String>,
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

        let mut query_games = String::new();
        let mut i = 0;
        for game in &games {
            let orstr = if i == 0 { "" } else { "," };
            query_games.push_str(&format!("{}{}", orstr, game.igdb));
            i += 1;
        }

        // vector len of games
        let mut gamesdata: Vec<GameData> = Vec::new();
        gamesdata.reserve(games.len());

        let query = format!(
            "
query covers \"covers\" {{
	fields url, game;
	where game = ({});
	limit 50;
}};
query artworks \"artworks\" {{
	fields url, game;
	where game = ({});
	limit 50;
}};
query games \"games\" {{
	fields name, summary;
	where id = ({});
	limit 50;
}};
		",
            query_games, query_games, query_games
        );
        let games_rq = fetch_igdb("multiquery", &query).unwrap();
        //println!("Multi: {} {:?}", query, games_rq);

        let void = vec![];
        let covers = games_rq
            .get(0)
            .and_then(|v| v.get("result").and_then(|v| v.as_array()))
            .unwrap_or(&void);
        let all_artworks = games_rq
            .get(1)
            .and_then(|v| v.get("result").and_then(|v| v.as_array()))
            .unwrap_or(&void);
        let gamesinfos = games_rq
            .get(2)
            .and_then(|v| v.get("result").and_then(|v| v.as_array()))
            .unwrap_or(&void);

        //println!("Covers: {:?}", covers);
        //println!("all_artworks: {:?}", all_artworks);

        for i in 0..games.len() {
            let game = &games[i];
            let id = game.igdb.as_u64().unwrap_or(0) as usize;

            let mut cover = "https://example.com/placeholder.jpg".to_string();
            let mut icon = "https://example.com/placeholder.jpg".to_string();
            covers.iter().for_each(|v| {
                if v.get("game").and_then(|v| v.as_u64()).unwrap_or(0) as usize == id {
                    cover = v
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                        .replace("t_thumb", "t_cover_big_2x")
                        .replace("//", "https://");
                    icon = v
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                        .replace("t_thumb", "t_logo_med")
                        .replace("//", "https://");
                }
            });

            let mut artworks: Vec<String> = vec![];
            all_artworks.iter().for_each(|v| {
                if v.get("game").and_then(|v| v.as_u64()).unwrap_or(0) as usize == id {
                    artworks.push(
                        v.get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                            .replace("t_thumb", "t_1080p_2x")
                            .replace("//", "https://"),
                    );
                }
            });

            let hdcover = cover.replace("t_cover_big_2x", "t_1080p_2x");
            artworks.push(hdcover);

            let void: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let gameinfo = gamesinfos
                .iter()
                .find(|v| v.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as usize == id)
                .and_then(|v| v.as_object())
                .unwrap_or(&void);

            let name = gameinfo
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let displayname = gameinfo
                .get("name")
                .and_then(|v| v.as_str())
                .expect("Missing game name")
                .to_string();
            let gamemode = gameinfo
                .get("gamemode")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let genres = vec!["Action".to_string(), "Adventure".to_string()]; // Placeholder
            let publisher = "Unknown".to_string(); // Placeholder
            let developer = "Unknown".to_string(); // Placeholder
            let summary = gameinfo
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let rating = 0.0; // Placeholder
            let release_dates = vec![0.into()]; // Placeholder
            let screenshots = vec!["https://example.com/screenshot.jpg".to_string()]; // Placeholder
            let videos = vec!["https://example.com/video.mp4".to_string()]; // Placeholder

            //println!("game: {:?}", gamemode);
            gamesdata.push(GameData {
                cover,
                icon,
                name,
                displayname,
                gamemode,
                genres,
                publisher,
                developer,
                summary,
                rating,
                release_dates,
                screenshots,
                videos,
                artworks,
            });
        }

        Ok(GameLibrary { games, gamesdata })
    }

    pub fn get_game(&self, name: &str) -> Option<&GameInfo> {
        self.games.iter().find(|game| game.name == name)
    }
}
