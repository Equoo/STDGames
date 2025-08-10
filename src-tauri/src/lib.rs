mod check_authorized;
mod errors;

use check_authorized::is_authorized;
use errors::AppError;

use std::error::Error;
use tauri::{Manager, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};


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


use std::thread;
use std::time::Duration;
// setup error return in this function
async fn setup_tools(app: tauri::AppHandle)
{
	loop {
		app.emit("progressbar_update", 42);
		thread::sleep(Duration::new(3, 0));
		app.emit("progressbar_update", 84);
		thread::sleep(Duration::new(3, 0));
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
	tauri::async_runtime::spawn(setup_tools(app.handle().clone()));

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
