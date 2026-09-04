mod defaults;
mod explain;
mod loading;
mod registry;
mod run;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "stdgames", version, about = "STDGames launch runtime — debug CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Resolve and print the full plan for a game/mode pair — final
    /// command, environment, container bindings, and layer order — without
    /// launching anything.
    Explain {
        /// Game id: looks up games/<id>.toml, then examples/games/<id>.toml
        #[arg(long)]
        game: String,
        /// Mode id: must be declared and enabled in the game's config
        #[arg(long)]
        mode: String,
    },
    /// Resolve the plan and actually launch it: runs every layer's real
    /// `prepare`/`patch`/`wrap`, then spawns the final command and waits
    /// for it to exit.
    Run {
        /// Game id: looks up games/<id>.toml, then examples/games/<id>.toml
        #[arg(long)]
        game: String,
        /// Mode id: must be declared and enabled in the game's config
        #[arg(long)]
        mode: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Explain { game, mode } => {
            if let Err(e) = explain::run(&game, &mode) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Command::Run { game, mode } => match run::run(&game, &mode) {
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
    }
}
