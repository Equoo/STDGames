/// Mutually exclusive slots, in wrapping order (outermost first). `Sandbox`
/// invokes `Runtime`, which invokes `Compat`, which invokes `SteamApi`,
/// which invokes `Supervision`, which invokes `Tooling`, which finally execs
/// the command built by the `Runner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    Sandbox,
    Runtime,
    Compat,
    SteamApi,
    Supervision,
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

    /// Application order for `patch`/`wrap`: innermost to outermost. Each
    /// layer's `wrap` receives a command already touched by every layer
    /// closer to the runner.
    pub fn application_order() -> impl Iterator<Item = Slot> {
        Self::ORDER.into_iter().rev()
    }
}
