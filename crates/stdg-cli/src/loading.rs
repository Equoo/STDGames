//! Loads a game config and resolves its `Plan`.

use std::path::PathBuf;

use anyhow::Context as _;
use stdg_core::{GameId, ModeId, PartialGameConfig, Plan};
use stdg_plan::{RunnerDefaults, build_plan};
use stdg_registry::Registry;

use crate::defaults::{load_global_defaults, load_runner_defaults};
use crate::registry::build_registry;

fn find_game_config(game_id: &GameId) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("games").join(format!("{}.toml", game_id.0)),
        PathBuf::from("examples/games").join(format!("{}.toml", game_id.0)),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub(crate) fn load_plan(game_id_str: &str, mode_id_str: &str) -> anyhow::Result<(Registry, Plan)> {
    let game_id = GameId(game_id_str.to_string());
    let mode_id = ModeId(mode_id_str.to_string());

    let config_path = find_game_config(&game_id).with_context(|| {
        format!(
            "no config file found for game `{game_id}` (looked in ./games and ./examples/games)"
        )
    })?;
    let raw = std::fs::read_to_string(&config_path)
        .with_context(|| format!("reading {}", config_path.display()))?;
    let game: PartialGameConfig =
        toml::from_str(&raw).with_context(|| format!("parsing {}", config_path.display()))?;

    let registry = build_registry();

    let global_defaults = load_global_defaults()?;
    let runner_defaults = match &game.target {
        Some(target) => load_runner_defaults(target)?,
        None => RunnerDefaults::default(),
    };

    let plan = build_plan(
        &game_id,
        &mode_id,
        &global_defaults,
        &runner_defaults,
        &game,
        &registry,
        &registry,
    )
    .with_context(|| format!("planning game `{game_id}` in mode `{mode_id}`"))?;

    Ok((registry, plan))
}
