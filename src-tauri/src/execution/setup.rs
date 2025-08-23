
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use zip_extensions::zip_extract;
use std::{path::Path};
use std::fs::{self, copy};
use anyhow::Result;

use crate::{
	config::CONFIG,
	execution::GameExecution,
	utils::{copy_directory, is_authorized},
};

impl GameExecution {
	pub fn setup(handle: impl Fn(f32)) -> Result<()> {
		if !is_authorized() {
			Err(anyhow::anyhow!(
				"Unauthorized: Please ensure you have the necessary permissions."
			))?;
		}

		for directory in [
			CONFIG.junest_home_dir.clone(),
			CONFIG.temp_junest_home_dir.clone(),
		] {
			fs::create_dir_all(directory)?;
		}
		
		copy_directory(
			Path::new(&CONFIG.junest_home_dir),
			Path::new(&CONFIG.temp_junest_home_dir),
			|data| {
				handle(data.files_copied as f32 / data.num_files as f32 * 50.0);
			},
		)?;

		if !Path::new(&CONFIG.umu_run).exists() {
			let zip_file = format!("{}/{}", CONFIG.temp_dir, "umu.zip");
			copy(CONFIG.archive_file.clone(), zip_file.clone())?;

			handle(75.0);

			zip_extract(&PathBuf::from(zip_file), &PathBuf::from(CONFIG.temp_umu_dir.clone()))?;
		}

		handle(100.0);

		sleep(Duration::from_secs_f32(0.5));

		Ok(())
	}
}