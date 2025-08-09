pub mod check_authorized;

use check_authorized::is_authorized;

use std::{collections::HashMap, error::Error, vec};
use tauri::{Manager, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};


use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Games {
    games: Vec<Game>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Game {
	slug: String,
	status: String,
    metadata: GameMetadata,
    launch: GameLaunchData,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameMetadata {
    idgb_id: Option<i32>,
    store_pages: Option<Vec<String>>,
	name: Option<String>,
	cover: Option<String>,
	icon: Option<String>,
	logo: Option<String>,
	description: Option<String>,
	tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameLaunchData {
    flags: String,
	environs: Option<HashMap<String, String>>,
	overlays: Vec<String>,
	start: Vec<String>,
	prestart: Option<Vec<String>>,
}

use std::fs;
use toml;

fn load_config(path: &str) -> Result<Games, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Games = toml::from_str(&content)?;
    Ok(config)
}

fn center_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) -> Result<(), Box<dyn Error>> {
	let monitor = window.current_monitor()?.ok_or("Failed to get current monitor")?;
	let monitor_size = monitor.size();
	let window_size = window.inner_size()?;
	let x = (monitor_size.width - window_size.width) / 2;
	let y = (monitor_size.height - window_size.height) / 2;
	window.set_position(tauri::PhysicalPosition::new(x, y))?;
	Ok(())
}


async fn splashscreen_loading(app: tauri::AppHandle) -> anyhow::Result<()> {
	let mut progress = 0;
	while progress < 100 {
		progress += 10;
		app.emit("splashscreen-progress", progress)?;
	}

	println!("Loading resources...");
	let games = load_config("/sgoinfre/dderny/private/STDGames/tmp.toml")
		.map_err(|e| println!("Failed to load config: {}", e));
	println!("Games loaded: {:?}", games);

	// After loading, close the splash screen and show the main window
	let splash_window = app.get_webview_window("splashscreen")
		.ok_or_else(|| anyhow::anyhow!("Failed to get splashscreen window"))?;
	let main_window = app.get_webview_window("main")
		.ok_or_else(|| anyhow::anyhow!("Failed to get main window"))?;
	splash_window.close()?;
	main_window.show()?;

	Ok(())
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {

	if let Some(reason) = is_authorized()
	{
		app.dialog()
			.message(reason)
			.title("Access Denied")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
		return Ok(());
	}

	// Initialize the splashscreen window
	let window = app.get_webview_window("splashscreen").ok_or("Failed to get splashscreen window")?;
	center_window(&window)?;
	window.show()?;

	// Thread to handle application resources loading
	tauri::async_runtime::spawn(splashscreen_loading(app.handle().clone()));

	Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_dialog::init())
		// .manage(Mutex::new(SetupState {
		//     progress: 0,
		//     finish: false,
		//     client_loaded: false
		// }))
		.invoke_handler(tauri::generate_handler![])
		.setup(setup_app)
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
