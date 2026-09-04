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

/// Unresolved reference to a layer plus its parameters (e.g. a Proton
/// version). This is the shape a mode uses to declare its layers in
/// configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerRef {
    pub id: LayerId,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

impl LayerRef {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: LayerId(id.into()),
            params: BTreeMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(String::as_str)
    }
}

/// Something that wraps or modifies the runner. Assigned by the mode, one
/// slot at a time. Hooks not marked mandatory carry a default
/// implementation, so a simple layer only has to write `id`, `slot`, and one
/// of `patch`/`wrap`.
pub trait Layer: Send + Sync {
    fn id(&self) -> LayerId;
    fn slot(&self) -> Slot;

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::default()
    }

    fn requires(&self) -> CapabilitySet {
        CapabilitySet::default()
    }

    /// Pre-launch checks (binary present, compatible version...). May
    /// perform read-only I/O; never invoked expecting mutation during a dry
    /// run.
    fn preflight(&self, ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        let _ = ctx;
        Ok(())
    }

    /// What this layer needs to cross the container boundary. Pure: derived
    /// from the layer's own fields (built from `LayerRef` + `ResolvedConfig`
    /// at construction time), never from a `prepare` side effect.
    fn container_needs(&self) -> Vec<Binding> {
        Vec::new()
    }

    /// Filesystem/process side effects (RAII). The returned guard cleans up
    /// in its `Drop`. During a dry run (`ctx.dry_run`), an implementation
    /// must avoid real mutation and return a no-op guard.
    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        let _ = ctx;
        Ok(Box::new(NullGuard))
    }

    /// Modifies `spec` in place without changing the root program (adding
    /// args/env). E.g. SteamApi::Native sets `SteamAppId` in the environment.
    fn patch(&self, spec: &mut CommandSpec, ctx: &LaunchCtx) -> Result<(), CoreError> {
        let _ = (spec, ctx);
        Ok(())
    }

    /// Restructures the command (e.g. Proton turns `<exe>` into an argument
    /// of `proton run`). Receives the command already patched by every layer
    /// closer to the runner.
    fn wrap(&self, inner: CommandSpec, ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        let _ = ctx;
        Ok(Outcome::Direct(inner))
    }
}
