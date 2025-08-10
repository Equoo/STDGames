mod check_authorized;
mod errors;

use check_authorized::is_authorized;
use errors::AppError;

use std::error::Error;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};


fn center_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) -> Result<(), Box<dyn Error>> {
	let monitor = window.current_monitor()?
		.ok_or(AppError::new("didn't find any monitor"))?;
	let monitor_size = monitor.size();
	let window_size = window.inner_size()?;
	let x = (monitor_size.width - window_size.width) / 2;
	let y = (monitor_size.height - window_size.height) / 2;
	window.set_position(tauri::PhysicalPosition::new(x, y))?;
	Ok(())
}


fn setup_app<R: tauri::Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn Error>> {

	if let Some(reason) = is_authorized()
	{
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
