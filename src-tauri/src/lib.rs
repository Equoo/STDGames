
use clap::Parser;

use crate::config::CONFIG;
use crate::cli::{Cli, init_cli};
use crate::execution::GameExecution;
use crate::window::init::init_window;
use crate::library::load_library;

mod execution;
mod utils;
mod window;
mod cli;
mod config;
mod library;
mod debug;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let library = load_library(CONFIG.library.clone())
		.expect("Failed to load game library");
	let mut exec = GameExecution::new(library.clone());
	
	let cli = Cli::parse();
	if cli.command.is_some() { // do setup tools
		init_cli(&cli, &library, &mut exec).expect("Failed to run CLI command");
	} else {
		init_window(library, exec);
	}
}
