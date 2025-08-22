
use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::library::Game;
use crate::execution::GameExecution;

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
	config: Option<String>,

	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

pub fn init_cli(
	cli: &Cli,
	library: &Vec<Game>,
	game_exec: &mut GameExecution,
) -> Result<()> {
	GameExecution::setup();

	match cli.command {
		Some(Commands::Run { game }) => {
			let launch_data = library.iter()
				.find(|g| g.slug == game) 
				.expect("Game not found in library").launch
				.clone();
			
			let proc = game_exec.run(&game, &launch_data)
				.expect("Failed to run game")
				.spawn()
				.expect("Failed to spawn game process");
			game_exec.running = Some(GameProcess {
				process: proc,
				game: game
			});
		}
		Some(Commands::Bash { game }) => {
			let mut launch_data = library.iter()
				.find(|g| g.slug == game) 
				.expect("Game not found in library").launch
				.clone();
			launch_data.start = ["/bin/bash".to_string()].to_vec();
			
			let err = game_exec.run(&game, &launch_data)
				.expect("Failed to run game")
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
			let err = GameExecution::junest_run(["bash".to_string()].to_vec(), HashMap::new(), &None)
				.exec();
			println!("Error running Junest: {}", err);
		}
		None => {}
	}

	Ok(())
}