use std::path::PathBuf;

use crate::{binding::Binding, ids::SessionId, plan::Plan};

pub struct SessionInfo {
    pub id: SessionId,
    pub tmp_dir: PathBuf,
}

/// Mutable state shared across layers while the pipeline runs (`stdg-exec`).
/// In `explain` mode, `dry_run` is `true`: layers must not perform real side
/// effects in `prepare`.
pub struct LaunchCtx {
    pub plan: Plan,
    pub session: SessionInfo,
    /// Filled in by the executor between each layer (inside-out) from
    /// `Layer::container_needs()`. Outer layers (e.g. Runtime) read it to
    /// know what to bind-mount into the container.
    pub bindings: Vec<Binding>,
    pub dry_run: bool,
}
