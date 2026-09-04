//! SteamApi slot, emulator-DLL variants.
//!
//! STUB. Real implementation needs:
//!   - `prepare()`: build a symlink farm for the replacement
//!     steam_api(64).{so,dll} inside the session's tmp dir (see
//!     `stdg_exec::SessionTmpDir`) and bind-mount it over the game's own
//!     copy — the "never write into the game folder" rule from the design
//!     doc means this cannot be a plain file copy into `ctx.plan.config.root`;
//!   - `container_needs()`: expose that replacement file as a `Binding`
//!     carrying a `PathValue::Translated { host, guest }` pair, so the
//!     Runtime layer can bind it into the pressure-vessel namespace at the
//!     same path the game expects to `dlopen()` its Steam API from;
//!   - picking the actual emulator variant (Goldberg, etc.) and locating
//!     its prebuilt library — no emulator distribution/download is in scope
//!     for this ebauche.
//! Left as a deliberate `todo!()`: the RAII shape is real infrastructure
//! (`SessionTmpDir` already exists in `stdg-exec`), only the emulator-specific
//! wiring is missing.

use stdg_core::capability::capabilities;
use stdg_core::{Binding, CapabilitySet, CoreError, LaunchCtx, Layer, LayerId, SessionGuard, Slot};

/// `PlainReplace` swaps the DLL/SO a Windows binary loads under Proton/Wine
/// — it needs the Windows ABI that only a Compat layer provides.
/// `OverNative` swaps a native Linux game's own `libsteam_api.so` directly
/// and never runs under a Compat layer, so it carries no such requirement.
pub enum EmuVariant {
    PlainReplace,
    OverNative,
}

pub struct SteamApiEmuLayer {
    pub variant: EmuVariant,
}

impl Layer for SteamApiEmuLayer {
    fn id(&self) -> LayerId {
        match self.variant {
            EmuVariant::PlainReplace => LayerId("steamapi-emu".to_string()),
            EmuVariant::OverNative => LayerId("steamapi-emu-over-native".to_string()),
        }
    }

    fn slot(&self) -> Slot {
        Slot::SteamApi
    }

    fn provides(&self) -> CapabilitySet {
        CapabilitySet::of([capabilities::STEAM_HANDSHAKE])
    }

    fn requires(&self) -> CapabilitySet {
        match self.variant {
            EmuVariant::PlainReplace => CapabilitySet::of([capabilities::WINDOWS_ABI]),
            EmuVariant::OverNative => CapabilitySet::new(),
        }
    }

    fn prepare(&self, _ctx: &mut LaunchCtx) -> Result<Box<dyn SessionGuard>, CoreError> {
        todo!("build the replacement steam_api symlink farm under the session tmp dir; see stdg_exec::SessionTmpDir")
    }

    fn container_needs(&self) -> Vec<Binding> {
        todo!("expose the injected library as a Binding with a translated guest path")
    }
}
