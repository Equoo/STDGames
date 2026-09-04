//! Executor: walks a `Plan`, instantiates layers through the registry, and
//! runs the real (or dry-run) pipeline. Owns everything that is genuinely
//! I/O: cgroups, tmpdirs, process supervision.

pub mod cgroup;
pub mod error;
pub mod pipeline;
pub mod session;
pub mod session_guard;
pub mod spawn;
pub mod subreaper;

use std::process::ExitStatus;

use stdg_core::LaunchCtx;
use stdg_registry::Registry;

pub use error::ExecError;
pub use pipeline::{run_pipeline, PipelineOutput};
pub use session::new_session_info;
pub use session_guard::SessionTmpDir;
pub use spawn::{spawn_and_wait, to_std_command, DisplayCommand};

/// Runs the full pipeline for `ctx` and spawns the resulting command for
/// real, blocking until it exits. `ctx.dry_run` must be `false` — this is
/// the one function in the crate that actually launches something.
pub fn launch(registry: &Registry, ctx: LaunchCtx) -> Result<ExitStatus, ExecError> {
    let output = run_pipeline(registry, ctx)?;
    let spec = output.outcome.into_command();
    spawn_and_wait(&spec, &output.guards)
}
