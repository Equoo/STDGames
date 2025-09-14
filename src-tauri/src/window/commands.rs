use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use crate::config::Config;
use crate::execution::GameExecution;
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

#[tauri::command]
pub fn launch_game(exec: State<'_, Mutex<GameExecution>>, game: String) -> Result<(), String> {
	let mut exec = exec.lock().map_err(|e| e.to_string())?;
    exec.start(&game).map_err(|e| e.to_string())?;
    Ok(())
}
