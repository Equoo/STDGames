//! `stdgames run --game <id> --mode <mode>`
//!
//! Resolves the plan exactly like `explain`, but actually builds the
//! session and spawns the real process instead of stopping at a dry run:
//! layers' `prepare()` runs for real (a cgroup session gets created, a
//! Proton prefix directory gets created if missing...), and the final
//! command is genuinely executed, blocking until it exits.

use stdg_core::LaunchCtx;
use stdg_exec::{run_pipeline, spawn_and_wait, DisplayCommand};

use crate::loading::load_plan;

pub fn run(game_id_str: &str, mode_id_str: &str) -> Result<i32, String> {
    let (registry, plan) = load_plan(game_id_str, mode_id_str)?;

    println!("launching {} ({})", plan.game_id, plan.mode_id);

    let session = stdg_exec::new_session_info(&std::env::temp_dir());
    println!("session: {}", session.id);

    let ctx = LaunchCtx {
        plan,
        session,
        bindings: Vec::new(),
        dry_run: false,
    };

    let output = run_pipeline(&registry, ctx).map_err(|e| e.to_string())?;
    let spec = output.outcome.into_command();

    println!("command: {}", DisplayCommand(&spec));
    println!();

    let status = spawn_and_wait(&spec, &output.guards).map_err(|e| e.to_string())?;

    println!("exited with {status}");
    Ok(status.code().unwrap_or(-1))
}
