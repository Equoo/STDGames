mod check_authorized;
mod setup_tools;
mod commands;
mod config;
mod errors;
mod library;
mod game_execution;
mod copy_directory;

use std::error::Error;
use tauri::{Builder, Manager, App, AppHandle, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use clap::{Parser, Subcommand};

use crate::check_authorized::is_authorized;
use crate::setup_tools::setup_tools;
use crate::errors::AppError;
use crate::config::Config;


#[derive(Parser, Debug)]
#[command(
    name = "stdgames",
    version,
    about = "Stdgames launcher by zsonie, tdaclin and dderny.",
    author = "zsonie, tdaclin, dderny"
)]
struct Cli {
    /// Use a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a game from the stdgames repository
    Run {
        /// Game name
        game: String,
    },

    /// Run bash with the game's config
    Bash {
        /// Game name
        game: String,
    },

    /// Run a game with a custom config file
    RunConfig {
        /// Path to TOML config file
        file: String,
    },

    /// Run bash with a custom config file
    BashConfig {
        /// Path to TOML config file
        file: String,
    },

    /// Enter the Junest environment
    Junest,
}


async fn setup_tools_wrapper(app: AppHandle) {
	if let Err(e) = setup_tools(app.clone(), app.state()) {
		app.dialog()
			.message(format!("setup_tools failed: {}", e))
			.title("Error")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
	}
}

fn center_window(window: &WebviewWindow) -> Result<(), Box<dyn Error>> {
	let monitor = window.current_monitor()?
		.ok_or(AppError::new("didn't find any monitor"))?;
	let monitor_size = monitor.size();
	let window_size = window.inner_size()?;
	let x = (monitor_size.width - window_size.width) / 2;
	let y = (monitor_size.height - window_size.height) / 2;
	window.set_position(tauri::PhysicalPosition::new(x, y))?;
	Ok(())
}

fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
	if let Some(reason) = is_authorized() {
		app.dialog()
			.message(reason)
			.title("Access Denied")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
		return Ok(());
	}

	let window = app.get_webview_window("splashscreen")
		.ok_or(AppError::new("didn't find the 'splashscreen' webview"))?;

	if let Err(e) = center_window(&window) {
		eprintln!("failed to center window: {}", e);
	}

	// maybe use spawn_blocking instead, if there is lag maybe it's because of that
	tauri::async_runtime::spawn(setup_tools_wrapper(app.handle().clone()));

	Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	let config = Config::default()
		.expect("Failed to load configuration");
		
	let cli = Cli::parse();
	match cli.command {
        Some(Commands::Run { game }) => {
            println!("Running game: {}", game);
        }
        Some(Commands::Bash { game }) => {
            println!("Starting bash with game config: {}", game);
        }
        Some(Commands::RunConfig { file }) => {
            println!("Running game with config file: {}", file);
        }
        Some(Commands::BashConfig { file }) => {
            println!("Running bash with config file: {}", file);
        }
        Some(Commands::Junest) => {
            println!("Entering Junest environment...");
        }
        None => {
			Builder::default()
				.plugin(tauri_plugin_opener::init())
				.plugin(tauri_plugin_dialog::init())
				.manage(config)
				.invoke_handler(tauri::generate_handler![
					commands::add_launcher_to_desktop
				])
				.setup(setup_app)
				.run(tauri::generate_context!())
				.expect("Erreur lors du lancement de Tauri");
		}
    }
}
