//! Real, generic runner for anything ROM-based.

mod manifest;

use std::path::PathBuf;

pub use manifest::EmulatorManifest;

use stdg_core::{CommandSpec, CoreError, LaunchCtx, PathValue, Runner, RunnerId, TargetKind};

/// Runs ROMs through the first manifest supporting their platform.
#[derive(Debug)]
pub struct EmulatorRunner {
    manifests: Vec<EmulatorManifest>,
}

impl EmulatorRunner {
    /// A runner backed by `manifests`, tried in order.
    pub fn new(manifests: Vec<EmulatorManifest>) -> Self {
        Self { manifests }
    }

    fn manifest_for(&self, platform: &str) -> Option<&EmulatorManifest> {
        self.manifests
            .iter()
            .find(|m| m.supports_platform(platform))
    }
}

impl Runner for EmulatorRunner {
    fn id(&self) -> RunnerId {
        RunnerId("emulator".to_string())
    }

    fn accepts(&self, target: &TargetKind) -> bool {
        match target {
            TargetKind::Rom(platform) => self.manifest_for(&platform.0).is_some(),
            _ => false,
        }
    }

    fn build(&self, ctx: &LaunchCtx) -> Result<CommandSpec, CoreError> {
        let config = &ctx.plan.config;
        let TargetKind::Rom(platform) = &ctx.plan.target else {
            return Err(CoreError::NoRunnerForTarget(ctx.plan.target.clone()));
        };
        let manifest = self
            .manifest_for(&platform.0)
            .ok_or_else(|| CoreError::NoRunnerForTarget(ctx.plan.target.clone()))?;

        let rom_path = config.root.join(&config.executable);
        let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from(&manifest.binary)));
        for token in &manifest.args {
            if token == "{rom}" {
                spec.push_arg_path(PathValue::Host(rom_path.clone()));
            } else {
                spec.push_arg_literal(token.clone());
            }
        }
        for arg in &config.args {
            spec.push_arg_literal(arg.clone());
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
