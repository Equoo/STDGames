//! Windows target runner.
//!
//! A Windows game is never runnable on its own: it always needs a Compat
//! layer (Proton or Wine) to actually execute. This runner only builds the
//! bare, un-wrapped invocation of the game's executable — the Compat layer
//! (`stdg-layers-compat`, currently a stub) is what turns that into
//! `proton run <exe>` / `wine <exe>` via `Layer::wrap`.

use std::path::PathBuf;

use stdg_core::{ArgValue, CommandSpec, CoreError, LaunchCtx, PathValue, Runner, RunnerId, TargetKind};

pub struct WindowsRunner;

impl Runner for WindowsRunner {
    fn id(&self) -> RunnerId {
        RunnerId("windows".to_string())
    }

    fn accepts(&self, target: &TargetKind) -> bool {
        matches!(target, TargetKind::Windows)
    }

    fn build(&self, ctx: &LaunchCtx) -> Result<CommandSpec, CoreError> {
        let config = &ctx.plan.config;
        let exe_path: PathBuf = config.root.join(&config.executable);

        let mut spec = CommandSpec::new(PathValue::Host(exe_path));
        for arg in &config.args {
            spec.push_arg(ArgValue::Literal(arg.clone()));
        }
        spec.cwd = Some(PathValue::Host(config.root.clone()));
        for (k, v) in &config.env {
            spec.set_env_literal(k.clone(), v.clone());
        }

        Ok(spec)
    }
}
