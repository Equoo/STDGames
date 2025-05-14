pub mod utils;
pub mod game_library;
pub mod launcher;
pub mod desktop_icons;

use serde::Deserialize;
use serde::Serialize;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Manager, State};
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
fn launch_game(game: String) {
	let launcher = &LAUNCHER;
	tauri::async_runtime::block_on(async {
		println!("Lancement du jeu : {}", game);
    	let _ = launcher.launch_game(&game).await;
	});
}

#[tauri::command]
fn game_state() -> bool {
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

async fn setup(app: tauri::AppHandle) {
	let state = app.state::<Mutex<SetupState>>();
	let mut state_lock = state.lock().await;
	state_lock.progress = 0;
	state_lock.finish = false;
	drop(state_lock);
	println!("Lancement de la configuration...");

	let user = env::var("USER").unwrap_or("".to_string());
	fs::create_dir_all(format!("/sgoinfre/{user}/.stdgames_saves/"))
		.expect("Erreur lors de la création du répertoire .stdgames_saves");

	let junest_dst = format!("/goinfre/{user}/.stdgames/junest");
	copy_recursively(Path::new("/sgoinfre/stdgames/.ressources/junest"), Path::new(&junest_dst)).await
		.expect("Erreur lors de la copie du répertoire .junest");
	let mut state_lock = state.lock().await;
	state_lock.progress = 60;
	drop(state_lock);

	let umu_path = format!("/goinfre/{user}/.stdgames/umu");
	if !Path::new(&umu_path).exists() {
		println!("Le répertoire umu n'existe pas, on le crée");
		let umu_dst_zip = format!("/goinfre/{user}/.stdgames/umu.zip");
		copy_recursively(Path::new("/sgoinfre/stdgames/.ressources/umu.zip"), Path::new(&umu_dst_zip)).await
			.expect("Erreur lors de la copie de umu.zip");
		let mut state_lock = state.lock().await;
		state_lock.progress = 75;
		drop(state_lock);

		let umu_zip_path = Path::new(&umu_dst_zip);
		let umu_dst = format!("/goinfre/{user}/.stdgames/");
		if umu_zip_path.exists() {
			std::process::Command::new("tar")
				.arg("-jxf")
				.arg(umu_dst_zip)
				.arg("-C")
				.arg(umu_dst)
				.output()
				.expect("Erreur lors de l'extraction de umu.zip");
		}
	}
	let mut state_lock = state.lock().await;
	state_lock.progress = 100;
	drop(state_lock);
}

#[tauri::command]
async fn get_setup_state(state: State<'_, Mutex<SetupState>>) -> Result<SetupState, ()>	{
	let state = state.lock().await;
	println!("get_setup_state: {:?}", state);
	Ok(SetupState {
		progress: state.progress,
		finish: state.finish,
	})
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {	
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.manage(Mutex::new(SetupState {
            progress: 0,
            finish: false,
        }))
		.invoke_handler(tauri::generate_handler![game_state, get_game_library, launch_game, add_launcher_desktop_icon, get_setup_state])
		.setup(|app| {
            // Spawn setup as a non-blocking task so the windows can be
            // created and ran while it executes
            spawn(setup(app.handle().clone()));
            // The hook expects an Ok result
            Ok(())
        })
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
