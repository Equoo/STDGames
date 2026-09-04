//! Runtime slot: pressure-vessel, in either the "soldier" (scout-compatible,
//! older glibc) or "sniper" (newer glibc) container image — Valve's Steam
//! Linux Runtime.
//!
//! Every Steam Linux Runtime depot ships a `run` script at its root
//! (`<depot>/run -- COMMAND [ARGS...]`) that resolves `pressure-vessel-wrap`
//! and its many flags internally; this layer targets that documented,
//! stable entry point rather than constructing `pressure-vessel-wrap`'s own
//! (much larger, version-sensitive) flag set itself.
//!
//! `depot_path` must point at an already-installed depot (e.g.
//! `~/.steam/steam/steamapps/common/SteamLinuxRuntime_soldier`) — this layer
//! does not discover, download, or verify a Steam library for one; that is
//! explicitly out of scope for this workspace (no installed-games
//! detection, no Steam manifest parsing).
//!
//! Unlike `stdg-layers-sandbox`'s bwrap layer, nothing here was checked
//! against a real depot: `pressure-vessel-wrap` isn't installed in this dev
//! environment (it only ships as part of an installed Steam Linux Runtime),
//! so this is built from the documented `run` script contract, not verified
//! end to end. `preflight` fails clearly rather than silently misbehaving
//! when the depot isn't where it's configured to be.
//!
//! pressure-vessel creates its own bwrap sandbox internally, independent of
//! `stdg-layers-sandbox`'s. That's intentional, not a bug to route around:
//! nested unprivileged sandboxes are a normal, supported pattern (Flatpak
//! runs pressure-vessel-wrapped games inside its own sandbox the same way).

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, Outcome, PathValue, Slot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureVesselVariant {
    Soldier,
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

pub struct PressureVesselLayer {
    pub variant: PressureVesselVariant,
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
        // Same reasoning as stdg-layers-compat: the Sandbox layer only
        // binds what it's told, and the depot lives outside every path it
        // binds on its own.
        vec![Binding {
            source: PathValue::Host(self.depot_path.clone()),
            mode: BindMode::ReadOnly,
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
        // The `run` script execs the command itself; env/cwd set on the
        // inner spec still need to reach that final process.
        spec.env = inner.env;
        spec.cwd = inner.cwd;

        Ok(Outcome::Direct(spec))
    }
}

#[cfg(test)]
mod tests;
