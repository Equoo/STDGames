//! Windows target runner.

use std::path::PathBuf;

use stdg_core::{
    ArgValue, CommandSpec, CoreError, LaunchCtx, PathValue, Runner, RunnerId, TargetKind,
};

/// Runner for [`TargetKind::Windows`].
#[derive(Debug)]
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

#[cfg(test)]
mod tests;
