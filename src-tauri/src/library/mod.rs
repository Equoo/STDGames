use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{collections::HashMap, fs, path::Path};
use toml;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameMetadataAPI {
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
    pub api: Option<ApiClient>,
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
    pub before: Option<Vec<String>>,
    pub start: Vec<String>,
    pub prestart: Option<Vec<String>>,
}

fn replace_all_vars(strings: &mut [String], vars: &HashMap<String, String>) {
    for (k, v) in vars {
        let mut vk = k.clone();
        vk.insert(0, '$');
        for s in strings.iter_mut() {
            *s = s.replace(&vk, v);
        }
    }
}

impl GameLaunchData {
    /// `command[0]` is exec'd from inside the junest/bwrap sandbox, so a
    /// relative program name must be made absolute using the sandbox-visible
    /// work dir (bare `/tmp/stdgames/work`, no username - see
    /// `Config::build_sandbox_vars`), not the host-side path.
    fn get_abs_command(mut command: Vec<String>) -> Vec<String> {
        command[0] = if Path::new(&command[0]).is_absolute() {
            command[0].clone()
        } else {
            format!("/tmp/stdgames/work/{}", command[0])
        };
        command
    }

    /// `host_vars` are used for `overlays`: bwrap resolves `--overlay-src`
    /// itself, on the host, before the sandbox's mount namespace exists.
    ///
    /// `sandbox_vars` are used for `start`/`prestart`/`before`/`environs`:
    /// those run as argv/env inside the already-constructed sandbox, where
    /// `/tmp/<username>`-rooted host paths aren't valid (see
    /// `Config::build_sandbox_vars`).
    pub fn replace_vars(
        &mut self,
        host_vars: &HashMap<String, String>,
        sandbox_vars: &HashMap<String, String>,
    ) {
        replace_all_vars(&mut self.start, sandbox_vars);
        if let Some(pre) = &mut self.prestart {
            replace_all_vars(pre, sandbox_vars);
        }
        if let Some(before) = &mut self.before {
            replace_all_vars(before, sandbox_vars);
        }
        replace_all_vars(&mut self.overlays, host_vars);

        if let Some(environs) = &mut self.environs {
            for (k, ev) in environs.iter_mut() {
                if let Some(v) = sandbox_vars.get(k) {
                    let mut vk = k.clone();
                    vk.insert(0, '$');
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

fn get_asset(asset_id: Option<usize>, assets: &Vec<String>) -> Option<String> {
    match asset_id {
        Some(id) if id < assets.len() => Some(assets[id].clone()),
        _ => None,
    }
}

pub async fn load_api_data(games: &mut Vec<Game>) -> Result<()> {
    let client = reqwest::Client::new();

    println!("Sending API request...");
    let res = client
        .post("http://37.59.106.4:2356/api/data")
        .json(
            &games
                .iter()
                .filter_map(|g| {
                    g.metadata
                        .api
                        .as_ref()
                        .map(|api| (g.slug.clone(), api).into())
                })
                .collect::<HashMap<String, &ApiClient>>(),
        )
        .send()
        .await?;

    println!("API response status: {}", res.status());

    let mut api_data: HashMap<String, GameMetadataAPI> = res.json().await?;

    for game in games.iter_mut() {
        if let Some(data) = api_data.get_mut(&game.slug) {
            data.assets.iter_mut().for_each(|asset| {
                *asset = asset.replace("resources/", "http://37.59.106.4:2356/cdn/");
            });

            game.metadata.name = game.metadata.name.clone().or(data.name.clone());
            game.metadata.description = game
                .metadata
                .description
                .clone()
                .or(data.description.clone());
            game.metadata.short_description = game
                .metadata
                .short_description
                .clone()
                .or(data.short_description.clone());
            game.metadata.icon = game
                .metadata
                .icon
                .clone()
                .or(get_asset(data.icon, &data.assets));
            game.metadata.logo = game
                .metadata
                .logo
                .clone()
                .or(get_asset(data.logo, &data.assets));
            game.metadata.hero = game
                .metadata
                .hero
                .clone()
                .or(get_asset(data.hero, &data.assets));
            game.metadata.cover = game
                .metadata
                .cover
                .clone()
                .or(get_asset(data.cover, &data.assets));
            game.metadata.screenshots =
                game.metadata
                    .screenshots
                    .clone()
                    .or(match &data.screenshots {
                        Some(screenshots) => Some(
                            screenshots
                                .iter()
                                .map(|id| data.assets[*id].clone())
                                .collect(),
                        ),
                        _ => None,
                    });
            game.metadata.movies = game.metadata.movies.clone().or(match &data.movies {
                Some(movies) => Some(movies.iter().map(|id| data.assets[*id].clone()).collect()),
                _ => None,
            });
            game.metadata.movies_thumbnails =
                game.metadata
                    .movies_thumbnails
                    .clone()
                    .or(match &data.movies_thumbnails {
                        Some(thumbnails) => Some(
                            thumbnails
                                .iter()
                                .map(|id| data.assets[*id].clone())
                                .collect(),
                        ),
                        _ => None,
                    });
            game.metadata.tags = game.metadata.tags.clone().or(data.tags.clone());
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
