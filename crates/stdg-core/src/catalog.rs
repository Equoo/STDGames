//! Lookup traits the planner validates plans against.

use crate::{
    config::ResolvedConfig,
    error::CoreError,
    ids::{LayerId, RunnerId},
    layer::{Layer, LayerRef},
    runner::Runner,
    target::TargetKind,
};

/// Lets `stdg-plan` validate plans without linking real layers.
pub trait LayerCatalog {
    /// Builds the layer `r` names, for the game described by `config`.
    fn resolve_layer(
        &self,
        r: &LayerRef,
        config: &ResolvedConfig,
    ) -> Result<Box<dyn Layer>, CoreError>;
    /// Every layer id this catalog can resolve.
    fn known_layer_ids(&self) -> Vec<LayerId>;
}

/// Lookup of the registered [`Runner`]s.
pub trait RunnerCatalog {
    /// The first runner that accepts `target`.
    fn find_for_target(&self, target: &TargetKind) -> Option<&dyn Runner>;
    /// The runner registered under `id`.
    fn resolve_runner(&self, id: &RunnerId) -> Option<&dyn Runner>;
}
