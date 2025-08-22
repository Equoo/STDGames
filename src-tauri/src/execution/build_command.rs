
use std::{collections::HashMap, process::Command};
use anyhow::{Context, Result};

use crate::{
	config::CONFIG,
	execution::{GameProcess, Overlay},
	library::GameLaunchData
};

impl GameProcess {
	fn build_command(
		name: &str,
		data: &GameLaunchData,
		env_vars: HashMap<String, String>,
	) -> Result<Command> {
		let overlay = Some(Overlay{
			src: data.overlays,
			dst: format!("{}/{}", CONFIG.user_save_dir, name)
		});
		
		let cmd = Self::junest_cmd(
			env_vars,
			&overlay,
		);
		cmd.envs(env_vars);

		Ok(cmd)
	}
}