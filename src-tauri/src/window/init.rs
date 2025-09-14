use std::{error::Error, sync::Mutex};
use std::env;

use anyhow::{anyhow, Result};
use tauri::{App, Builder, Manager, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::{
	execution::GameExecution,
	library::Game,
	utils::is_authorized,
	window::{commands, splashscreen::setup_tools_wrapper}
};

fn center_window(window: &WebviewWindow) -> Result<()> {
    let monitor = window
        .current_monitor()?
        .ok_or_else(|| anyhow!("didn't find any monitor"))?;
    let monitor_size = monitor.size();
    let window_size = window.inner_size()?;
    let x = (monitor_size.width - window_size.width) / 2;
    let y = (monitor_size.height - window_size.height) / 2;
    window.set_position(tauri::PhysicalPosition::new(x, y))?;
    Ok(())
}

fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
    if !is_authorized() {
        app.dialog()
            .message("You do not have permission to run this application.")
            .title("Access Denied")
            .kind(MessageDialogKind::Error)
            .show(|_| std::process::exit(1));
        return Ok(());
    }

    let window = app
        .get_webview_window("splashscreen")
        .ok_or(anyhow!("didn't find the 'splashscreen' webview"))?;

    if let Err(e) = center_window(&window) {
        eprintln!("failed to center window: {}", e);
    }

    tauri::async_runtime::spawn(setup_tools_wrapper(app.handle().clone()));

    Ok(())
}

fn init_env_for_codecs() -> Result<()> {
    Ok(unsafe {
        env::set_var("LD_LIBRARY_PATH", format!("{}:/sgoinfre/stdgames/.resources/launcher_libs", env::var("LD_LIBRARY_PATH").unwrap_or_default()));
        env::set_var("GST_PLUGIN_PATH", format!("{}:/sgoinfre/stdgames/.resources/launcher_libs/gstreamer-1.0", env::var("GST_PLUGIN_PATH").unwrap_or_default()));
        env::set_var("GST_REGISTRY_UPDATE", "yes");
    })
}

pub fn init_window(
    library: Vec<Game>,
    game_exec: GameExecution,
) {
    init_env_for_codecs().expect("Failed to set environment variables for codecs");
    
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(library)
		.manage(Mutex::new(game_exec))
        .invoke_handler(tauri::generate_handler![
			commands::add_launcher_to_desktop,
            commands::get_game_library,
            commands::launch_game
		])
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de Tauri");
}
