//! Executor: runs a `Plan`'s pipeline and spawns the result.

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
pub use pipeline::{PipelineOutput, run_pipeline};
pub use session::new_session_info;
pub use session_guard::SessionTmpDir;
pub use spawn::{DisplayCommand, spawn_and_wait, to_std_command};

/// Runs the pipeline and spawns the command, blocking until it exits.
pub fn launch(registry: &Registry, ctx: LaunchCtx) -> Result<ExitStatus, ExecError> {
    let output = run_pipeline(registry, ctx)?;
    let spec = output.outcome.into_command();
    spawn_and_wait(&spec, &output.guards)
}

#[cfg(test)]
mod tests;
