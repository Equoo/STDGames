//! Compat slot: Proton or Wine.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, Outcome, PathValue, SessionGuard, Slot,
};

/// Prefixes persist across launches: nothing to clean up.
#[derive(Debug)]
struct NoCleanupGuard;
impl SessionGuard for NoCleanupGuard {
    fn label(&self) -> &'static str {
        "compat-prefix"
    }
}

/// Builds `<entry_point>/proton run <exe> <args...>`.
fn wrap_via_proton_script(
    entry_point: &Path,
    prefix_path: &Path,
    steam_client_path: Option<&Path>,
    inner: CommandSpec,
) -> CommandSpec {
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

    spec.set_env_path(
        "STEAM_COMPAT_DATA_PATH",
        PathValue::Host(prefix_path.to_path_buf()),
    );
    spec.set_env_path("WINEPREFIX", PathValue::Host(prefix_path.to_path_buf()));
    if let Some(steam_client_path) = steam_client_path
        .map(Path::to_path_buf)
        .or_else(find_steam_client_install)
    {
        spec.set_env_path(
            "STEAM_COMPAT_CLIENT_INSTALL_PATH",
            PathValue::Host(steam_client_path),
        );
    }

    spec
}

/// Generic, portable fallback locations for an installed Steam client.
fn find_steam_client_install() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    [home.join(".steam/steam"), home.join(".local/share/Steam")]
        .into_iter()
        .find(|p| p.join("linux64/steamclient.so").is_file())
}

/// Binds the install, prefix and Steam client.
fn container_needs_for(
    entry_point: &Path,
    prefix_path: &Path,
    steam_client_path: Option<&Path>,
    purpose_prefix: &str,
) -> Vec<Binding> {
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
    if let Some(steam_client_path) = steam_client_path
        .map(Path::to_path_buf)
        .or_else(find_steam_client_install)
    {
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

fn prepare_prefix(
    layer_id: &str,
    prefix_path: &Path,
    ctx: &LaunchCtx,
) -> Result<Box<dyn SessionGuard>, CoreError> {
    if !ctx.is_dry_run() {
        std::fs::create_dir_all(prefix_path).map_err(|e| CoreError::LayerFailure {
            layer: LayerId(layer_id.to_string()),
            reason: format!(
                "could not create compat prefix at {}: {e}",
                prefix_path.display()
            ),
        })?;
    }
    Ok(Box::new(NoCleanupGuard))
}

/// Runs Windows games through an installed Proton build.
#[derive(Debug)]
pub struct ProtonLayer {
    /// Informational version label, shown in diagnostics only.
    pub version: Option<String>,
    /// Root of an installed Proton build (contains its own `proton` script).
    pub proton_path: PathBuf,
    /// Persistent Wine prefix (`STEAM_COMPAT_DATA_PATH`, `WINEPREFIX`).
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
        container_needs_for(
            &self.proton_path,
            &self.prefix_path,
            self.steam_client_path.as_deref(),
            "proton",
        )
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

/// Runs Windows games through a Wine build shipping a `proton` script.
#[derive(Debug)]
pub struct WineLayer {
    /// Informational version label, shown in diagnostics only.
    pub version: Option<String>,
    /// Root of a Wine build shipping a `proton` script.
    pub wine_path: PathBuf,
    /// Persistent per-user Wine prefix for this build.
    pub prefix_path: PathBuf,
    /// Overrides the generic `find_steam_client_install` fallback.
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
        container_needs_for(
            &self.wine_path,
            &self.prefix_path,
            self.steam_client_path.as_deref(),
            "wine",
        )
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
