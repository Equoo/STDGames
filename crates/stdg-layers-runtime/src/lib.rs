//! Runtime slot: Steam Linux Runtime (pressure-vessel), soldier or sniper.

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, Outcome, PathValue, Slot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Which Steam Linux Runtime image to use.
pub enum PressureVesselVariant {
    /// Steam Linux Runtime 2.0 (scout-compatible, older glibc).
    Soldier,
    /// Steam Linux Runtime 3.0 (newer glibc).
    Sniper,
}

impl PressureVesselVariant {
    fn layer_id(self) -> &'static str {
        match self {
            PressureVesselVariant::Soldier => "soldier",
            PressureVesselVariant::Sniper => "sniper",
        }
    }
}

/// Runs the command inside a Steam Linux Runtime container.
#[derive(Debug)]
pub struct PressureVesselLayer {
    /// Which runtime image.
    pub variant: PressureVesselVariant,
    /// Root of the installed runtime depot (contains its `run` script).
    pub depot_path: PathBuf,
}

impl PressureVesselLayer {
    fn run_script(&self) -> PathBuf {
        self.depot_path.join("run")
    }
}

impl Layer for PressureVesselLayer {
    fn id(&self) -> LayerId {
        LayerId(self.variant.layer_id().to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Runtime
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::SCOUT_LIBS])
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        let run_script = self.run_script();
        if !run_script.is_file() {
            return Err(Diagnostic::error(format!(
                "{} runtime depot not found: {} does not exist",
                self.variant.layer_id(),
                run_script.display()
            ))
            .with_hint(format!(
                "install \"Steam Linux Runtime - {}\" from your Steam library, then point `depot_path` at it",
                self.variant.layer_id()
            )));
        }
        Ok(())
    }

    fn container_needs(&self) -> Vec<Binding> {
        // Conty doesn't bind the depot itself.
        vec![Binding {
            source: PathValue::Host(self.depot_path.clone()),
            mode: BindMode::ReadWrite,
            purpose: BindPurpose(format!("{}-depot", self.variant.layer_id())),
        }]
    }

    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        let mut spec = CommandSpec::new(PathValue::Host(self.run_script()));
        spec.push_arg_literal("--");
        if let Some(program) = &inner.program {
            spec.push_arg_path(program.clone());
        }
        for arg in &inner.args {
            spec.push_arg(arg.clone());
        }
        // Keep env/cwd for the final process.
        spec.env = inner.env;
        spec.cwd = inner.cwd;

        Ok(Outcome::Direct(spec))
    }
}

#[cfg(test)]
mod tests;
