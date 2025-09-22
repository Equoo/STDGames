
use anyhow::{Context, Result};

use crate::{
	config::CONFIG,
	execution::{GameExecution, GameProcess},
	library::Game
};

fn get_game<'a>(library: &'a Vec<Game>, name: &'a String) -> Result<&'a Game> {
	library.iter()
		.find(|g| &g.slug == name)
		.ok_or_else(|| anyhow::anyhow!("Game '{}' not found in library", name))
}

impl GameExecution {
	pub fn start(
		&mut self,
		name: &str,
	) -> Result<()> {
		// add possibility to launch via steam

		let vars = CONFIG.clone().build_vars();
		let mut launch_data = get_game(&self.library, &name.to_string())?.launch.clone();
			launch_data.replace_vars(&vars);	
		let child = GameExecution::build_command(name, &launch_data)?
			.spawn()?;
		self.running = Some(GameProcess{name: name.to_string(), process: child});

		Ok(())
	}

	pub fn is_running(&mut self) -> bool {
		self.running.is_some() || self.running.as_mut().unwrap().process.try_wait().is_ok()
	}

	pub fn stop(&mut self) -> Result<()> {
		if self.is_running() {
			self.running.as_mut().unwrap().process.kill().context("Failed to kill the running game process")?;
			self.running = None;
		}
		Ok(())
	}

	pub fn kill(&mut self) -> Result<()> {
		if self.is_running() {
			self.running.as_mut().unwrap().process.kill().context("Failed to kill the running game process")?;
			self.running = None;
		}
		Ok(())
	}
}