use crate::{
    config::ResolvedConfig,
    error::CoreError,
    ids::{LayerId, RunnerId},
    layer::{Layer, LayerRef},
    runner::Runner,
    target::TargetKind,
};

/// Dependency inversion boundary: `stdg-plan` validates a `Plan` through
/// these traits without ever linking the real implementations (bwrap,
/// Proton...). Planner tests supply a fake catalog.
pub trait LayerCatalog {
    fn resolve_layer(&self, r: &LayerRef, config: &ResolvedConfig) -> Result<Box<dyn Layer>, CoreError>;
    fn known_layer_ids(&self) -> Vec<LayerId>;
}

pub trait RunnerCatalog {
    fn find_for_target(&self, target: &TargetKind) -> Option<&dyn Runner>;
    fn resolve_runner(&self, id: &RunnerId) -> Option<&dyn Runner>;
}
