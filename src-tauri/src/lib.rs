pub mod utils;
pub mod game_library;
pub mod launcher;
pub mod desktop_icons;

use once_cell::sync::Lazy;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {	
	tauri::async_runtime::block_on(async {
		let user = env::var("USER").unwrap_or("".to_string());

		fs::create_dir_all(format!("/sgoinfre/{user}/.stdgames_saves/"))
			.expect("Erreur lors de la création du répertoire .stdgames_saves");

		let junest_dst = format!("/goinfre/{user}/.stdgames/junest");
		copy_recursively(Path::new("/sgoinfre/stdgames/.data/junest"), Path::new(&junest_dst)).await
			.expect("Erreur lors de la copie du répertoire .junest");

		let umu_path = format!("/goinfre/{user}/.stdgames/umu");
		if !Path::new(&umu_path).exists() {
			println!("Le répertoire umu n'existe pas, on le crée");
			let umu_dst_zip = format!("/goinfre/{user}/.stdgames/umu.zip");
			copy_recursively(Path::new("/sgoinfre/stdgames/.data/umu.zip"), Path::new(&umu_dst_zip)).await
				.expect("Erreur lors de la copie de umu.zip");

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
	});
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![game_state, get_game_library, launch_game, add_launcher_desktop_icon])
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
