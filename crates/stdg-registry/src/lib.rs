//! Concrete `LayerCatalog` / `RunnerCatalog` implementation.
//!
//! `stdg-plan` only ever sees these through `stdg_core`'s trait objects, so
//! it never links a real runner or layer crate. `stdg-cli` is the only
//! place that pulls in every implementation crate and builds a `Registry`.

use std::collections::BTreeMap;

use stdg_core::{
    CoreError, LayerCatalog, LayerId, LayerRef, ResolvedConfig, Runner, RunnerCatalog, RunnerId,
    Layer, TargetKind,
};

/// Builds a layer instance from its `LayerRef` (id + params) and the
/// resolved game config. Registered per `LayerId`.
pub type LayerFactory = Box<dyn Fn(&LayerRef, &ResolvedConfig) -> Result<Box<dyn Layer>, CoreError> + Send + Sync>;

#[derive(Default)]
pub struct Registry {
    layer_factories: BTreeMap<LayerId, LayerFactory>,
    runners: Vec<Box<dyn Runner>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_layer(&mut self, id: LayerId, factory: LayerFactory) {
        self.layer_factories.insert(id, factory);
    }

    pub fn register_runner(&mut self, runner: Box<dyn Runner>) {
        self.runners.push(runner);
    }
}

impl LayerCatalog for Registry {
    fn resolve_layer(&self, r: &LayerRef, config: &ResolvedConfig) -> Result<Box<dyn Layer>, CoreError> {
        let factory = self
            .layer_factories
            .get(&r.id)
            .ok_or_else(|| CoreError::UnknownLayer(r.id.clone()))?;
        factory(r, config)
    }

    fn known_layer_ids(&self) -> Vec<LayerId> {
        self.layer_factories.keys().cloned().collect()
    }
}

impl RunnerCatalog for Registry {
    fn find_for_target(&self, target: &TargetKind) -> Option<&dyn Runner> {
        self.runners
            .iter()
            .find(|r| r.accepts(target))
            .map(|r| r.as_ref())
    }

    fn resolve_runner(&self, id: &RunnerId) -> Option<&dyn Runner> {
        self.runners.iter().find(|r| r.id() == *id).map(|r| r.as_ref())
    }
}
