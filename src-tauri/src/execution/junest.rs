
use std::{collections::HashMap, path::PathBuf, process::Command};
use uzers::get_current_uid;

use crate::{
	config::CONFIG,
	execution::{GameExecution, Overlay}
};

impl GameExecution {
	pub fn junest_cmd(
		mut environ: HashMap<String, String>,
		overlay: &Option<Overlay>,
	) -> Command {
		environ.insert("JUNEST_HOME".to_string(), CONFIG.temp_junest_home_dir);

		let cmd = Command::new(CONFIG.junest_bin);
		
		let work_dir = if overlay.is_some() {
			PathBuf::from(format!("/tmp/{}/stdgames/work", CONFIG.username))
		} else {
			PathBuf::from(&CONFIG.user_home)
		};
		
		cmd.envs(environ)
			.current_dir(work_dir);

		let overlay_str = if let Some(o) = overlay {
			// Ensure the overlay directories exist
			let user = CONFIG.username;
			[
				&format!("/tmp/{user}"),
				&format!("/tmp/{user}/stdgames/rw"),
				&format!("/tmp/{user}/stdgames/overlay_work"),
				&format!("/tmp/{user}/stdgames/work"),
			].iter()
			.for_each(|folder| {
				std::fs::create_dir_all(folder).unwrap_or_else(|e| {
					eprintln!("Failed to create directory {}: {}", folder, e);
				});
			});

			cmd.arg(CONFIG.overlay);
			cmd.arg(format!("/tmp/{user}/stdgames/rw"));
			cmd.arg(o.dst);

			format!(
				"--overlay-src {}
				--overlay /tmp/{user}/stdgames/rw /tmp/{user}/stdgames/overlay_work /tmp/{user}/stdgames/work",
				o.src.join(" ")
			)
		} else {
			String::new()
		};

		let uid = get_current_uid();
		cmd.arg("-b")
			.arg(format!(
				"--bind /sgoinfre /sgoinfre
				--bind /goinfre /goinfre
				--bind /media /media
				--bind /tmp/{} /tmp
				--bind {} /usr
				--bind /tmp/.X11-unix /tmp/.X11-unix
				--bind /run/user/{uid}/pulse/native /run/pulse/native
				{overlay_str}
			", CONFIG.username, CONFIG.junest_bind
			));
		cmd
	}
}