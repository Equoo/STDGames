//! `stdgames`: debug CLI for the `STDGames` launch runtime.

mod defaults;
mod explain;
mod loading;
mod registry;
mod run;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::run::Program;

const GAME_HELP: &str = "Game id: looks up games/<id>.toml, then examples/games/<id>.toml";
const MODE_HELP: &str = "Mode id: must be declared and enabled in the game's config";

#[derive(Debug, Parser)]
#[command(
    name = "stdgames",
    version,
    about = "STDGames launch runtime — debug CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the resolved plan without launching anything.
    Explain {
        #[arg(long, help = GAME_HELP)]
        game: String,
        #[arg(long, help = MODE_HELP)]
        mode: String,
    },
    /// Resolve the plan and launch it.
    Run {
        #[arg(long, help = GAME_HELP)]
        game: String,
        #[arg(long, help = MODE_HELP)]
        mode: String,
        /// Open a shell in the wrapped environment instead of the game.
        #[arg(long)]
        shell: bool,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .without_time()
        .init();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Explain { game, mode } => explain::run(&game, &mode).map(|()| ExitCode::SUCCESS),
        Command::Run { game, mode, shell } => {
            let program = if shell { Program::Shell } else { Program::Game };
            run::run(&game, &mode, program).map(ExitCode::from)
        }
    };

    result.unwrap_or_else(|e| {
        tracing::error!("{e:#}");
        ExitCode::FAILURE
    })
}
