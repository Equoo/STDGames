//! RAII cleanup handles returned by `Layer::prepare`.

/// Cleans up a layer's side effects on `Drop`.
pub trait SessionGuard: Send + std::fmt::Debug {
    /// Label used in logs / `explain` output for real guards.
    fn label(&self) -> &'static str {
        "guard"
    }

    /// Called once the final process is spawned.
    fn adopt_pid(&self, _pid: u32) {}
}

/// Guard for layers with nothing to clean up.
#[derive(Debug)]
pub(crate) struct NullGuard;

impl SessionGuard for NullGuard {}
