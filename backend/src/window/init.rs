use std::env;
use std::{error::Error, sync::Mutex};

use anyhow::{Result, anyhow};
use clap::error::Result;
use tauri::{App, Builder, Manager, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::{
    library::Game,
    utils::is_authorized,
    window::{commands, splashscreen::setup_tools_wrapper},
};

pub struct AppState {
    pub games: Mutex<Vec<Game>>,
    pub setup_finished: Mutex<bool>,
    pub exec: Mutex<GameExecution>,
}

pub fn init_window(library: Vec<Game>, game_exec: GameExecution) {
    // init_env_for_codecs().expect("Failed to set environment variables for codecs");

    let app = Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // .manage(AppState {
        //     games: Mutex::new(library),
        //     setup_finished: Mutex::new(false),
        //     exec: Mutex::new(game_exec),
        // })
        .invoke_handler(tauri::generate_handler![
            commands::add_launcher_to_desktop,
            commands::get_game_library,
            commands::launch_game,
            commands::get_running_game
        ])
        .setup(setup_app)
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Hide instead of closing
                    window.hide().unwrap();
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("Error while launching Tauri");

    app.run(|_app, event| {
        match event {
            RunEvent::ExitRequested { api, .. } => {
                // Prevent exit on window close, only exit on tray menu quit
                api.prevent_exit();
            }
            _ => {}
        }
    });
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

    setup_tray(app)?;

    let window = app
        .get_webview_window("splashscreen")
        .ok_or(anyhow!("didn't find the 'splashscreen' webview"))?;

    if let Err(e) = center_window(&window) {
        eprintln!("failed to center window: {}", e);
    }

    tauri::async_runtime::spawn(setup_tools_wrapper(app.handle().clone()));

    Ok(())
}

fn setup_tray(app: &mut app) -> Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<String>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<String>)?;
    let menu = Menu::with_items(app, &[&toggle, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "toggle" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = if window.is_visible().unwrap_or(false) {
                            window.hide()
                        } else {
                            window.show().and_then(|_| window.set_focus())
                        };
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_window("main") {
                    let _ = if window.is_visible().unwrap_or(false) {
                        window.hide()
                    } else {
                        window.show().and_then(|_| window.set_focus())
                    };
                }
            }
        })
        .build(app)?;

    #[cfg(target_os = "windows")]
    app.get_window("main").set_skip_taskbar(false)?; // Show in taskbar on Windows

    #[cfg(target_os = "linux")]
    app.get_window("main").set_skip_taskbar(false)?; // Show in taskbar on Linux

    Ok(())
}


fn init_env_for_codecs() -> Result<()> {
    Ok(unsafe {
        env::set_var(
            "LD_LIBRARY_PATH",
            format!(
                "{}:/sgoinfre/stdgames/.resources/launcher_libs",
                env::var("LD_LIBRARY_PATH").unwrap_or_default()
            ),
        );
        env::set_var(
            "GST_PLUGIN_PATH",
            format!(
                "{}:/sgoinfre/stdgames/.resources/launcher_libs/gstreamer-1.0",
                env::var("GST_PLUGIN_PATH").unwrap_or_default()
            ),
        );
        env::set_var("GST_REGISTRY_UPDATE", "yes");
    })
}


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
