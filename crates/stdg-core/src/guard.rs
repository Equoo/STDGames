/// A layer that produces filesystem/process side effects returns a guard
/// whose `Drop` cleans them up, including on panic or on the game crashing.
/// Most of the contract is carried by `Drop` on the concrete type; the two
/// methods here have default (no-op) implementations so a guard that
/// doesn't care about them — most don't — needs neither.
pub trait SessionGuard: Send {
    /// Label used in logs / `explain` output for real guards. Not invoked
    /// during a dry run, since `prepare` itself is skipped there.
    fn label(&self) -> &str {
        "guard"
    }

    /// Called by the executor once the final process has actually been
    /// spawned, since a guard created during `prepare` (before the command
    /// is even fully built) has no pid to act on yet. A cgroup-backed guard
    /// uses this to add the real pid to its session; anything else ignores
    /// it.
    fn adopt_pid(&self, _pid: u32) {}
}

pub(crate) struct NullGuard;

impl SessionGuard for NullGuard {}
