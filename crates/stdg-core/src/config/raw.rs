use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;

use crate::{ids::ModeId, layer::LayerRef, target::TargetKind};

/// One layer of the resolution cascade: global defaults -> runner defaults
/// -> game config -> mode override. `Option<T>` only ever appears here;
/// everything downstream sees a `ResolvedConfig` with no `Option`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialGameConfig {
    pub target: Option<TargetKind>,
    pub root: Option<PathBuf>,
    pub executable: Option<PathBuf>,
    pub args: Option<Vec<String>>,
    pub command_prefix: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
    pub modes: Option<BTreeMap<ModeId, PartialModeConfig>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialModeConfig {
    pub enabled: Option<bool>,
    pub layers: Option<Vec<LayerRef>>,
    pub args: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
}
