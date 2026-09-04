use std::collections::BTreeMap;

use serde::Deserialize;

use stdg_core::{
    ConfigError, CoreError, GameId, LayerRef, ModeId, PartialGameConfig, PartialModeConfig, ResolvedConfig,
};

/// Base of the cascade: values with no more-specific override anywhere.
/// `baseline_layers` is the lowest-precedence tier of the per-slot layer
/// merge (see [`crate::validate::assign_slots`]): layers that make sense
/// regardless of target, e.g. cgroup supervision on every launch.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalDefaults {
    #[serde(default)]
    pub command_prefix: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub baseline_layers: Vec<LayerRef>,
}

/// Second rung of the cascade: defaults tied to the runner a game will use
/// (e.g. every Windows game gets `gamemoderun` unless overridden, and every
/// Windows game's "desktop" mode defaults to the same Compat layer). `modes`
/// is keyed by the same `ModeId`s a game itself uses; a game's own
/// `PartialModeConfig::layers` overrides these per slot rather than having
/// to restate them.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RunnerDefaults {
    #[serde(default)]
    pub command_prefix: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub modes: BTreeMap<ModeId, ModeLayerDefaults>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModeLayerDefaults {
    #[serde(default)]
    pub layers: Vec<LayerRef>,
}

impl RunnerDefaults {
    pub fn layers_for(&self, mode_id: &ModeId) -> Vec<LayerRef> {
        self.modes.get(mode_id).map(|m| m.layers.clone()).unwrap_or_default()
    }
}

/// Merges `global -> runner_defaults -> game -> mode` into a `ResolvedConfig`
/// with no `Option` left. `target`/`root`/`executable` only ever come from
/// the game layer: they identify the game, so cascading them from a
/// lower-specificity layer would be meaningless. `args` and `env` are
/// additive across layers (mode contributions extend rather than replace the
/// game's); `command_prefix` is a full replacement, since a mode's prefix
/// list (e.g. under gamescope) usually isn't a partial edit of the runner's.
pub fn resolve_config(
    game_id: &GameId,
    global: &GlobalDefaults,
    runner_defaults: &RunnerDefaults,
    game: &PartialGameConfig,
    mode: &PartialModeConfig,
) -> Result<ResolvedConfig, CoreError> {
    let target = game.target.clone().ok_or_else(|| {
        ConfigError::MissingField {
            game_id: game_id.0.clone(),
            field: "target",
        }
    })?;
    let root = game.root.clone().ok_or_else(|| ConfigError::MissingField {
        game_id: game_id.0.clone(),
        field: "root",
    })?;
    let executable = game.executable.clone().ok_or_else(|| ConfigError::MissingField {
        game_id: game_id.0.clone(),
        field: "executable",
    })?;

    let mut args = game.args.clone().unwrap_or_default();
    args.extend(mode.args.clone().unwrap_or_default());

    let command_prefix = game
        .command_prefix
        .clone()
        .or_else(|| runner_defaults.command_prefix.clone())
        .or_else(|| global.command_prefix.clone())
        .unwrap_or_default();

    let mut env = global.env.clone().unwrap_or_default();
    env.extend(runner_defaults.env.clone().unwrap_or_default());
    env.extend(game.env.clone().unwrap_or_default());
    env.extend(mode.env.clone().unwrap_or_default());

    Ok(ResolvedConfig {
        game_id: game_id.clone(),
        target,
        root,
        executable,
        args,
        command_prefix,
        env,
    })
}
