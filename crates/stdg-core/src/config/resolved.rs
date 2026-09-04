use std::{collections::BTreeMap, path::PathBuf};

use crate::{ids::GameId, target::TargetKind};

/// Fully resolved view: the only one seen by the planner (after resolution),
/// by runners, and by layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub game_id: GameId,
    pub target: TargetKind,
    pub root: PathBuf,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub command_prefix: Vec<String>,
    pub env: BTreeMap<String, String>,
}
