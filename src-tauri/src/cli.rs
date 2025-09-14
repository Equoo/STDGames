
use std::collections::HashMap;
use std::os::unix::process::CommandExt;

use clap::{Parser, Subcommand};
use anyhow::Result;
use std::io::{self, Write};

use crate::config::CONFIG;
use crate::library::Game;
use crate::execution::{GameExecution, GameProcess};

#[derive(Parser, Debug)]
#[command(
	name = "stdgames",
	version,
	about = "Stdgames launcher by zsonie, tdaclin and dderny.",
	author = "zsonie, tdaclin, dderny"
)]
pub struct Cli {
	/// Use a custom config file
	#[arg(short, long, value_name = "FILE")]
	pub config: Option<String>,

	#[command(subcommand)]
	pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Run a game from the stdgames repository
	Run {
		/// Game name
		game: String,
	},

	/// Run bash with the game's config
	Bash {
		/// Game name
		game: String,
	},

	/// Run a game with a custom config file
	RunConfig {
		/// Path to TOML config file
		file: String,
	},

	/// Run bash with a custom config file
	BashConfig {
		/// Path to TOML config file
		file: String,
	},

	/// Enter the Junest environment
	Junest,
}

fn get_game<'a>(library: &'a Vec<Game>, name: &'a String) -> Result<&'a Game> {
	library.iter()
		.find(|g| &g.slug == name)
		.ok_or_else(|| anyhow::anyhow!("Game '{}' not found in library", name))
}

pub fn init_cli(
	cli: &Cli,
	library: &Vec<Game>,
	game_exec: &mut GameExecution,
) -> Result<()> {
	GameExecution::setup(|progress: f32| {
		let progress = progress / 100.0;
		let filled = (progress * 50.0) as usize;
		let empty = 50 - filled;
		
		print!("\rSetup progress: [{}{}] {:.1}%", 
			"█".repeat(filled),
			"░".repeat(empty),
			progress * 100.0);
		
		io::stdout().flush().unwrap();
	})?;

	let vars = CONFIG.clone().build_vars();

	match &cli.command {
		Some(Commands::Run { game }) => {
			let mut launch_data = get_game(library, &game)?.launch.clone();
			launch_data.replace_vars(&vars);
			
			let _ = GameExecution::build_command(&game, &launch_data)?
				.spawn()?;
		}
		Some(Commands::Bash { game }) => {
			let mut launch_data = get_game(library, &game)?.launch.clone();
			launch_data.start = ["/bin/bash".to_string()].to_vec();
			launch_data.noruntime = Some(true);
			launch_data.replace_vars(&vars);
			
			let err = GameExecution::build_command(&game, &launch_data)?
				.exec();
			println!("Error running bash: {}", err);
		}
		Some(Commands::RunConfig { file }) => {
			println!("Running game with config file: {}", file);
		}
		Some(Commands::BashConfig { file }) => {
			println!("Running bash with config file: {}", file);
		}
		Some(Commands::Junest) => {
			let err = GameExecution::junest_cmd(HashMap::new(), &None)
				.arg("/bin/bash")
				.exec();
			println!("Error running Junest: {}", err);
		}
		None => {}
	}

	Ok(())
}