//! `SteamApi` slot: native Steam client, emulator library, or online-fix overlay.

mod native;
mod replace;

use std::path::Path;

use stdg_core::Diagnostic;

pub use native::SteamApiNativeLayer;
pub use replace::{
    Arch, LibReplace, Overlay, Platform, ReplaceKind, SteamApiReplaceLayer, find_steam_api_libs,
};

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
