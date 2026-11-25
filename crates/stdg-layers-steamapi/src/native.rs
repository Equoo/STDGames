//! `SteamApi` slot using the real Steam client.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{
    CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx, Layer, LayerId, Slot,
};

/// Lets the game talk to the real Steam client.
#[derive(Debug)]
pub struct SteamApiNativeLayer {
    /// The game's Steam app id, exported as `SteamAppId`/`SteamGameId`.
    pub app_id: String,
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
