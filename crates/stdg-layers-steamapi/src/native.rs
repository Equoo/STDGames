//! Real, simple layer: no DLL swap, no injected library — the game's own
//! Steam API resolves normally. All this layer does is set the environment
//! variables the Steam client's handshake looks for and, best-effort, warn
//! if `steamclient.so` is nowhere to be found.

use std::path::{Path, PathBuf};

use stdg_core::capability::capabilities;
use stdg_core::{CapabilitySet, CommandSpec, CoreError, Diagnostic, LaunchCtx, Layer, LayerId, Slot};

pub struct SteamApiNativeLayer {
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
        if ctx.dry_run {
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
        spec.set_env_literal("SteamAppId", self.app_id.clone());
        spec.set_env_literal("SteamGameId", self.app_id.clone());
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

/// Best-effort check for whether the Steam client is currently running,
/// by scanning `/proc` for a process named exactly `steam` — the name
/// Valve's client sets for its main process, as opposed to helpers like
/// `steamwebhelper` or `steamerrorreporter`.
///
/// This is deliberately process-table based rather than tied to a single
/// install layout (native package, Flatpak, snap all differ in `$HOME`
/// layout, but all end up as a `steam`-named process), and it fails closed
/// (reports "not running") if `/proc` can't be read at all, e.g. on a
/// non-Linux host.
fn steam_client_is_running() -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    entries.flatten().any(|entry| {
        let is_pid_dir = entry.file_name().to_str().is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()));
        is_pid_dir
            && std::fs::read_to_string(entry.path().join("comm"))
                .is_ok_and(|comm| comm.trim() == "steam")
    })
}
