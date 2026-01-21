
use std::{collections::HashMap, process::Command};
use anyhow::{Context, Result};

use crate::{
	config::CONFIG,
	execution::{GameExecution, Overlay},
	library::GameLaunchData
};

fn command_clone(self_v: &Command) -> Command {
	let mut cmd = Command::new(self_v.get_program());
	cmd.args(self_v.get_args());
	for (key, value) in self_v.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))) {
		cmd.env(key, value);
	}
	if self_v.get_current_dir().is_some() {
		cmd.current_dir(self_v.get_current_dir().unwrap());
	}
	cmd
}

impl GameExecution {
	pub fn build_command(
		name: &str,
		data: &GameLaunchData,
	) -> Result<Command> {
		let overlay = Some(Overlay{
			src: data.overlays.clone(),
			dst: format!("{}/{}", CONFIG.user_save_dir, name)
		});

		let mut cmd = Self::junest_cmd(
			data.environs.clone().unwrap_or(HashMap::new()),
			&overlay,
		);

        if data.before.is_some() {
            cmd.args(data.before.clone().unwrap());
        }

		if data.proton.is_some() {
			let proton = data.proton.as_ref().unwrap();
			let prefix = format!("{}/{proton}", CONFIG.user_save_dir);

			cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone())
			.env("WINEPREFIX", prefix)
			.env("PROTONPATH", format!("{}/{}", CONFIG.protons_dir, proton));
		} else {
			cmd.env("UMU_NO_PROTON", "1");
		}

		cmd.env("UMU_RUNTIME_UPDATE", "0");

		if !data.noruntime.unwrap_or(false) {
			cmd.arg(CONFIG.umu_run.clone());
		}

		if data.prestart.is_some() {
			let mut prestart_cmd = command_clone(&cmd);
			prestart_cmd.args(data.prestart.as_ref().unwrap())
				.spawn()
				.with_context(|| format!("Failed to run prestart for {}", name))?
				.wait()
				.with_context(|| format!("Failed to wait for prestart to finish for {}", name))?;
		}

		cmd.args(&data.start)
			.current_dir(format!("/tmp/{}/stdgames/work", CONFIG.username));

		Ok(cmd)
	}
}
