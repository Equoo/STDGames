//! Native Linux runner: runs the executable from the game root.

use std::path::PathBuf;

use stdg_core::{
    ArgValue, CommandSpec, CoreError, LaunchCtx, PathValue, Runner, RunnerId, TargetKind,
};

/// Runner for [`TargetKind::NativeLinux`].
#[derive(Debug)]
pub struct NativeLinuxRunner;

impl Runner for NativeLinuxRunner {
    fn id(&self) -> RunnerId {
        RunnerId("native-linux".to_string())
    }

    fn accepts(&self, target: &TargetKind) -> bool {
        matches!(target, TargetKind::NativeLinux)
    }

    fn build(&self, ctx: &LaunchCtx) -> Result<CommandSpec, CoreError> {
        let config = &ctx.plan.config;
        let exe_path: PathBuf = config.root.join(&config.executable);

        let mut prefix = config.command_prefix.iter();
        let mut spec = match prefix.next() {
            Some(first) => {
                let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from(first)));
                for arg in prefix {
                    spec.push_arg_literal(arg.clone());
                }
                spec.push_arg_path(PathValue::Host(exe_path));
                spec
            }
            None => CommandSpec::new(PathValue::Host(exe_path)),
        };

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
