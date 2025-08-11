mod check_authorized;
mod config;
mod errors;

use crate::check_authorized::is_authorized;
use crate::errors::AppError;

use std::error::Error;
use tauri::{Manager, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use std::fs;


fn center_window(window: &tauri::WebviewWindow) -> Result<(), Box<dyn Error>> {
	let monitor = window.current_monitor()?
		.ok_or(AppError::new("didn't find any monitor"))?;
	let monitor_size = monitor.size();
	let window_size = window.inner_size()?;
	let x = (monitor_size.width - window_size.width) / 2;
	let y = (monitor_size.height - window_size.height) / 2;
	window.set_position(tauri::PhysicalPosition::new(x, y))?;
	Ok(())
}


fn setup_tools(app: tauri::AppHandle) -> Result<(), Box<dyn Error>> {
	println!("Installing tools ...");

	app.emit("progressbar_update", 0)?;

	let config = config::Config::default()?;

	for directory in vec![config.resources_junest_home_dir.clone(), config.temp_junest_home_dir.clone()] {
		// if !std::path::Path::new(&directory).exists() {
			fs::create_dir_all(directory)?;
		// }
	}

	let option = fs_extra::dir::CopyOptions::new().skip_exist(true);
	// use copy_items_with_progress instead
	// this copy as a lot of exclude, just delete the folder on the src folder
	// this will copy to config.temp_dir/junest, maybe use option.copy_inside
	fs_extra::copy_items(&vec![config.resources_junest_home_dir.clone()], config.temp_dir.clone(), &option)?;


	app.emit("progressbar_update", 60)?;

	tar::Archive::new(
		fs::File::open(config.resource_umu_archive_file.clone())?)
		.unpack(config.temp_dir.clone())?;
	
	app.emit("progressbar_update", 100)?;

	let splash_window = app.get_webview_window("splashscreen").unwrap();
	let main_window = app.get_webview_window("main").unwrap();
	splash_window.close().unwrap();
	main_window.show().unwrap();

	Ok(())
}


async fn setup_tools_wrapper(app: tauri::AppHandle) {
	if let Err(e) = setup_tools(app.clone()) {
		app.dialog()
			.message(format!("setup_tools failed: {}", e))
			.title("Error")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
	}
}


fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {

	if let Some(reason) = is_authorized() {
		app.dialog()
			.message(reason)
			.title("Access Denied")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
		return Ok(());
	}

	let window = app.get_webview_window("splashscreen").
		ok_or(AppError::new("didn't find 'splashscreen webview'"))?;

	if let Err(e) = center_window(&window) {
		eprintln!("failed to center window: {}", e);
	}

	window.show()?;

	// maybe use spawn_blocking instead, if there is lag maybe it's because of that
	tauri::async_runtime::spawn(setup_tools_wrapper(app.handle().clone()));

	Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![])
		.setup(setup_app)
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
