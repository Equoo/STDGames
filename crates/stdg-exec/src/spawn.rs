//! Turns a resolved `CommandSpec` into an actual running process.

use std::os::unix::process::ExitStatusExt as _;
use std::path::Path;
use std::process::{Command, ExitStatus};

use stdg_core::{ArgValue, CommandSpec, SessionGuard};

use crate::error::ExecError;

/// Builds a `std::process::Command` from a `CommandSpec`.
pub fn to_std_command(spec: &CommandSpec) -> Result<Command, ExecError> {
    let program = spec.program.as_ref().ok_or(ExecError::MissingProgram)?;
    let mut cmd = Command::new(program.effective());

    for arg in &spec.args {
        cmd.arg(arg.render());
    }
    for (key, value) in &spec.env {
        cmd.env(key, value.render());
    }
    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd.effective());
    }

    Ok(cmd)
}

/// Spawns `spec`, hands its pid to the guards, and waits.
pub fn spawn_and_wait(
    spec: &CommandSpec,
    guards: &[Box<dyn SessionGuard>],
) -> Result<ExitStatus, ExecError> {
    let mut cmd = to_std_command(spec)?;
    let mut child = cmd.spawn().map_err(|source| ExecError::Spawn {
        program: cmd.get_program().into(),
        source,
    })?;

    for guard in guards {
        guard.adopt_pid(child.id());
    }

    let status = child.wait().map_err(ExecError::Wait)?;
    Ok(match &spec.status_file {
        Some(file) => read_status_file(file),
        None => status,
    })
}

/// The exit code written by the wrapped command; `127` if it never ran.
fn read_status_file(file: &Path) -> ExitStatus {
    let code = std::fs::read_to_string(file)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok());
    if code.is_none() {
        tracing::warn!("{} was not written: the command never ran", file.display());
    }
    ExitStatus::from_raw(code.unwrap_or(127) << 8)
}

/// One-line rendering of a command.
#[derive(Debug)]
pub struct DisplayCommand<'a>(pub &'a CommandSpec);

impl std::fmt::Display for DisplayCommand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let spec = self.0;
        let program = spec
            .program
            .as_ref()
            .map(|p| p.effective().display().to_string())
            .unwrap_or_default();
        let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
        write!(
            f,
            "{}",
            std::iter::once(program)
                .chain(args)
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
