//! Capabilities layers provide to and require from each other.

use std::collections::BTreeSet;

/// A structural capability a layer provides or requires (e.g. "windows-abi").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Capability(pub &'static str);

/// The closed set of capabilities known to the workspace.
pub mod capabilities {
    use super::Capability;

    /// Windows binaries can execute (Proton, Wine).
    pub const WINDOWS_ABI: Capability = Capability("windows-abi");
    /// Steam Linux Runtime libraries are available.
    pub const SCOUT_LIBS: Capability = Capability("scout-libs");
    /// The process runs inside a sandbox.
    pub const SANDBOXED: Capability = Capability("sandboxed");
    /// The Steam API can complete its client handshake.
    pub const STEAM_HANDSHAKE: Capability = Capability("steam-handshake");
    /// Every descendant process is tracked for teardown.
    pub const CGROUP_SUPERVISED: Capability = Capability("cgroup-supervised");
}

/// An ordered set of [`Capability`] values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet(BTreeSet<Capability>);

impl CapabilitySet {
    /// An empty set.
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    /// A set holding exactly `caps`.
    pub fn of(caps: impl IntoIterator<Item = Capability>) -> Self {
        Self(caps.into_iter().collect())
    }

    /// Adds `cap` to the set.
    pub fn insert(&mut self, cap: Capability) {
        self.0.insert(cap);
    }

    /// Whether `cap` is in the set.
    pub fn contains(&self, cap: Capability) -> bool {
        self.0.contains(&cap)
    }

    /// Every capability in either set.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).copied().collect())
    }

    /// Iterates the capabilities in order.
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.0.iter().copied()
    }

    /// Requirements in `self` missing from `provided`.
    #[must_use]
    pub fn missing_from(&self, provided: &Self) -> CapabilitySet {
        Self(self.0.difference(&provided.0).copied().collect())
    }

    /// Whether the set holds no capability.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
