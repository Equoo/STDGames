use std::path::Path;
use tauri::State;

use crate::config::CONFIG;
use crate::library::Game;
use crate::window::init::AppState;

// TODO: update the database using: `update-desktop-database ~/.local/share/applications`
// TODO: add a desktop file action to remove itself
#[tauri::command]
pub fn add_launcher_to_desktop() -> Result<(), String> {
    println!("executed !!!");
    let dest = format!(
        "{}/.local/share/applications/stdgames.desktop",
        CONFIG.user_home
    );
    println!("adding symlink '{}' -> '{}'", dest, CONFIG.desktop_file);
    if Path::new(&dest).exists() {
        fs_extra::remove_items(&[&dest]).map_err(|e| e.to_string())?;
    }
    match std::os::unix::fs::symlink(&CONFIG.desktop_file, &dest) {
        Err(err) => eprintln!("error: {:#?}", err),
        _ => {},
    }
    Ok(())
}

#[tauri::command]
pub fn get_game_library(app_state: State<'_, AppState>) -> Result<Vec<Game>, String> {
    while (!app_state
        .setup_finished
        .lock()
        .map_err(|e| e.to_string())?
        .to_owned())
    {
        1;
    }
    let lib = app_state.games.lock().map_err(|e| e.to_string())?;
    Ok(lib.to_vec())
}

#[tauri::command]
pub fn launch_game(app_state: State<'_, AppState>, game: String) -> Result<(), String> {
    let mut exec = app_state.exec.lock().map_err(|e| e.to_string())?;
    exec.start(&game).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_running_game(app_state: State<'_, AppState>) -> Result<String, String> {
    let mut exec = app_state.exec.lock().map_err(|e| e.to_string())?;
    if exec.is_running() {
        Ok(exec.running.as_ref().unwrap().name.clone())
    } else {
        Ok("".to_string())
    }
}
