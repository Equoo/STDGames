use std::{collections::HashMap, error::Error, process::Command};

use crate::config::{CONFIG, Config};

pub struct GameProcess {
	pub process: std::process::Child,
	pub game: String,
}

pub struct GameExecution {
	running: Option<GameProcess>,
}

impl GameExecution {
	pub fn new() -> Self {
		Self {
			running: None,
		}
	}

	pub fn junest_run(environ: HashMap<String, String>, overlays: Option<Vec<String>>) -> Result<(), Box<dyn Error>> {
		let conf: Config = CONFIG.clone();
		let user = conf.username;
		let uid: i64 = unsafe { libc::getuid() };

		environ.insert("JUNEST_HOME", conf.temp_junest_home_dir);

		let folders = [
			&format!("/tmp/{user}"),
			&format!("/tmp/{user}/stdgames/rw"),
			&format!("/tmp/{user}/stdgames/overlay_work"),
		];

		let mut overlays_str = overlays
			.map(|o| o.join(" "))
			.unwrap_or("");
		if !overlays_str.is_empty() {
			overlays_str = format!("
				--overlay-src {overlays_str}
				--overlay /tmp/stdgames/rw /tmp/stdgames/overlay_work /tmp/stdgames/{gamename}");
		}

		Command::new(conf.junest_bin)
			.envs(environ)
			.arg("-b")
			.arg(format!("
				--bind /sgoinfre /sgoinfre
				--bind /goinfre /goinfre
				--bind /media /media
				--bind /tmp/{user} /tmp
				--bind /tmp/.X11-unix /tmp/.X11-unix
				--bind /run/user/{uid}/pulse/native /run/pulse/native
				{overlays_str}
			"));

		Ok(())
	}
}