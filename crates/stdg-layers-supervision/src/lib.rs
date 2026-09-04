//! Real, simple layer: a dedicated cgroup v2 leaf per session when
//! delegation is available, falling back to `PR_SET_CHILD_SUBREAPER` when
//! it is not. No external `reaper` binary — see `stdg_exec::cgroup` and
//! `stdg_exec::subreaper` for the actual mechanics; this layer only wires
//! them into the pipeline.

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{CapabilitySet, CoreError, LaunchCtx, Layer, LayerId, SessionGuard, Slot};
use stdg_exec::cgroup::CgroupSession;
use stdg_exec::subreaper;

pub struct SupervisionLayer {
    pub cgroup_parent: PathBuf,
}

impl SupervisionLayer {
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
        if ctx.dry_run {
            return Ok(Box::new(FallbackGuard));
        }

        match CgroupSession::create(&self.cgroup_parent, &ctx.session.id.0) {
            Ok(Some(session)) => Ok(Box::new(session)),
            // Delegation unavailable, or creating the leaf failed (e.g. no
            // write permission): fall back rather than fail the launch.
            Ok(None) | Err(_) => {
                let _ = subreaper::set_child_subreaper();
                Ok(Box::new(FallbackGuard))
            }
        }
    }
}

struct FallbackGuard;

impl SessionGuard for FallbackGuard {
    fn label(&self) -> &str {
        "subreaper-fallback"
    }
}

/// Resolves the delegated cgroup v2 leaf the current process already runs
/// in (typically a systemd --user scope), by reading the unified hierarchy
/// entry from `/proc/self/cgroup`. Sessions are created as children of it.
fn default_cgroup_parent() -> PathBuf {
    current_cgroup_path()
        .map(|rel| PathBuf::from("/sys/fs/cgroup").join(rel.trim_start_matches('/')))
        .unwrap_or_else(|| PathBuf::from("/sys/fs/cgroup"))
}

fn current_cgroup_path() -> Option<String> {
    let content = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    content.lines().find_map(|line| {
        let mut parts = line.splitn(3, ':');
        let hierarchy_id = parts.next()?;
        let controllers = parts.next()?;
        let path = parts.next()?;
        // cgroup v2's unified hierarchy is reported as a single "0::/path"
        // line (empty controller list, hierarchy id 0).
        (hierarchy_id == "0" && controllers.is_empty()).then(|| path.to_string())
    })
}
