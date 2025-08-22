
fn center_window(window: &WebviewWindow) -> Result<(), Box<dyn Error>> {
	let monitor = window.current_monitor()?
		.ok_or(AppError::new("didn't find any monitor"))?;
	let monitor_size = monitor.size();
	let window_size = window.inner_size()?;
	let x = (monitor_size.width - window_size.width) / 2;
	let y = (monitor_size.height - window_size.height) / 2;
	window.set_position(tauri::PhysicalPosition::new(x, y))?;
	Ok(())
}

fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
	if let Some(reason) = is_authorized() {
		app.dialog()
			.message(reason)
			.title("Access Denied")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
		return Ok(());
	}

	let window = app.get_webview_window("splashscreen")
		.ok_or(AppError::new("didn't find the 'splashscreen' webview"))?;

	if let Err(e) = center_window(&window) {
		eprintln!("failed to center window: {}", e);
	}

	// maybe use spawn_blocking instead, if there is lag maybe it's because of that
	tauri::async_runtime::spawn(setup_tools_wrapper(app.handle().clone()));

	Ok(())
}

pub fn init_window(config: &Config, library: &library::Library, game_exec: &mut execution::GameExecution) {
	Builder::default()
		.plugin(tauri_plugin_opener::init())
		.plugin(tauri_plugin_dialog::init())
		.manage(config)
		.invoke_handler(tauri::generate_handler![
			commands::add_launcher_to_desktop
		])
		.setup(setup_app)
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}