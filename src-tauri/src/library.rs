use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use toml;

use crate::config::CONFIG;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

pub fn load_library(path: String) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Games = toml::from_str(&content)?;
    Ok(config.games)
}