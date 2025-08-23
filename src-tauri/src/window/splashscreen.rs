
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use anyhow::{Result, anyhow};

use crate::execution::GameExecution;


pub async fn setup_tools_wrapper(app: AppHandle) {
	if let Err(e) = setup_tools(app.clone()) {
		app.dialog()
			.message(format!("setup_tools failed: {}", e))
			.title("Error")
			.kind(MessageDialogKind::Error)
			.show(|_| std::process::exit(1));
	}
}

pub fn setup_tools(app: AppHandle) -> Result<()> {
    println!("Installing tools ...");

    let splashscreen_window = app
        .get_webview_window("splashscreen")
        .ok_or(anyhow!("didn't find the 'splashscreen' webview"))?;
    let launcher_window = app
        .get_webview_window("main")
        .ok_or(anyhow!("didn't find the 'main' webview"))?;

    splashscreen_window.emit("progressbar_update", 0)?;
    splashscreen_window.show()?;

    let handle = |progress: f32| {
        let _ = splashscreen_window.emit(
            "progressbar_update",
            progress as u8,
        );
    };
    
    GameExecution::setup(handle)?;
    
    splashscreen_window.emit("progressbar_update", 100)?;
    println!("Finished installing tools.");

    splashscreen_window.close()?;
    launcher_window.show()?;

    // TODO: the interface is not displayed if there is no loading,
    // : maybe readd the client_loaded switch

    Ok(())
}
