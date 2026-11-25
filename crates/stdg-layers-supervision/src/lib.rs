//! Supervision slot: cgroup v2 leaf, or subreaper fallback.

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{CapabilitySet, CoreError, LaunchCtx, Layer, LayerId, SessionGuard, Slot};
use stdg_exec::cgroup::CgroupSession;
use stdg_exec::subreaper;

/// Puts the launched process tree under supervision.
#[derive(Debug)]
pub struct SupervisionLayer {
    /// Delegated cgroup v2 directory sessions are created under.
    pub cgroup_parent: PathBuf,
}

impl SupervisionLayer {
    /// Supervision under the cgroup this process already runs in.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cgroup_parent: default_cgroup_parent(),
        }
    }
}

impl Default for SupervisionLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Layer for SupervisionLayer {
    fn id(&self) -> LayerId {
        LayerId("cgroup".to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Supervision
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::CGROUP_SUPERVISED])
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        if ctx.is_dry_run() {
            return Ok(Box::new(FallbackGuard));
        }

        match CgroupSession::create(&self.cgroup_parent, &ctx.session.id.0) {
            Ok(Some(session)) => return Ok(Box::new(session)),
            Ok(None) => tracing::info!(
                "cgroup v2 delegation unavailable at {}, using the subreaper fallback",
                self.cgroup_parent.display()
            ),
            // Fall back rather than fail the launch.
            Err(e) => tracing::warn!(
                "could not create a cgroup session under {}: {e}; using the subreaper fallback",
                self.cgroup_parent.display()
            ),
        }
        subreaper::set_child_subreaper().map_err(|e| CoreError::LayerFailure {
            layer: self.id(),
            reason: format!("neither cgroup nor subreaper supervision is available: {e}"),
        })?;
        Ok(Box::new(FallbackGuard))
    }
}

#[derive(Debug)]
struct FallbackGuard;

impl SessionGuard for FallbackGuard {
    fn label(&self) -> &'static str {
        "subreaper-fallback"
    }
}

/// cgroup v2 path of the current process.
fn default_cgroup_parent() -> PathBuf {
    // No `/proc`: use the root; `prepare` then falls back to the subreaper.
    let content = std::fs::read_to_string("/proc/self/cgroup").ok();
    content
        .as_deref()
        .and_then(unified_cgroup_path)
        .map_or_else(
            || PathBuf::from("/sys/fs/cgroup"),
            |rel| PathBuf::from("/sys/fs/cgroup").join(rel.trim_start_matches('/')),
        )
}

/// Extracts the cgroup v2 path from the contents of `/proc/self/cgroup`.
fn unified_cgroup_path(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let mut parts = line.splitn(3, ':');
        let hierarchy_id = parts.next()?;
        let controllers = parts.next()?;
        let path = parts.next()?;
        // cgroup v2 line: `0::/path`.
        (hierarchy_id == "0" && controllers.is_empty()).then(|| path.to_string())
    })
}

#[cfg(test)]
mod tests;
