
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use zip_extensions::zip_extract;
use std::{path::Path};
use std::fs::{self, copy};
use anyhow::Result;
use std::process::Command;
use std::os::unix::fs::PermissionsExt;
use std::env;

use crate::{
	config::CONFIG,
	execution::GameExecution,
	utils::{copy_directory, is_authorized, is_mounted},
};

impl GameExecution {
	pub fn setup(handle: impl Fn(f32)) -> Result<()> {
		if !is_authorized() {
			Err(anyhow::anyhow!(
				"Unauthorized: Please ensure you have the necessary permissions."
			))?;
		}

        for directory in [
			&CONFIG.junest_home_dir,
			&CONFIG.temp_junest_home_dir,
		] {
			fs::create_dir_all(directory)?;
		}

        if  fs::read_dir(&CONFIG.games_dir).is_err() {
            Command::new("fusermount").args(vec!["-u", &CONFIG.games_dir]).spawn()?;
        }

        if !is_mounted("std@82.67.99.87:/shared") {
            let home = env::var("HOME").expect("HOME not set");
            let mut dest = PathBuf::from(home);
            dest.push(".ssh/stdgame");

        // Ensure .ssh directory exists
            let ssh_dir = dest.parent().unwrap();
            fs::create_dir_all(ssh_dir)?;

            // Copy the file
            fs::copy(&format!("{}/ssh_key", CONFIG.resources_dir), &dest)?;

            // Set permissions to 600 (rw-------)
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&dest, permissions)?; 

            Command::new(format!("{}/sshfs", CONFIG.resources_dir)).args(vec![
                "-p", "44424",
                "-o", &format!("ssh_command=ssh -i {}", dest.to_str().unwrap()),
                "std@82.67.99.87:/shared",
                &CONFIG.games_dir,
                "-o", "StrictHostKeyChecking=no,UserKnownHostsFile=/dev/null,compression=no,cache=yes,kernel_cache,max_read=131072,max_write=131072,Ciphers=aes128-gcm@openssh.com,auto_cache,attr_timeout=600,entry_timeout=600,negative_timeout=120,ServerAliveInterval=15,reconnect"
            ]).spawn()?;
        }

		copy_directory(
			Path::new(&CONFIG.junest_home_dir),
			Path::new(&CONFIG.temp_junest_home_dir),
			|data| {
				handle(data.files_copied as f32 / data.num_files as f32 * 75.0);
			},
		)?;

		if !Path::new(&CONFIG.temp_umu_dir).exists() {
			let zip_file = format!("{}/{}", CONFIG.temp_dir, "umu.zip");
			copy(CONFIG.archive_file.clone(), zip_file.clone())?;

			handle(83.0);

			zip_extract(&PathBuf::from(zip_file), &PathBuf::from(CONFIG.temp_umu_dir.clone()))?;
		}

		handle(100.0);

		sleep(Duration::from_secs_f32(0.5));

		Ok(())
	}
}
