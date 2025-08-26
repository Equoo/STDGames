use std::ops::Deref;
use std::path::Path;
use tauri::State;

use crate::config::Config;
use crate::library::Game;

// TODO: update the database using: `update-desktop-database ~/.local/share/applications`
// TODO: add a desktop file action to remove itself 
#[tauri::command]
pub fn add_launcher_to_desktop(config: State<'_, Config>) -> Result<(), String> {
	println!("executed !!!");
	let dest = format!("{}/.local/share/applications/stdgames.desktop", config.user_home);
	if Path::new(&dest).exists() {
		fs_extra::remove_items(&[&dest]).map_err(|e| e.to_string())?;
	}
	std::os::unix::fs::symlink(&config.desktop_file, &dest).map_err(|e| e.to_string())?;
	Ok(())
}

#[tauri::command]
pub fn get_game_library(lib: State<'_, Vec<Game>>) -> Result<Vec<Game>, String> {
	let lib = lib.inner();
	Ok(lib.clone())
}

// #[tauri::command]
// pub fn get_game_list(config: State<'_, Config>) -> Result<Value, String> {
//     return Ok(load_data_from_toml(config.game_list_file));
// }

// #[tauri::command]
// pub fn launch_game(config: State<'_, Config>, launch: GameLaunch) -> Result<(), String> {
//     // maybe spawn a new thread
//     // how to be able to kill it afterward
//     return Ok(launch_game(launch));
// }
