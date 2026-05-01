
use clap::Parser;
use std::{collections::{BTreeMap, HashMap}, path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

use crate::{cli::{Cli, init_cli}, config::AppConfig};
use crate::window::init::init_window;
use crate::library::load_library;

use crate::{
    library::Game, managers::{games::GameProcessManager, stores::StoreProcessManager}
};

mod utils;
mod window;
mod cli;
mod config;

mod download;
mod runtime;
mod library;
mod methods;
mod store;
mod managers;


pub struct AppState {
    pub config: Arc<AppConfig>,
    pub games: Arc<RwLock<HashMap<String, Game>>>,
    pub store_manager: Arc<RwLock<StoreProcessManager>>,
    pub game_manager: Arc<RwLock<GameProcessManager>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let library = load_library(PathBuf::from_str(CONFIG.library).into())
		.expect("Failed to load game library");
	let mut exec = GameExecution::new(library.clone());
	
	let cli = Cli::parse();
	if cli.command.is_some() { // do setup tools
		init_cli(&cli, &library, &mut exec).expect("Failed to run CLI command");
	} else {
		init_window(library, exec);
	}
}
