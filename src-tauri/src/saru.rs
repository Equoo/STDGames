#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod game_library;
pub mod launcher;
pub mod desktop_icons;

use serde::Deserialize;
use serde::Serialize;
use tauri::async_runtime::spawn;
use tauri::{Manager, State};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use utils::copy_recursively;
use std::env;
use std::path::Path;
use std::sync::Arc;
use crate::game_library::GameLibrary;
use crate::launcher::Launcher;
use crate::desktop_icons::add_launcher_desktop_icon;
use std::fs;

static LAUNCHER: Lazy<Arc<Launcher>> = Lazy::new(|| {
    let library = Arc::new(GameLibrary::new().expect("Failed to create game library"));
    Arc::new(Launcher::new(library))
});

#[tauri::command]
fn game_state() -> Option<String> {
	let launcher = &LAUNCHER;
    launcher.is_game_running()
}

#[tauri::command]
fn get_game_library() -> Arc<GameLibrary> {
    let _launcher = &LAUNCHER;
    _launcher.library.clone()
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct SetupState {
    progress: u8,
    finish: bool,
}

#[tauri::command]
async fn get_setup_state(state: State<'_, Mutex<SetupState>>) -> Result<SetupState, ()>	{
	let state = state.lock().await;
	Ok(SetupState {
		progress: state.progress,
		finish: state.finish,
	})
}

#[tauri::command]
async fn get_gameprocess_state() -> Result<Option<String>, ()> {
	let launcher = &LAUNCHER;
	let state = launcher.is_game_running();
	Ok(state)
}
fn main() {
    tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![game_state, get_game_library, get_setup_state, get_gameprocess_state])

        .setup(|app| {
            println!("✅ App is setting up...");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
