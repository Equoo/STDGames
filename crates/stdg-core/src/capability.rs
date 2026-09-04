use std::collections::BTreeSet;

/// A structural capability a layer provides or requires (e.g. "windows-abi").
/// A closed vocabulary defined by layer authors, never by configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capability(pub &'static str);

pub mod capabilities {
    use super::Capability;

    pub const WINDOWS_ABI: Capability = Capability("windows-abi");
    pub const SCOUT_LIBS: Capability = Capability("scout-libs");
    pub const SANDBOXED: Capability = Capability("sandboxed");
    pub const STEAM_HANDSHAKE: Capability = Capability("steam-handshake");
    pub const CGROUP_SUPERVISED: Capability = Capability("cgroup-supervised");
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<Capability>);

impl CapabilitySet {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn of(caps: impl IntoIterator<Item = Capability>) -> Self {
        Self(caps.into_iter().collect())
    }

    pub fn insert(&mut self, cap: Capability) {
        self.0.insert(cap);
    }

    pub fn contains(&self, cap: Capability) -> bool {
        self.0.contains(&cap)
    }

    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).copied().collect())
    }

    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.0.iter().copied()
    }

    /// Capabilities in `self` (i.e. requirements) absent from `provided`
    /// (the union of every layer's `provides()` in the plan).
    pub fn missing_from(&self, provided: &Self) -> CapabilitySet {
        Self(self.0.difference(&provided.0).copied().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
