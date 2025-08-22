
use crate::config::{Config, CONFIG};
use crate::cli::{Cli, init_cli};
use crate::execution::GameExecution;
use crate::window::init_window;
use crate::library::load_library;

mod execution;
mod utils;
mod window;
mod cli;
mod config;
mod library;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	CONFIG = Config::default();

	let mut exec = GameExecution::new();
	let library = load_library(CONFIG.library)
		.expect("Failed to load game library");
	
	let cli = Cli::parse();
	if cli.command.is_some() { // do setup tools
		init_cli(&cli, &library, &mut exec);
	} else {
		init_window(&library, &mut exec);
	}
}
