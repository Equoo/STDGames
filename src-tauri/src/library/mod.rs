use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

use crate::{methods::ModeId, utils::format_toml_error};

mod metadata;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct Game {
    pub slug: String,
    pub metadata: GameMetadata,
    pub methods: Vec<ModeId>,
    pub proton: Option<String>,
    pub force_dl: bool,
    pub environs: HashMap<String, String>,
    pub srcs: Vec<String>,
    pub cmd: Vec<String>,
    pub precmd: Vec<String>,
    pub prelaunch: Option<Vec<String>>,
    pub postlaunch: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct GameMetadata {
    pub api: Option<ApiClient>,
    pub store_pages: Vec<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub hero: Option<String>,
    pub cover: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub screenshots: Vec<String>,
    pub movies: Vec<String>,
    pub movies_thumbnails: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiClient {
    pub id: u32,
    pub client: String,
}

impl Game {
    fn abs_command(root: &String, command: &mut Vec<String>) {
        command[0] = if Path::new(&command[0]).is_absolute() {
            command[0].clone()
        } else {
            format!("{root}/{}", command[0])
        };
    }

    fn expand_environs(environs: &HashMap<String, String>, str: &String) -> Result<()> {
        let re = Regex::new(r"[^\\]\$([a-zA-Z_][a-zA-Z0-9_]*)")?;

        re.captures_iter(str).for_each(|c| {
            if let Some(m) = c.get(1) {
                if let Some(value) = environs.get(m.as_str()) {
                    str.replace(&format!("${}", m.as_str()), value);
                }
            }
        });

        Ok(())
    } // TODO: Check working

    pub fn update_paths(&mut self, environs: &HashMap<String, String>, root: &String) {
        self.srcs.iter_mut().for_each(|v| {
            Self::expand_environs(environs, v);
        });

        self.cmd.iter_mut().for_each(|v| {
            Self::expand_environs(environs, v);
        });

        Self::abs_command(root, &mut self.cmd);

        self.precmd.iter_mut().for_each(|v| {
            Self::expand_environs(environs, v);
        });

        Self::abs_command(root, &mut self.precmd);
    }
}

pub fn load_library(path: &Path) -> Result<HashMap<String, Game>> {
    let content = fs::read_to_string(path)?;

    Ok(
        toml::from_str::<Vec<Game>>(&content).map_err(|e| {
            let error_msg = format_toml_error(&content, &e, path.to_str());
            anyhow!("\n\n{}", error_msg)
        })?
        .into_iter()
        .map(|v| (v.slug.clone(), v))
        .collect()
    )
}
