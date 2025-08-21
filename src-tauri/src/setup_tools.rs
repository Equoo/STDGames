use std::error::Error;
use std::fs;
use std::path::Path;
use tar::Archive;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::config::Config;
use crate::copy_directory::{CopyData, copy_directory};
use crate::errors::AppError;

pub fn setup_tools(app: AppHandle, config: State<'_, Config>) -> Result<(), Box<dyn Error>> {
    println!("Installing tools ...");

    let splashscreen_window = app
        .get_webview_window("splashscreen")
        .ok_or(AppError::new("didn't find the 'splashscreen' webview"))?;
    let launcher_window = app
        .get_webview_window("main")
        .ok_or(AppError::new("didn't find the 'main' webview"))?;

    splashscreen_window.emit("progressbar_update", 0)?;
    splashscreen_window.show()?;

    for directory in [
        config.junest_home_dir.clone(),
        config.temp_junest_home_dir.clone(),
    ] {
        fs::create_dir_all(directory)?;
    }

    let handle = |copy_data: CopyData| {
        let _ = splashscreen_window.emit(
            "progressbar_update",
            copy_data.files_copied * 50 / copy_data.num_files,
        );
    };
    copy_directory(
        Path::new(&config.junest_home_dir),
        Path::new(&config.temp_junest_home_dir),
        handle,
    )?;

    // splashscreen_window.emit("progressbar_update", 60)?;

    // untested code
    // Archive::new(File::open(config.resource_umu_archive_file.clone())?).unpack(config.temp_dir.clone())?;

    splashscreen_window.emit("progressbar_update", 100)?;
    println!("Finished installing tools.");

    splashscreen_window.close()?;
    launcher_window.show()?;

    // TODO: the interface is not displayed if there is no loading,
    // : maybe readd the client_loaded switch

    Ok(())
}
