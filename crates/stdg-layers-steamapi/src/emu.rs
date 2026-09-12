//! SteamApi slot, emulator-DLL variants.
//!
//! The mechanism is a single container-boundary trick, shared by both
//! variants: bind-mount a replacement Steam API library (a prebuilt
//! emulator such as Goldberg SteamEmu — building or fetching one is out of
//! scope here, same "point this layer at an already-installed build, no
//! discovery, no download" contract as `stdg-layers-compat`'s
//! `proton_path`) directly over the absolute path where the game expects
//! to find its own copy.
//!
//! That target path is a plain host filesystem path, not a
//! container-namespace path needing its own translation: whether the game
//! is native or running under Proton/Wine, its Steam API library sits at a
//! normal path under the install root, and `stdg-layers-sandbox::ContyLayer`
//! binds that root at the same path on both sides of the container
//! boundary. So `PathValue::Translated { host: <replacement>, guest: <the
//! game's real path> }` is enough for Conty to overlay the file — no
//! separate rename/symlink step is needed first, since a bind mount does
//! not require its source and destination to share a basename.
//!
//! This also means no scratch space is needed in `prepare()` (contrast
//! `stdg-layers-compat`, which does create/own a persistent Wine prefix
//! directory): both fields below are plain, already-resolved paths known
//! at construction time, so `container_needs()` — which is pure by
//! contract (see `Layer::container_needs`'s doc) — can derive the binding
//! straight from `self` with no session-scoped state to smuggle in.
//!
//! Left out of scope for this ebauche: picking the actual emulator variant
//! (Goldberg, etc.) and locating its prebuilt library — the caller supplies
//! `replacement_lib_path` explicitly, same as every other "already
//! installed" path parameter in this workspace.

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, Diagnostic, LaunchCtx, Layer, LayerId,
    PathValue, Slot,
};

/// `PlainReplace` swaps the DLL/SO a Windows binary loads under Proton/Wine
/// — it needs the Windows ABI that only a Compat layer provides.
/// `OverNative` swaps a native Linux game's own `libsteam_api.so` directly
/// and never runs under a Compat layer, so it carries no such requirement.
pub enum EmuVariant {
    PlainReplace,
}

pub struct SteamApiEmuLayer {
    pub variant: EmuVariant,
    /// Host path to an already-built replacement Steam API library (e.g. a
    /// Goldberg SteamEmu `steam_api64.dll` or `libsteam_api.so`). Checked
    /// to exist in `preflight`; never downloaded or built by this layer.
    pub replacement_lib_path: PathBuf,
    /// Absolute, on-disk path to the file this layer shadows: the game's
    /// own copy of the Steam API library, at the path it will actually
    /// `dlopen()`/`LoadLibrary()` from.
    pub target_lib_path: PathBuf,
}

impl Layer for SteamApiEmuLayer {
    fn id(&self) -> LayerId {
        match self.variant {
            EmuVariant::PlainReplace => LayerId("steamapi-emu".to_string()),
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
            EmuVariant::PlainReplace => CapabilitySet::new(),
        }
    }

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        if !self.replacement_lib_path.is_file() {
            return Err(Diagnostic::error(format!(
                "replacement Steam API library not found: {} does not exist",
                self.replacement_lib_path.display()
            ))
            .with_hint("point `replacement_lib_path` at an already-built Steam API emulator library (e.g. Goldberg SteamEmu)"));
        }
        Ok(())
    }

    fn container_needs(&self) -> Vec<Binding> {
        vec![Binding {
            source: PathValue::Translated {
                host: self.replacement_lib_path.clone(),
                guest: self.target_lib_path.clone(),
            },
            mode: BindMode::ReadOnly,
            purpose: BindPurpose(format!("{}-dll", self.id().0)),
        }]
    }
}
