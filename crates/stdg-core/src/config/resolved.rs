use std::{collections::BTreeMap, path::PathBuf};

use crate::{ids::GameId, target::TargetKind};

/// Config after cascade resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    /// The game this configuration belongs to.
    pub game_id: GameId,
    /// Steam app id.
    pub app_id: Option<String>,
    /// What kind of game this is.
    pub target: TargetKind,
    /// Install directory.
    pub root: PathBuf,
    /// Executable (or ROM), relative to `root`.
    pub executable: PathBuf,
    /// Arguments passed to the executable.
    pub args: Vec<String>,
    /// Programs prepended to the command.
    pub command_prefix: Vec<String>,
    /// Environment variables set for the game.
    pub env: BTreeMap<String, String>,
}
