//! Real, generic runner for anything ROM-based. Behavior is entirely driven
//! by `EmulatorManifest`s handed in at construction (typically loaded from
//! `*.toml` files by the caller — this crate does not itself discover or
//! parse files on disk, to keep loading a `stdg-cli` concern).

mod manifest;

use std::path::PathBuf;

pub use manifest::EmulatorManifest;

use stdg_core::{CommandSpec, CoreError, LaunchCtx, PathValue, Runner, RunnerId, TargetKind};

pub struct EmulatorRunner {
    manifests: Vec<EmulatorManifest>,
}

impl EmulatorRunner {
    pub fn new(manifests: Vec<EmulatorManifest>) -> Self {
        Self { manifests }
    }

    fn manifest_for(&self, platform: &str) -> Option<&EmulatorManifest> {
        self.manifests.iter().find(|m| m.supports_platform(platform))
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
        let platform = match &ctx.plan.target {
            TargetKind::Rom(platform) => platform,
            _ => return Err(CoreError::NoRunnerForTarget(ctx.plan.target.clone())),
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
