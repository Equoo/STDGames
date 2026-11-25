//! `SteamApi` slot, `OnlineFix` variant (work in progress).

use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{CapabilitySet, Diagnostic, LaunchCtx, Layer, LayerId, Slot};

use crate::preflight_replacement_lib;

/// Replaces the game's Steam API library with an `OnlineFix` build.
// TODO(dderny): overlay the OnlineFix source files onto the game root.
#[derive(Debug)]
pub struct SteamApiOnlineFixLayer {
    /// Host path to a prebuilt replacement Steam API library.
    pub replacement_lib_path: PathBuf,
    /// The game's own Steam API library, which this layer shadows.
    pub target_lib_path: PathBuf,
}

impl Layer for SteamApiOnlineFixLayer {
    fn id(&self) -> LayerId {
        LayerId("steamapi-onlinefix".to_string())
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
}
