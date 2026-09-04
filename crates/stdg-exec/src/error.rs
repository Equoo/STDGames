use std::path::PathBuf;

use thiserror::Error;

use stdg_core::CoreError;

#[derive(Debug, Error)]
pub enum ExecError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cgroup v2 delegation is not available at {0}")]
    CgroupUnavailable(PathBuf),

    #[error("the final command has no program set")]
    MissingProgram,
}
