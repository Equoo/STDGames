//! `SteamApi` slot: native Steam client, emulator library, or online fix.

mod emu;
mod native;
mod onlinefix;

use std::path::Path;

use stdg_core::Diagnostic;

pub use emu::{EmuVariant, SteamApiEmuLayer};
pub use native::SteamApiNativeLayer;
pub use onlinefix::SteamApiOnlineFixLayer;

/// Fails if the replacement library is missing.
fn preflight_replacement_lib(replacement_lib_path: &Path) -> Result<(), Diagnostic> {
    if replacement_lib_path.is_file() {
        return Ok(());
    }
    Err(Diagnostic::error(format!(
        "replacement Steam API library not found: {} does not exist",
        replacement_lib_path.display()
    ))
    .with_hint("point `replacement_lib_path` at a prebuilt Steam API library"))
}

#[cfg(test)]
mod tests;
