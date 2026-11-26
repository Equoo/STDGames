//! The configuration cascade: global -> runner -> game -> mode.

use std::collections::BTreeMap;

use serde::Deserialize;

use stdg_core::{
    ConfigError, CoreError, GameId, LayerRef, ModeId, PartialGameConfig, PartialModeConfig,
    ResolvedConfig,
};

/// Base of the cascade: values with no more-specific override anywhere.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalDefaults {
    /// Lowest-precedence `command_prefix`.
    #[serde(default)]
    pub command_prefix: Option<Vec<String>>,
    /// Lowest-precedence environment.
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    /// Layers applied to every mode of every game.
    #[serde(default)]
    pub baseline_layers: Vec<LayerRef>,
}

/// Runner-level defaults.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RunnerDefaults {
    /// `command_prefix` for every game of this runner.
    #[serde(default)]
    pub command_prefix: Option<Vec<String>>,
    /// Environment for every game of this runner.
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    /// Layers for every mode of this runner.
    #[serde(default)]
    pub layers: Vec<LayerRef>,
    /// Default layers per mode.
    #[serde(default)]
    pub modes: BTreeMap<ModeId, ModeLayerDefaults>,
}

/// A runner's default layers for one mode.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModeLayerDefaults {
    /// The layers, at most one per slot.
    #[serde(default)]
    pub layers: Vec<LayerRef>,
}

impl RunnerDefaults {
    /// This runner's default layers for `mode_id` (empty when it has none).
    pub fn layers_for(&self, mode_id: &ModeId) -> Vec<LayerRef> {
        self.modes
            .get(mode_id)
            .map(|m| m.layers.clone())
            .unwrap_or_default()
    }
}

/// Merges `global -> runner -> game -> mode` into a `ResolvedConfig`.
pub fn resolve_config(
    game_id: &GameId,
    global: &GlobalDefaults,
    runner_defaults: &RunnerDefaults,
    game: &PartialGameConfig,
    mode: &PartialModeConfig,
) -> Result<ResolvedConfig, CoreError> {
    let target = game
        .target
        .clone()
        .ok_or_else(|| ConfigError::MissingField {
            game_id: game_id.0.clone(),
            field: "target",
        })?;
    let root = game.root.clone().ok_or_else(|| ConfigError::MissingField {
        game_id: game_id.0.clone(),
        field: "root",
    })?;
    let executable = game
        .executable
        .clone()
        .ok_or_else(|| ConfigError::MissingField {
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
        app_id: game.app_id.clone(),
        target,
        root,
        executable,
        args,
        command_prefix,
        env,
        saves: game.saves.clone().unwrap_or_default(),
    })
}
