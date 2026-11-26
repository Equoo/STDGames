//! `stdgames`: debug CLI for the `STDGames` launch runtime.

mod defaults;
mod explain;
mod loading;
mod registry;
mod run;
mod saves;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::run::Program;

const GAME_HELP: &str = "Game id: looks up games/<id>.toml, then examples/games/<id>.toml";
const MODE_HELP: &str = "Mode id: must be declared and enabled in the game's config";
const PROFILE_HELP: &str = "Save profile for this launch (default: the game's current one)";

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
        #[arg(long, help = PROFILE_HELP)]
        profile: Option<String>,
    },
    /// Resolve the plan and launch it.
    Run {
        #[arg(long, help = GAME_HELP)]
        game: String,
        #[arg(long, help = MODE_HELP)]
        mode: String,
        #[arg(long, help = PROFILE_HELP)]
        profile: Option<String>,
        /// Open a shell in the wrapped environment instead of the game.
        #[arg(long)]
        shell: bool,
    },
    /// Manage save profiles.
    Saves {
        #[command(subcommand)]
        command: saves::SavesCommand,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .without_time()
        .init();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Explain {
            game,
            mode,
            profile,
        } => explain::run(&game, &mode, profile).map(|()| ExitCode::SUCCESS),
        Command::Run {
            game,
            mode,
            profile,
            shell,
        } => {
            let program = if shell { Program::Shell } else { Program::Game };
            run::run(&game, &mode, profile, program).map(ExitCode::from)
        }
        Command::Saves { command } => saves::run(command).map(|()| ExitCode::SUCCESS),
    };

    result.unwrap_or_else(|e| {
        tracing::error!("{e:#}");
        ExitCode::FAILURE
    })
}
