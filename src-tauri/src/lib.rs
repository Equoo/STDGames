pub mod utils;
pub mod game_library;
pub mod launcher;
pub mod desktop_icons;
pub mod check_authorized;

use serde::Deserialize;
use serde::Serialize;
use tauri::async_runtime::spawn;
use tauri::{Manager, State};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use utils::copy_recursively;
use check_authorized::is_authorized;
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
	client_loaded: bool
}

async fn setup(app: tauri::AppHandle) {
	let state = app.state::<Mutex<SetupState>>();
	let mut state_lock = state.lock().await;
	state_lock.progress = 0;
	state_lock.finish = false;
	drop(state_lock);
	println!("Lancement de la configuration...");

	let tmp_path = format!("/tmp/{}/.stdgames", env::var("USER").unwrap_or("".to_string()));
	if !Path::new(&tmp_path).exists() {
		fs::create_dir_all(tmp_path.clone()).expect("Erreur lors de la création du répertoire .stdgames");
	}

	let user = env::var("USER").unwrap_or("".to_string());
	fs::create_dir_all(format!("/sgoinfre/{user}/.stdgames_saves/"))
		.expect("Erreur lors de la création du répertoire .stdgames_saves");
	let junest_dst = format!("/tmp/{user}/.stdgames/junest");
	fs::create_dir_all(format!("{junest_dst}/usr"))
		.expect("Erreur lors de la création du répertoire junest");

	std::process::Command::new("rsync")
		.arg("-aAXHv")
		.arg("--numeric-ids")
		.arg("--exclude=/dev/")
		.arg("--exclude=/proc/")
		.arg("--exclude=/sys/")
		.arg("--exclude=/tmp/")
		.arg("--exclude=/mnt/")
		.arg("--exclude=/media/*")
		.arg("--exclude=/lost+found")
		.arg("--exclude=/usr")
		.arg("/sgoinfre/stdgames/.ressources/junest/")
		.arg(&junest_dst)
		.output()
		.expect("Erreur lors de l'exécution de rsync");
	std::process::Command::new("rsync")
		.arg("-aAXHv")
		.arg("--numeric-ids")
		.arg("/sgoinfre/stdgames/.ressources/junest_tmp_usr/")
		.arg(format!("{junest_dst}/usr"))
		.output()
		.expect("Erreur lors de l'exécution de rsync");

	let mut state_lock = state.lock().await;
	state_lock.progress = 60;
	drop(state_lock);

	let umu_path = format!("/tmp/{user}/.stdgames/umu");
	if !Path::new(&umu_path).exists() {
		println!("Le répertoire umu n'existe pas, on le crée");
		let umu_dst_zip = format!("/tmp/{user}/.stdgames/umu.zip");
		copy_recursively(Path::new("/sgoinfre/stdgames/.ressources/umu.zip"), Path::new(&umu_dst_zip)).await
			.expect("Erreur lors de la copie de umu.zip");
		let mut state_lock = state.lock().await;
		state_lock.progress = 75;
		drop(state_lock);

		let umu_zip_path = Path::new(&umu_dst_zip);
		let umu_dst = format!("/tmp/{user}/.stdgames/");
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

	loop {
		let state_lock = state.lock().await;
		if state_lock.client_loaded {
			break;
		}
		drop(state_lock);
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
	}

	let splash_window = app.get_webview_window("splashscreen").unwrap();
	let main_window = app.get_webview_window("main").unwrap();
	splash_window.close().unwrap();
	main_window.show().unwrap();
}

#[tauri::command]
async fn get_setup_state(state: State<'_, Mutex<SetupState>>) -> Result<SetupState, ()>	{
	let state = state.lock().await;
	Ok(SetupState {
		progress: state.progress,
		finish: state.finish,
		client_loaded: state.client_loaded
	})
}

#[tauri::command]
async fn set_client_loaded(state: State<'_, Mutex<SetupState>>) -> Result<(), ()> {
	let mut state = state.lock().await;
	state.client_loaded = true;
	return Ok(());
}

#[tauri::command]
async fn get_gameprocess_state() -> Result<Option<String>, ()> {
	let launcher = &LAUNCHER;
	let state = launcher.is_game_running();
	Ok(state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.manage(Mutex::new(SetupState {
            progress: 0,
            finish: false,
			client_loaded: false
        }))
		.invoke_handler(tauri::generate_handler![game_state, get_game_library, launch_game, add_launcher_desktop_icon, get_setup_state, get_gameprocess_state, set_client_loaded])
		.setup(|app| {


            if let Some(msg) = is_authorized() {
                println!("access denied: {} !!!", msg);
                std::process::exit(1);
            }


            // Spawn setup as a non-blocking task so the windows can be
            // created and ran while it executes
			let window = app.get_webview_window("splashscreen").unwrap();

			let monitor = window.current_monitor()?.unwrap();
            let monitor_size = monitor.size();
            let monitor_position = monitor.position();

            // Calculate the position to truly center the window
            let new_x = monitor_position.x + (monitor_size.width as i32 / 2) - 200;
            let new_y = monitor_position.y + (monitor_size.height as i32 / 2) - 200;

            // Move the window to the computed position
            window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
				x: new_x as f64,
				y: new_y as f64,
			})).unwrap();
			
            spawn(setup(app.handle().clone()));
            // The hook expects an Ok result
            Ok(())
        })
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
