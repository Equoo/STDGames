//! Errors raised while running a plan.

use std::path::PathBuf;

use thiserror::Error;

use stdg_core::CoreError;

/// Errors raised by the executor.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExecError {
    /// A planning or layer error.
    #[error(transparent)]
    Core(#[from] CoreError),

    /// The final command could not be started.
    #[error("failed to spawn {}", program.display())]
    Spawn {
        /// The program that failed to start.
        program: PathBuf,
        /// The underlying OS error.
        #[source]
        source: std::io::Error,
    },

    /// Waiting for the spawned process failed.
    #[error("failed to wait for the launched process")]
    Wait(#[source] std::io::Error),

    /// The pipeline produced a command with no program.
    #[error("the final command has no program set")]
    MissingProgram,
}
