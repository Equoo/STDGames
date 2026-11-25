//! `SteamApi` slot: bind-mounts a prebuilt emulator library over the game's own copy.

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, BindPurpose, Binding, CapabilitySet, Diagnostic, LaunchCtx, Layer, LayerId,
    PathValue, Slot,
};

use crate::preflight_replacement_lib;

/// How the emulator library replaces the game's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuVariant {
    /// Bind-mount the replacement directly over the game's library.
    PlainReplace,
}

/// Replaces the game's Steam API library with a prebuilt emulator.
#[derive(Debug)]
pub struct SteamApiEmuLayer {
    /// How the replacement is applied.
    pub variant: EmuVariant,
    /// Host path to a prebuilt replacement Steam API library.
    pub replacement_lib_path: PathBuf,
    /// The game's own Steam API library, which this layer shadows.
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

    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        preflight_replacement_lib(&self.replacement_lib_path)
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
