use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;

use crate::{config::SavesConfig, ids::ModeId, layer::LayerRef, target::TargetKind};

/// One tier of the config cascade.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialGameConfig {
    /// What kind of game this is.
    pub target: Option<TargetKind>,
    /// Steam app id, used by the Steam API layers and `${game.app_id}`.
    pub app_id: Option<String>,
    /// Install directory.
    pub root: Option<PathBuf>,
    /// Executable (or ROM), relative to `root`.
    pub executable: Option<PathBuf>,
    /// Extra arguments.
    pub args: Option<Vec<String>>,
    /// Programs prepended to the command (e.g. `gamemoderun`).
    pub command_prefix: Option<Vec<String>>,
    /// Extra environment variables.
    pub env: Option<BTreeMap<String, String>>,
    /// Layers for every mode of this game.
    pub layers: Option<Vec<LayerRef>>,
    /// Save locations and profile.
    pub saves: Option<SavesConfig>,
    /// Per-mode overrides.
    pub modes: Option<BTreeMap<ModeId, PartialModeConfig>>,
}

/// One mode's override of the game configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialModeConfig {
    /// `Some(false)` disables the mode.
    pub enabled: Option<bool>,
    /// Per-slot layer overrides.
    pub layers: Option<Vec<LayerRef>>,
    /// Arguments appended to the game's.
    pub args: Option<Vec<String>>,
    /// Environment variables merged over the game's.
    pub env: Option<BTreeMap<String, String>>,
}
