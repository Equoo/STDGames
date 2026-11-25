//! `stdgames run --game <id> --mode <mode>`

use std::os::unix::process::ExitStatusExt as _;
use std::path::PathBuf;
use std::process::ExitStatus;

use stdg_core::{LaunchCtx, RunKind};
use stdg_exec::{DisplayCommand, run_pipeline, spawn_and_wait};

use crate::loading::load_plan;

/// What to start at the core of the wrapped command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Program {
    /// The game's own executable.
    Game,
    /// `/bin/bash`, to inspect the environment the game would get.
    Shell,
}

#[expect(clippy::print_stdout, reason = "user-facing CLI output")]
pub(crate) fn run(game_id_str: &str, mode_id_str: &str, program: Program) -> anyhow::Result<u8> {
    let (registry, mut plan) = load_plan(game_id_str, mode_id_str)?;

    if program == Program::Shell {
        plan.config.executable = PathBuf::from("/bin/bash");
        plan.config.args = vec![];
    }

    println!("launching {} ({})", plan.game_id, plan.mode_id);

    let session = stdg_exec::new_session_info(&std::env::temp_dir());
    println!("session: {}", session.id);

    let ctx = LaunchCtx {
        plan,
        session,
        bindings: Vec::new(),
        run_kind: RunKind::Live,
    };

    let output = run_pipeline(&registry, ctx)?;
    let spec = output.outcome.into_command();

    println!("command: {}", DisplayCommand(&spec));
    println!();

    let status = spawn_and_wait(&spec, &output.guards)?;

    println!("exited with {status}");
    Ok(exit_code(status))
}

/// Shell-style exit code: the process's code, or `128 + signal`.
fn exit_code(status: ExitStatus) -> u8 {
    let code = status
        .code()
        .or_else(|| status.signal().map(|s| 128 + s))
        .unwrap_or(1);
    u8::try_from(code).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_passes_through_normal_exits() {
        assert_eq!(exit_code(ExitStatus::from_raw(0)), 0);
        assert_eq!(exit_code(ExitStatus::from_raw(3 << 8)), 3);
    }

    #[test]
    fn exit_code_maps_signals_like_a_shell() {
        // Raw wait status 9 = killed by SIGKILL.
        assert_eq!(exit_code(ExitStatus::from_raw(9)), 137);
    }
}
