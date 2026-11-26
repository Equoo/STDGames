//! `SteamApi` slot using the real Steam client.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx,
    Layer, LayerId, PathValue, Slot,
};

/// Lets the game talk to the real Steam client.
#[derive(Debug)]
pub struct SteamApiNativeLayer {
    /// The game's Steam app id, exported as `SteamAppId`/`SteamGameId`.
    pub app_id: String,
    /// Running Steam's `.steam` dir, mounted over `~/.steam` when they differ.
    pub steam_config: Option<SteamConfigBind>,
}

/// Mounts the running Steam's config dir (`sdk64`, `steam.pipe`...) where the game looks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamConfigBind {
    /// The running Steam's `.steam` dir.
    pub host: PathBuf,
    /// `~/.steam` of the launching user.
    pub guest: PathBuf,
}

impl SteamConfigBind {
    /// For the running Steam client, if its `.steam` is not already `~/.steam`.
    #[must_use]
    pub fn detect() -> Option<Self> {
        let guest = PathBuf::from(std::env::var_os("HOME")?).join(".steam");
        let host = running_steam_config_dir()?;
        let same = std::fs::canonicalize(&host).ok() == std::fs::canonicalize(&guest).ok();
        (!same).then_some(Self { host, guest })
    }
}

impl Layer for SteamApiNativeLayer {
    fn id(&self) -> LayerId {
        LayerId("steamapi-native".to_string())
    }

    fn slot(&self) -> Slot {
        Slot::SteamApi
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::STEAM_HANDSHAKE])
    }

    fn preflight(&self, ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        if ctx.is_dry_run() {
            return Ok(());
        }
        if !likely_steamclient_locations().iter().any(|p| p.exists()) {
            return Err(Diagnostic::warning(
                "steamclient.so was not found in any well-known location; the Steam client handshake may fail",
            )
            .with_hint("start the Steam client at least once, or check STEAM_COMPAT_CLIENT_INSTALL_PATH"));
        }
        if !steam_client_is_running() {
            return Err(Diagnostic::warning(
                "the Steam client does not appear to be running; the game's Steam API handshake will likely fail or the game will try to relaunch itself through Steam",
            )
            .with_hint("start the Steam client before launching this game"));
        }
        Ok(())
    }

    fn container_needs(&self, _ctx: &LaunchCtx) -> Vec<Binding> {
        self.steam_config
            .iter()
            .map(|bind| Binding {
                source: PathValue::Translated {
                    host: bind.host.clone(),
                    guest: bind.guest.clone(),
                },
                mode: BindMode::ReadWrite,
                purpose: BindPurpose("steam-config".to_string()),
            })
            .collect()
    }

    fn patch(&self, spec: &mut CommandSpec, _ctx: &LaunchCtx) -> Result<(), CoreError> {
        crate::set_app_id_env(spec, &self.app_id);
        Ok(())
    }
}

fn likely_steamclient_locations() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = Path::new(&home);
        paths.push(home.join(".steam/steam/linux64/steamclient.so"));
        paths.push(home.join(".steam/steam/ubuntu12_64/steamclient.so"));
    }
    paths.push(PathBuf::from("/usr/lib/steam/steamclient.so"));
    paths
}

/// `STEAMCONFIG`, else `$HOME/.steam`, of the running Steam client.
fn running_steam_config_dir() -> Option<PathBuf> {
    let environ = steam_client_pids()
        .into_iter()
        .find_map(|pid| std::fs::read(format!("/proc/{pid}/environ")).ok())?;
    let var = |name: &str| {
        environ.split(|&b| b == 0).find_map(|entry| {
            let entry = std::str::from_utf8(entry).ok()?;
            entry
                .strip_prefix(name)?
                .strip_prefix('=')
                .map(PathBuf::from)
        })
    };
    var("STEAMCONFIG")
        .or_else(|| var("HOME").map(|h| h.join(".steam")))
        .filter(|dir| dir.is_dir())
}

/// Pids whose exe is `<install>/{steamrt64,ubuntu12_32}/steam`.
fn steam_client_pids() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|pid| pid.bytes().all(|b| b.is_ascii_digit()))
        .filter(|pid| {
            std::fs::read_link(format!("/proc/{pid}/exe"))
                .is_ok_and(|exe| is_steam_client_exe(&exe))
        })
        .collect()
}

pub(crate) fn is_steam_client_exe(exe: &Path) -> bool {
    exe.file_name().is_some_and(|n| n == "steam")
        && exe
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|d| d == "steamrt64" || d == "ubuntu12_32")
}

/// Whether a process named `steam` is running.
fn steam_client_is_running() -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    entries.flatten().any(|entry| {
        let is_pid_dir = entry
            .file_name()
            .to_str()
            .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()));
        is_pid_dir
            && std::fs::read_to_string(entry.path().join("comm"))
                .is_ok_and(|comm| comm.trim() == "steam")
    })
}
