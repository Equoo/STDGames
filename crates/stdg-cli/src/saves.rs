//! `stdgames saves ...`: manage save profiles and the Ludusavi manifest.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as _;
use clap::Subcommand;
use stdg_saves::{GameSaves, MANIFEST_URL, ManifestIndex, ProfileName};

use crate::registry::{manifest_index_path, saves_store};

#[derive(Debug, Subcommand)]
pub(crate) enum SavesCommand {
    /// Download the Ludusavi manifest and rebuild the save-location index.
    UpdateManifest {
        /// Use this manifest.yaml instead of downloading it.
        #[arg(long)]
        from: Option<PathBuf>,
    },
    /// List a game's profiles; `*` marks the current one.
    List {
        #[arg(long)]
        game: String,
    },
    /// Create a profile, empty or as a copy of another.
    New {
        #[arg(long)]
        game: String,
        #[arg(long)]
        name: String,
        /// Profile to copy.
        #[arg(long)]
        from: Option<String>,
    },
    /// Use this profile for the next launches.
    Use {
        #[arg(long)]
        game: String,
        #[arg(long)]
        name: String,
    },
    /// Write a profile to a .tar.gz archive.
    Export {
        #[arg(long)]
        game: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Create a profile from an archive made by `export`.
    Import {
        #[arg(long)]
        game: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        file: PathBuf,
    },
}

#[expect(clippy::print_stdout, reason = "user-facing CLI output")]
pub(crate) fn run(command: SavesCommand) -> anyhow::Result<()> {
    match command {
        SavesCommand::UpdateManifest { from } => {
            let count = update_manifest(from.as_deref())?;
            println!("indexed {count} games");
        }
        SavesCommand::List { game } => {
            let saves = game_saves(&game)?;
            let current = saves.current()?;
            for profile in saves.list()? {
                let mark = if profile == current { "*" } else { " " };
                println!("{mark} {profile}");
            }
        }
        SavesCommand::New { game, name, from } => {
            let from = from.as_deref().map(str::parse::<ProfileName>).transpose()?;
            game_saves(&game)?.create(&name.parse()?, from.as_ref())?;
        }
        SavesCommand::Use { game, name } => game_saves(&game)?.set_current(&name.parse()?)?,
        SavesCommand::Export { game, name, out } => {
            game_saves(&game)?.export(&name.parse()?, &out)?;
        }
        SavesCommand::Import { game, name, file } => {
            game_saves(&game)?.import(&name.parse()?, &file)?;
        }
    }
    Ok(())
}

fn game_saves(game: &str) -> anyhow::Result<GameSaves> {
    Ok(GameSaves::new(&saves_store()?, game)?)
}

/// Builds the index from `from`, or from a freshly downloaded manifest.
fn update_manifest(from: Option<&Path>) -> anyhow::Result<usize> {
    let index_path = manifest_index_path()?;
    let downloaded = index_path.with_file_name("manifest.yaml");
    let manifest = if let Some(path) = from {
        path
    } else {
        download(MANIFEST_URL, &downloaded)?;
        &downloaded
    };
    let yaml = std::fs::read_to_string(manifest)
        .with_context(|| format!("reading {}", manifest.display()))?;
    let index = ManifestIndex::from_manifest_yaml(&yaml)?;
    index.save(&index_path)?;
    if from.is_none() {
        // Only the index is kept; failing to delete the download is harmless.
        let _ = std::fs::remove_file(&downloaded);
    }
    Ok(index.len())
}

fn download(url: &str, to: &Path) -> anyhow::Result<()> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--output",
        ])
        .arg(to)
        .arg(url)
        .status()
        .context("running curl")?;
    anyhow::ensure!(status.success(), "downloading {url} failed: curl {status}");
    Ok(())
}
