//! The `Layer` trait and its configuration-side reference.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    binding::Binding,
    capability::CapabilitySet,
    command::CommandSpec,
    ctx::LaunchCtx,
    error::{CoreError, Diagnostic},
    guard::{NullGuard, SessionGuard},
    ids::LayerId,
    outcome::Outcome,
    slot::Slot,
};

/// A layer id plus its parameters, as written in config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerRef {
    /// Which layer.
    pub id: LayerId,
    /// Free-form `key = value` parameters, interpreted by the layer.
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

impl LayerRef {
    /// A reference to layer `id` with no parameters.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: LayerId(id.into()),
            params: BTreeMap::new(),
        }
    }

    /// Adds or replaces one parameter.
    #[must_use]
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// The value of parameter `key`, if set.
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }
}

/// Something that wraps or modifies the runner.
pub trait Layer: Send + Sync + std::fmt::Debug {
    /// The id this layer is registered under.
    fn id(&self) -> LayerId;
    /// The one slot this layer occupies.
    fn slot(&self) -> Slot;

    /// Capabilities this layer brings to the plan.
    fn provides(&self) -> CapabilitySet {
        CapabilitySet::default()
    }

    /// Capabilities some other layer of the plan must provide.
    fn requires(&self) -> CapabilitySet {
        CapabilitySet::default()
    }

    /// Pre-launch checks (binary present, compatible version...).
    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        Ok(())
    }

    /// What this layer needs to cross the container boundary.
    fn container_needs(&self) -> Vec<Binding> {
        Vec::new()
    }

    /// Filesystem/process side effects (RAII).
    fn prepare(&self, _ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        Ok(Box::new(NullGuard))
    }

    /// Adds args/env to `spec` without changing its program.
    fn patch(&self, _spec: &mut CommandSpec, _ctx: &LaunchCtx) -> Result<(), CoreError> {
        Ok(())
    }

    /// Wraps the command (e.g. `proton run <exe>`).
    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        Ok(Outcome::Direct(inner))
    }
}
