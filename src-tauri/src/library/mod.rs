use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{collections::HashMap, fs, path::Path};
use toml;

use crate::config::CONFIG;
use crate::library::igdb_client::IgdbClient;
use crate::library::steamdb::SteamAssetsClient;

#[derive(Debug, Deserialize, Serialize)]
pub struct Games {
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Game {
    pub slug: String,
    pub status: String,
    pub metadata: GameMetadata,
    pub launch: GameLaunchData,
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameLaunchData {
    pub proton: Option<String>,
    pub winetricks: Option<Vec<String>>,
    pub noruntime: Option<bool>,
    pub epicgame: Option<bool>,
    pub environs: Option<HashMap<String, String>>,
    pub overlays: Vec<String>,
    pub start: Vec<String>,
    pub prestart: Option<Vec<String>>,
}

impl GameLaunchData {
    fn get_abs_command(mut command: Vec<String>) -> Vec<String> {
        command[0] = if Path::new(&command[0]).is_absolute() {
            command[0].clone()
        } else {
            format!("/tmp/{}/stdgames/work/{}", CONFIG.username, command[0])
        };
        command
    }

    pub fn replace_vars(&mut self, vars: &HashMap<String, String>) {
        for (k, v) in vars {
            let mut vk = k.clone();
            vk.insert(0, '$');
            self.start.iter_mut().for_each(|s| {
                *s = s.replace(&vk, v);
            });
            if let Some(pre) = &mut self.prestart {
                pre.iter_mut().for_each(|s| {
                    *s = s.replace(&vk, v);
                });
            }
            self.overlays.iter_mut().for_each(|s| {
                *s = s.replace(&vk, v);
            });
            if let Some(environs) = &mut self.environs {
                if let Some(ev) = environs.get_mut(k) {
                    *ev = ev.replace(&vk, v);
                }
            }
        }

        self.start = Self::get_abs_command(self.start.clone());
        if let Some(pre) = &mut self.prestart {
            *pre = Self::get_abs_command(pre.clone());
        }
    }
}

fn format_toml_error(content: &str, error: &toml::de::Error, file_path: Option<&str>) -> String {
    let mut output = String::new();

    // Header with file info
    if let Some(path) = file_path {
        writeln!(output, "┌─ Error in {}", path).unwrap();
    } else {
        writeln!(output, "┌─ TOML Parse Error").unwrap();
    }

    writeln!(output, "│").unwrap();

    // Get error details
    let message = error.message();
    let span = error.span();

    if let Some(span) = span {
        let lines: Vec<&str> = content.lines().collect();
        let start_line = content[..span.start].lines().count();
        let end_line = content[..span.end].lines().count();
        let start_col = content[..span.start].lines().last().map_or(0, |l| l.len());
        let end_col = if start_line == end_line {
            start_col + (span.end - span.start)
        } else {
            content[..span.end].lines().last().map_or(0, |l| l.len())
        };

        // Show error message
        writeln!(output, "│ ❌ {}", message).unwrap();
        writeln!(output, "│").unwrap();

        // Show line numbers and context
        let context_start = start_line.saturating_sub(2);
        let context_end = (end_line + 2).min(lines.len());

        for (i, line) in lines
            .iter()
            .enumerate()
            .take(context_end)
            .skip(context_start)
        {
            let line_num = i + 1;
            let is_error_line = line_num + 1 > start_line && line_num + 1 <= end_line + 1;

            if is_error_line {
                // Error line with highlighting
                writeln!(output, "│ {:3} │ {}", line_num, line).unwrap();

                // Add error pointer
                if line_num + 1 == start_line + 1 {
                    let pointer_start = start_col;
                    let pointer_len = if start_line == end_line {
                        (end_col - start_col).max(1)
                    } else {
                        line.len() - start_col
                    };

                    write!(output, "│     │ ").unwrap();
                    write!(output, "{}", " ".repeat(pointer_start)).unwrap();
                    write!(output, "{}", "^".repeat(pointer_len.max(1))).unwrap();
                    writeln!(output, " {}", message).unwrap();
                }
            } else {
                // Context line
                writeln!(output, "│ {:3} │ {}", line_num, line).unwrap();
            }
        }

        writeln!(output, "│").unwrap();
        writeln!(
            output,
            "└─ at line {}, column {}",
            start_line + 1,
            start_col + 1
        )
        .unwrap();
    } else {
        // No span information available
        writeln!(output, "│ ❌ {}", message).unwrap();
        writeln!(output, "│").unwrap();
        writeln!(output, "└─ Unable to determine exact location").unwrap();
    }

    output
}

async fn steam_game_data(
    client: &SteamAssetsClient,
    meta: &mut GameMetadata,
    appid: u32,
) -> Result<()> {
    match client.get_game_assets_with_icons(appid).await {
        Ok(assets) => {
            meta.name = Some(assets.name);
            meta.description = assets.description;
            meta.tags = Some(assets.genres);
            meta.short_description = assets.short_description;
            meta.logo = Some(assets.logo);
            meta.icon = Some(assets.icon);
            meta.hero = Some(assets.library_hero);
            meta.cover = Some(assets.library_600x900);
            meta.screenshots = if !assets.screenshots.is_empty() {
                Some(assets.screenshots)
            } else {
                None
            };
            meta.movies_thumbnails = if !assets.movies.is_empty() {
                Some(assets.movies.iter().map(|m| m.thumbnail.clone()).collect())
            } else {
                None
            };
            meta.movies = if !assets.movies.is_empty() {
                Some(
                    assets
                        .movies
                        .iter()
                        .map(|m| {
                            m.webm_urls
                                .get("max")
                                .or_else(|| m.mp4_urls.get("max"))
                                .or_else(|| m.webm_urls.get("480"))
                                .or_else(|| m.mp4_urls.get("480"))
                                .cloned()
                                .unwrap_or_default()
                        })
                        .collect(),
                )
            } else {
                None
            };
        }
        Err(e) => {
            println!("Error fetching assets with icons: {}", e);
        }
    }

    Ok(())
}

pub async fn load_api_data(games: &mut Vec<Game>) -> Result<()> {
    let steam_client = SteamAssetsClient::new(
        Some("19A33BB7E5367795078D0F3BFB663BD9".to_string()),
        "french".to_string(),
    );
    let mut igdb_client = IgdbClient::new(
        "rggouo5m4dsiowf6upejcgzyskt2vj",
        "pr902t650n6wrs7vax0fyk9twjjbuk",
    )
    .await?; // TODO: move this to config

    igdb_client.load_igdb_games(&games).await?;

    for game in games {
        if let Some(id) = game.metadata.appid {
            steam_game_data(&steam_client, &mut game.metadata, id).await?;
        }
        if let Some(id) = game.metadata.igdbid {
            igdb_client.fill_game_metadata(&mut game.metadata);
        }
    }
    Ok(())
}

pub fn load_library(path: String) -> Result<Vec<Game>> {
    let content = fs::read_to_string(&path)?;

    let mut config: Games = toml::from_str(&content).map_err(|e| {
        let error_msg = format_toml_error(&content, &e, Some(path.as_str())); // or None if no file path
        anyhow!("\n\n{}", error_msg)
    })?;

    Ok(config.games)
}
