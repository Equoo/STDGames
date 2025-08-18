use std::error::Error;
use tauri::{Manager, Emitter, AppHandle};
use fs_extra::{TransitProcess, dir::TransitProcessResult, dir::CopyOptions};
use tar::Archive;
use std::fs::{self, File};

use crate::errors::AppError;


pub fn setup_tools(app: AppHandle) -> Result<(), Box<dyn Error>> {
	println!("Installing tools ...");

	let splashscreen_window = app.get_webview_window("splashscreen")
		.ok_or(AppError::new("didn't find the 'splashscreen' webview"))?;
	let launcher_window = app.get_webview_window("main")
		.ok_or(AppError::new("didn't find the 'main' webview"))?;

	splashscreen_window.emit("progressbar_update", 0)?;
	splashscreen_window.show()?;

	// for directory in [config.resources_junest_home_dir.clone(), config.temp_junest_home_dir.clone()] {
	//     fs::create_dir_all(directory)?;
	// }


	let option = CopyOptions::new().skip_exist(true);
	let handle = |process_info: TransitProcess| {
		let _ = splashscreen_window.emit("progressbar_update", process_info.copied_bytes * 100 / process_info.total_bytes);
		TransitProcessResult::ContinueOrAbort
	};
	// this copy as a lot of excluded folders, just delete the folders on the src folder
	// fs_extra::copy_items_with_progress(&vec![config.resources_junest_home_dir.clone()], config.temp_dir.clone(), &option, handle)?;
	fs_extra::copy_items_with_progress(&vec!["/home/sky/game/cracked/art of rally"], "./dst", &option, handle)?;


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
