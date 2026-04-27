use std::{collections::{BTreeMap, HashMap}, ops::Deref, path::Path};

use anyhow::{Result, anyhow};
use clap::error::Result;

use crate::{
    execution::GameProcess,
    library::Game,
    methods::{LaunchMethod, LaunchMode},
    store::{SteamStore, Store},
    utils::format_toml_error,
};

struct AppState {
    games: Vec<Game>,
    store: BTreeMap<LaunchMethod, Box<dyn Store>>,
    active: Option<GameProcess>,
}

impl AppState {
    pub fn load_library(&mut self, path: Path) -> Result<()> {
        let content = fs::read_to_string(&path)?;

        self.games = toml::from_str(&content).map_err(|e| {
            let error_msg = format_toml_error(&content, &e, Some(path.into()));
            anyhow!("\n\n{}", error_msg)
        })?;

        Ok(())
    }

    pub fn launch(&self, slug: &str, method: LaunchMethod) -> Result<GameProcess> {
        if let Some(store) = self.store.get(&method) {
            
        }
    }

    pub fn kill(slug: &str) -> Result<()> {
        Ok(())
    }

    pub fn is_running(slug: &str) -> bool {
        false
    }

    pub fn open_store(name: &str) -> Result<()> {
        Ok(())
    }
    pub fn close_store(name: &str) -> Result<()> {
        Ok(())
    }
}
