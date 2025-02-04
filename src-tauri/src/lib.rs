pub mod utils;
pub mod game_library;
pub mod launcher;
pub mod desktop_icons;

use once_cell::sync::Lazy;
use std::sync::Arc;
use crate::game_library::{GameLibrary, GameInfo};
use crate::launcher::Launcher;
use crate::desktop_icons::add_launcher_desktop_icon;

const JUNEST_HOME: &str = "/sgoinfre/dderny/.junest";
const JUNEST_PATH: &str = "/sgoinfre/dderny/junest/bin/junest";
const LEGENDARY_LAUNCH: &str = "legendary launch";
const GAMES_PATH: &str = "/sgoinfre/42GamingNight/.STDGames";

static LAUNCHER: Lazy<Arc<Launcher>> = Lazy::new(|| {
    let library = Arc::new(GameLibrary::new().expect("Failed to create game library"));
    Arc::new(Launcher::new(library))
});

#[tauri::command]
fn launch_game(game: String) {
	let launcher = &LAUNCHER;
    let _ = launcher.launch_game(&game);
}

#[tauri::command]
fn game_state() -> bool {
	let launcher = &LAUNCHER;
    launcher.is_game_running()
}

#[tauri::command]
fn get_game_library() -> Vec<GameInfo> {
    let _launcher = &LAUNCHER;
    _launcher.library.games.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![game_state, get_game_library, launch_game, add_launcher_desktop_icon])
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
