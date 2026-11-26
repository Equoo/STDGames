//! Compat slot: Proton or Wine.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, Outcome, PathValue, SessionGuard, Slot, WinePrefix,
};

/// The Windows user the `proton` script creates.
const PROTON_USER: &str = "steamuser";

/// The `proton` script keeps the Wine prefix in `<STEAM_COMPAT_DATA_PATH>/pfx`.
fn proton_wine_prefix(prefix_path: &Path) -> WinePrefix {
    WinePrefix {
        drive_c: prefix_path.join("pfx/drive_c"),
        user: PROTON_USER.to_string(),
    }
}

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
    if let Some(steam_client_path) = steam_client_path {
        spec.set_env_path(
            "STEAM_COMPAT_CLIENT_INSTALL_PATH",
            PathValue::Host(steam_client_path.to_path_buf()),
        );
    }

    spec
}

/// Where `STEAM_COMPAT_CLIENT_INSTALL_PATH` points.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SteamClient {
    /// A configured or installed Steam client.
    Real(PathBuf),
    /// An empty dir: Proton only needs the variable set.
    Stub(PathBuf),
}

impl SteamClient {
    fn resolve(configured: Option<&Path>, stub: Option<&Path>) -> Option<Self> {
        configured
            .map(Path::to_path_buf)
            .or_else(find_steam_client_install)
            .map(Self::Real)
            .or_else(|| stub.map(|p| Self::Stub(p.to_path_buf())))
    }

    fn path(&self) -> &Path {
        match self {
            Self::Real(p) | Self::Stub(p) => p,
        }
    }
}

/// The running Steam client's install, else Steam in the home dir.
fn find_steam_client_install() -> Option<PathBuf> {
    running_steam_install().or_else(|| {
        let home = PathBuf::from(std::env::var_os("HOME")?);
        [home.join(".steam/steam"), home.join(".local/share/Steam")]
            .into_iter()
            .find(|p| is_steam_install(p))
    })
}

fn is_steam_install(dir: &Path) -> bool {
    dir.join("linux64/steamclient.so").is_file()
}

/// Install dir of a running `steam` process.
fn running_steam_install() -> Option<PathBuf> {
    std::fs::read_dir("/proc")
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
        })
        .find_map(|e| steam_install_of(&std::fs::read_link(e.path().join("exe")).ok()?))
}

/// Dirs the Steam client runs from: 64-bit client, then the old 32-bit one.
const STEAM_BIN_DIRS: [&str; 2] = ["steamrt64", "ubuntu12_32"];

/// `<install>` for an exe at `<install>/{steamrt64,ubuntu12_32}/steam`.
fn steam_install_of(exe: &Path) -> Option<PathBuf> {
    let bin_dir = exe.parent()?;
    let bin_name = bin_dir.file_name()?;
    if exe.file_name()? != "steam" || !STEAM_BIN_DIRS.iter().any(|d| bin_name == *d) {
        return None;
    }
    let install = bin_dir.parent()?;
    is_steam_install(install).then(|| install.to_path_buf())
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
            mode: BindMode::ReadWrite,
            purpose: BindPurpose(format!("{purpose_prefix}-install")),
        },
        Binding {
            source: PathValue::Host(prefix_path.to_path_buf()),
            mode: BindMode::ReadWrite,
            purpose: BindPurpose(format!("{purpose_prefix}-prefix")),
        },
    ];
    if let Some(steam_client_path) = steam_client_path {
        needs.push(Binding {
            source: PathValue::Host(steam_client_path.to_path_buf()),
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

/// Creates the prefix, and the Steam stub dir when it is used.
fn prepare_prefix(
    layer_id: &str,
    prefix_path: &Path,
    steam_client: Option<&SteamClient>,
    ctx: &LaunchCtx,
) -> Result<Box<dyn SessionGuard>, CoreError> {
    if ctx.is_dry_run() {
        return Ok(Box::new(NoCleanupGuard));
    }
    let mut dirs = vec![prefix_path];
    if let Some(SteamClient::Stub(stub)) = steam_client {
        dirs.push(stub);
    }
    for dir in dirs {
        std::fs::create_dir_all(dir).map_err(|e| CoreError::LayerFailure {
            layer: LayerId(layer_id.to_string()),
            reason: format!("could not create {}: {e}", dir.display()),
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
    /// Empty dir used when no Steam client is found.
    pub steam_stub_dir: Option<PathBuf>,
}

impl ProtonLayer {
    fn steam_client(&self) -> Option<SteamClient> {
        SteamClient::resolve(
            self.steam_client_path.as_deref(),
            self.steam_stub_dir.as_deref(),
        )
    }
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

    fn container_needs(&self, _ctx: &LaunchCtx) -> Vec<Binding> {
        container_needs_for(
            &self.proton_path,
            &self.prefix_path,
            self.steam_client().as_ref().map(SteamClient::path),
            "proton",
        )
    }

    fn wine_prefix(&self) -> Option<WinePrefix> {
        Some(proton_wine_prefix(&self.prefix_path))
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        prepare_prefix(
            "proton",
            &self.prefix_path,
            self.steam_client().as_ref(),
            ctx,
        )
    }

    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        Ok(Outcome::Direct(wrap_via_proton_script(
            &self.proton_path,
            &self.prefix_path,
            self.steam_client().as_ref().map(SteamClient::path),
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
    /// Empty dir used when no Steam client is found.
    pub steam_stub_dir: Option<PathBuf>,
}

impl WineLayer {
    fn steam_client(&self) -> Option<SteamClient> {
        SteamClient::resolve(
            self.steam_client_path.as_deref(),
            self.steam_stub_dir.as_deref(),
        )
    }
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

    fn container_needs(&self, _ctx: &LaunchCtx) -> Vec<Binding> {
        container_needs_for(
            &self.wine_path,
            &self.prefix_path,
            self.steam_client().as_ref().map(SteamClient::path),
            "wine",
        )
    }

    fn wine_prefix(&self) -> Option<WinePrefix> {
        Some(proton_wine_prefix(&self.prefix_path))
    }

    fn prepare(&self, ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        prepare_prefix("wine", &self.prefix_path, self.steam_client().as_ref(), ctx)
    }

    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        Ok(Outcome::Direct(wrap_via_proton_script(
            &self.wine_path,
            &self.prefix_path,
            self.steam_client().as_ref().map(SteamClient::path),
            inner,
        )))
    }
}

#[cfg(test)]
mod tests;
