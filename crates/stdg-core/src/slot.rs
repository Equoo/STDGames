//! Layer slots and their wrapping order.

/// Mutually exclusive slots, in wrapping order (outermost first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    /// Container isolation (Conty).
    Sandbox,
    /// Steam Linux Runtime (pressure-vessel).
    Runtime,
    /// Windows compatibility (Proton, Wine).
    Compat,
    /// Steam API handling (native or emulated).
    SteamApi,
    /// Process supervision (cgroup).
    Supervision,
    /// Extra tooling (overlays, profilers...).
    Tooling,
}

impl Slot {
    /// Wrapping order, outermost to innermost.
    pub const ORDER: [Slot; 6] = [
        Slot::Sandbox,
        Slot::Runtime,
        Slot::Compat,
        Slot::SteamApi,
        Slot::Supervision,
        Slot::Tooling,
    ];

    /// Application order for `patch`/`wrap`: innermost to outermost.
    pub fn application_order() -> impl Iterator<Item = Slot> {
        Self::ORDER.into_iter().rev()
    }
}
