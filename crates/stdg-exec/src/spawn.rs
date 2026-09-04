//! Turns a resolved `CommandSpec` into an actual running process. Separate
//! from `pipeline.rs`: building the command is pure-ish (aside from the
//! layers' own `prepare` side effects), spawning it is the one place that
//! genuinely starts a process — kept small and easy to audit on its own.

use std::process::{Command, ExitStatus};

use stdg_core::{ArgValue, CommandSpec, SessionGuard};

use crate::error::ExecError;

/// Builds a `std::process::Command` from a `CommandSpec`. The outer
/// process's own environment isn't cleared here: for a plan whose Sandbox
/// layer emitted its own `--clearenv`/`--setenv` bwrap arguments, that
/// already isolates the *sandboxed* process's environment, so inheriting
/// the launcher's environment for the (short-lived, immediately-exec'd)
/// wrapper process itself is harmless — `CommandSpec.env` entries are still
/// applied on top, taking precedence.
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

/// Spawns `spec`, hands the resulting pid to every guard (so a cgroup-backed
/// one can put it under supervision), then blocks until it exits.
pub fn spawn_and_wait(spec: &CommandSpec, guards: &[Box<dyn SessionGuard>]) -> Result<ExitStatus, ExecError> {
    let mut cmd = to_std_command(spec)?;
    let mut child = cmd.spawn()?;

    for guard in guards {
        guard.adopt_pid(child.id());
    }

    Ok(child.wait()?)
}

/// Wraps a `&CommandSpec` for a one-line `{}` rendering (e.g. for a log
/// line right before spawning it).
pub struct DisplayCommand<'a>(pub &'a CommandSpec);

impl std::fmt::Display for DisplayCommand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let spec = self.0;
        let program = spec.program.as_ref().map(|p| p.effective().display().to_string()).unwrap_or_default();
        let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
        write!(f, "{}", std::iter::once(program).chain(args).collect::<Vec<_>>().join(" "))
    }
}
