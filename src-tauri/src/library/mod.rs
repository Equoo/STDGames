use anyhow::{Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::{collections::HashMap, fs, path::Path};
use toml;

use crate::config::CONFIG;

mod metadata;

#[derive(Debug, Deserialize, Serialize)]
pub struct Games {
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Game {
    pub slug: String,
    pub metadata: GameMetadata,
    pub methods: Vec<String>,
    pub proton: Option<String>,
    pub is_local: Option<bool>,
    pub environs: Option<HashMap<String, String>>,
    pub srcs: Vec<String>,
    pub cmd: Vec<String>,
    pub precmd: Option<Vec<String>>,
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
pub struct ApiClient {
    pub id: u32,
    pub client: String,
}

impl Game {
    fn abs_command(root: &String, command: &Vec<String>) {
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
                    str.replace(format!("${}", m.as_str()), value);
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

        Self::abs_command(root, &self.cmd);
        if let Some(cmd) = &mut self.precmd {
            cmd.iter_mut().for_each(|v| {
                Self::expand_environs(environs, v);
            });

            Self::abs_command(root, &cmd);
        }
    }
}

