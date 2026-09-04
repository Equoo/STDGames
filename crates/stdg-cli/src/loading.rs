//! Shared game-config loading and plan resolution, used by both `explain`
//! and `run`: parse `<game>.toml`, load the global/runner defaults tiers,
//! build the registry, and resolve the `Plan`.

use std::path::PathBuf;

use stdg_core::{GameId, ModeId, PartialGameConfig, Plan};
use stdg_plan::{build_plan, RunnerDefaults};
use stdg_registry::Registry;

use crate::defaults::{load_global_defaults, load_runner_defaults};
use crate::registry::build_registry;

pub fn find_game_config(game_id: &GameId) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("games").join(format!("{}.toml", game_id.0)),
        PathBuf::from("examples/games").join(format!("{}.toml", game_id.0)),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

pub fn load_plan(game_id_str: &str, mode_id_str: &str) -> Result<(Registry, Plan), String> {
    let game_id = GameId(game_id_str.to_string());
    let mode_id = ModeId(mode_id_str.to_string());

    let config_path = find_game_config(&game_id)
        .ok_or_else(|| format!("no config file found for game `{}` (looked in ./games and ./examples/games)", game_id.0))?;
    let raw = std::fs::read_to_string(&config_path).map_err(|e| format!("reading {}: {e}", config_path.display()))?;
    let game: PartialGameConfig = toml::from_str(&raw).map_err(|e| format!("parsing {}: {e}", config_path.display()))?;

    let registry = build_registry();

    // The runner isn't resolved yet at this point (that happens inside
    // build_plan), but which defaults file to load only needs the target
    // kind the game itself declares.
    let global_defaults = load_global_defaults();
    let runner_defaults = match &game.target {
        Some(target) => load_runner_defaults(target),
        None => RunnerDefaults::default(),
    };

    let plan = build_plan(&game_id, &mode_id, &global_defaults, &runner_defaults, &game, &registry, &registry).map_err(|e| e.to_string())?;

    Ok((registry, plan))
}
