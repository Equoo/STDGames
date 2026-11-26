//! `SteamApi` slot: native Steam client, emulator library, or online-fix overlay.

mod native;
mod replace;

use std::path::Path;

use stdg_core::{CommandSpec, Diagnostic};

pub use native::{SteamApiNativeLayer, SteamConfigBind};
pub use replace::{
    Arch, LibReplace, Platform, ReplaceKind, SteamApiReplaceLayer, find_steam_api_libs,
};

/// Exports `SteamAppId`/`SteamGameId`, read by Steam, emulators and Proton.
fn set_app_id_env(spec: &mut CommandSpec, app_id: &str) {
    spec.set_env_literal("SteamAppId", app_id);
    spec.set_env_literal("SteamGameId", app_id);
}

/// Fails if the replacement library is missing.
fn preflight_replacement_lib(replacement_lib_path: &Path) -> Result<(), Diagnostic> {
    if replacement_lib_path.is_file() {
        return Ok(());
    }
    Err(Diagnostic::error(format!(
        "replacement Steam API library not found: {} does not exist",
        replacement_lib_path.display()
    ))
    .with_hint("point `lib_x64`/`lib_x86` at a prebuilt Steam API library"))
}

#[cfg(test)]
mod tests;
