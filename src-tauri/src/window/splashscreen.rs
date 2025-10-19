use anyhow::{Result, anyhow};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::execution::GameExecution;
use crate::library;
use crate::window::init::AppState;

pub async fn setup_tools_wrapper(app: AppHandle) {
    if let Err(e) = setup_tools(app.clone()).await {
        app.dialog()
            .message(format!("setup_tools failed: {}", e))
            .title("Error")
            .kind(MessageDialogKind::Error)
            .show(|_| std::process::exit(1));
    }
}

pub async fn setup_tools(app: AppHandle) -> Result<()> {
    println!("Installing tools ...");

    let splashscreen_window = app
        .get_webview_window("splashscreen")
        .ok_or(anyhow!("didn't find the 'splashscreen' webview"))?;
    let launcher_window = app
        .get_webview_window("main")
        .ok_or(anyhow!("didn't find the 'main' webview"))?;

    splashscreen_window.emit("progressbar_update", 0)?;
    splashscreen_window.show()?;

    let mut app_state = app.state::<AppState>();
    let mut games: Vec<library::Game>;
    {
        let games_mutex = app_state.games.lock().map_err(|e| anyhow!(e.to_string()));
        games = games_mutex
            .as_ref()
            .clone()
            .map_err(|e| anyhow!(e.to_string()))?
            .to_vec();
    }
    println!("API DATA RESULT: {:?}", library::load_api_data(&mut games).await);
    {
        let mut games_mutex = app_state.games.lock().map_err(|e| anyhow!(e.to_string()))?;
        *games_mutex = games;
    }
    {
        *app_state
            .setup_finished
            .lock()
            .map_err(|e| anyhow!(e.to_string()))? = true;
    }

    let handle = |progress: f32| {
        let _ = splashscreen_window.emit("progressbar_update", progress as u8);
    };

    GameExecution::setup(handle)?;

    splashscreen_window.emit("progressbar_update", 100)?;
    println!("Finished installing tools.");

    splashscreen_window.close()?;
    launcher_window.show()?;

    Ok(())
}
