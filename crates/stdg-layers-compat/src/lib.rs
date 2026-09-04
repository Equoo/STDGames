//! Compat slot: Proton or Wine.
//!
//! Every Proton (and, in practice, every Proton-GE-flavored "Wine") build
//! ships its own `proton` entry-point script at its install root, invoked
//! as `<install>/proton run <exe> <args...>` with `STEAM_COMPAT_DATA_PATH`
//! (and, for compatibility, `WINEPREFIX`) pointing at the persistent
//! per-user Wine prefix, and `STEAM_COMPAT_CLIENT_INSTALL_PATH` pointing at
//! an installed Steam client. That is Valve's own documented contract for
//! `proton`, not `umu-launcher`'s — this workspace never links or shells
//! out to umu, per the top-level constraint that ruled it out from the
//! start.
//!
//! `proton_path`/`wine_path` and `prefix_path` are explicit, required
//! parameters (no version discovery, no download): the caller points this
//! layer at an already-installed build, the same way `depot_path` works for
//! `stdg-layers-runtime`.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, Outcome, PathValue, SessionGuard, Slot,
};

/// No cleanup needed: unlike a session's own scratch space, a Proton/Wine
/// prefix is meant to persist across launches, so `prepare` only needs to
/// make sure the directory exists — nothing to undo afterwards.
struct NoCleanupGuard;
impl SessionGuard for NoCleanupGuard {
    fn label(&self) -> &str {
        "compat-prefix"
    }
}

/// Builds the `<entry_point>/proton run <exe> <args...>` invocation shared
/// by Proton and this deployment's "Wine" builds alike (both ship the same
/// `proton` script; see the module docs).
fn wrap_via_proton_script(entry_point: &Path, prefix_path: &Path, steam_client_path: Option<&Path>, inner: CommandSpec) -> CommandSpec {
    let mut spec = CommandSpec::new(PathValue::Host(entry_point.join("proton")));
    spec.push_arg_literal("run");
    if let Some(program) = &inner.program {
        spec.push_arg_path(program.clone());
    }
    for arg in &inner.args {
        spec.push_arg(arg.clone());
    }
    spec.cwd = inner.cwd;
    spec.env = inner.env;

    spec.set_env_path("STEAM_COMPAT_DATA_PATH", PathValue::Host(prefix_path.to_path_buf()));
    spec.set_env_path("WINEPREFIX", PathValue::Host(prefix_path.to_path_buf()));
    if let Some(steam_client_path) = steam_client_path.map(Path::to_path_buf).or_else(find_steam_client_install) {
        spec.set_env_path("STEAM_COMPAT_CLIENT_INSTALL_PATH", PathValue::Host(steam_client_path));
    }

    spec
}

/// Generic, portable fallback locations for an installed Steam client.
/// Deployment-specific installs (a shared network install, say) belong in
/// the layer's `steam_client_path` parameter, set from `defaults/*.toml` —
/// not hardcoded here.
fn find_steam_client_install() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    [home.join(".steam/steam"), home.join(".local/share/Steam")]
        .into_iter()
        .find(|p| p.join("linux64/steamclient.so").is_file())
}

/// The Sandbox layer only binds what it's told to via `ctx.bindings`, built
/// from `container_needs()` up the pipeline — the Proton/Wine install, the
/// prefix, and the Steam client install all live outside the paths the
/// Sandbox layer binds on its own (`/usr`, the game's own root), so without
/// this bwrap would isolate the sandbox right past them: `execvp` on the
/// entry-point script would see nothing there at all.
fn container_needs_for(entry_point: &Path, prefix_path: &Path, steam_client_path: Option<&Path>, purpose_prefix: &str) -> Vec<Binding> {
    let mut needs = vec![
        Binding {
            source: PathValue::Host(entry_point.to_path_buf()),
            mode: BindMode::ReadOnly,
            purpose: BindPurpose(format!("{purpose_prefix}-install")),
        },
        Binding {
            source: PathValue::Host(prefix_path.to_path_buf()),
            mode: BindMode::ReadWrite,
            purpose: BindPurpose(format!("{purpose_prefix}-prefix")),
        },
    ];
    if let Some(steam_client_path) = steam_client_path.map(Path::to_path_buf).or_else(find_steam_client_install) {
        needs.push(Binding {
            source: PathValue::Host(steam_client_path),
            mode: BindMode::ReadOnly,
            purpose: BindPurpose("steam-client".to_string()),
        });
    }
    needs
}

fn preflight_entry_point(entry_point: &Path, kind: &str) -> Result<(), Diagnostic> {
    let script = entry_point.join("proton");
    if !script.is_file() {
        return Err(Diagnostic::error(format!("{kind} build not found: {} does not exist", script.display()))
            .with_hint(format!("point this layer's path parameter at an installed {kind} build (a directory containing its own `proton` script)")));
    }
    Ok(())
}

fn prepare_prefix(layer_id: &str, prefix_path: &Path, ctx: &LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
    if !ctx.dry_run {
        std::fs::create_dir_all(prefix_path).map_err(|e| CoreError::LayerFailure {
            layer: LayerId(layer_id.to_string()),
            reason: format!("could not create compat prefix at {}: {e}", prefix_path.display()),
        })?;
    }
    Ok(Box::new(NoCleanupGuard))
}

pub struct ProtonLayer {
    pub version: String,
    /// Root of an installed Proton build (contains its own `proton` script).
    pub proton_path: PathBuf,
    /// Persistent per-user Wine prefix for this Proton build — becomes both
    /// `STEAM_COMPAT_DATA_PATH` and `WINEPREFIX`.
    pub prefix_path: PathBuf,
    /// Overrides the generic `find_steam_client_install` fallback.
    pub steam_client_path: Option<PathBuf>,
}

impl Layer for ProtonLayer {
    fn id(&self) -> LayerId {
        LayerId("proton".to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Compat
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::WINDOWS_ABI])
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        preflight_entry_point(&self.proton_path, "Proton")
    }

    fn container_needs(&self) -> Vec<Binding> {
        container_needs_for(&self.proton_path, &self.prefix_path, self.steam_client_path.as_deref(), "proton")
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        prepare_prefix("proton", &self.prefix_path, ctx)
    }

    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        Ok(Outcome::Direct(wrap_via_proton_script(
            &self.proton_path,
            &self.prefix_path,
            self.steam_client_path.as_deref(),
            inner,
        )))
    }
}

pub struct WineLayer {
    pub version: String,
    /// Root of an installed Wine/Proton-GE-flavored build (in this
    /// ecosystem, "Wine" builds ship the exact same `proton` entry point as
    /// Proton itself — see the module docs).
    pub wine_path: PathBuf,
    pub prefix_path: PathBuf,
    pub steam_client_path: Option<PathBuf>,
}

impl Layer for WineLayer {
    fn id(&self) -> LayerId {
        LayerId("wine".to_string())
    }

    fn slot(&self) -> Slot {
        Slot::Compat
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::WINDOWS_ABI])
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        preflight_entry_point(&self.wine_path, "Wine")
    }

    fn container_needs(&self) -> Vec<Binding> {
        container_needs_for(&self.wine_path, &self.prefix_path, self.steam_client_path.as_deref(), "wine")
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        prepare_prefix("wine", &self.prefix_path, ctx)
    }

    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        Ok(Outcome::Direct(wrap_via_proton_script(
            &self.wine_path,
            &self.prefix_path,
            self.steam_client_path.as_deref(),
            inner,
        )))
    }
}

#[cfg(test)]
mod tests;
