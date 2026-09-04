//! Test-only catalog and toy layers/runners. None of this links a real
//! runner or layer crate: it exists to exercise `stdg-plan` in isolation,
//! proving the planner is testable without anything that could actually
//! launch a process.

use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    CapabilitySet, CommandSpec, CoreError, GameId, LaunchCtx, Layer, LayerCatalog, LayerId,
    LayerRef, ModeId, PartialGameConfig, PartialModeConfig, PathValue, ResolvedConfig, Runner,
    RunnerCatalog, RunnerId, Slot, TargetKind,
};

pub struct ToyLayer {
    pub id: &'static str,
    pub slot: Slot,
    pub provides: CapabilitySet,
    pub requires: CapabilitySet,
}

impl Layer for ToyLayer {
    fn id(&self) -> LayerId {
        LayerId(self.id.to_string())
    }

    fn slot(&self) -> Slot {
        self.slot
    }

    fn provides(&self) -> CapabilitySet {
        self.provides.clone()
    }

    fn requires(&self) -> CapabilitySet {
        self.requires.clone()
    }
}

pub struct ToyRunner {
    pub id: &'static str,
    pub target: TargetKind,
}

impl Runner for ToyRunner {
    fn id(&self) -> RunnerId {
        RunnerId(self.id.to_string())
    }

    fn accepts(&self, target: &TargetKind) -> bool {
        &self.target == target
    }

    fn build(&self, _ctx: &LaunchCtx) -> Result<CommandSpec, CoreError> {
        Ok(CommandSpec::new(PathValue::Host(PathBuf::from("/bin/true"))))
    }
}

pub struct FakeCatalog;

impl LayerCatalog for FakeCatalog {
    fn resolve_layer(&self, r: &LayerRef, _config: &ResolvedConfig) -> Result<Box<dyn Layer>, CoreError> {
        let layer: Box<dyn Layer> = match r.id.0.as_str() {
            "bwrap" => Box::new(ToyLayer {
                id: "bwrap",
                slot: Slot::Sandbox,
                provides: CapabilitySet::of([capabilities::SANDBOXED]),
                requires: CapabilitySet::new(),
            }),
            "soldier" => Box::new(ToyLayer {
                id: "soldier",
                slot: Slot::Runtime,
                provides: CapabilitySet::of([capabilities::SCOUT_LIBS]),
                requires: CapabilitySet::new(),
            }),
            "proton" => Box::new(ToyLayer {
                id: "proton",
                slot: Slot::Compat,
                provides: CapabilitySet::of([capabilities::WINDOWS_ABI]),
                requires: CapabilitySet::new(),
            }),
            "wine" => Box::new(ToyLayer {
                id: "wine",
                slot: Slot::Compat,
                provides: CapabilitySet::of([capabilities::WINDOWS_ABI]),
                requires: CapabilitySet::new(),
            }),
            "steamapi-native" => Box::new(ToyLayer {
                id: "steamapi-native",
                slot: Slot::SteamApi,
                provides: CapabilitySet::of([capabilities::STEAM_HANDSHAKE]),
                requires: CapabilitySet::new(),
            }),
            "steamapi-emu" => Box::new(ToyLayer {
                id: "steamapi-emu",
                slot: Slot::SteamApi,
                provides: CapabilitySet::of([capabilities::STEAM_HANDSHAKE]),
                // The injected steam_api.dll swap only makes sense for a
                // binary running under a Windows ABI (Proton/Wine).
                requires: CapabilitySet::of([capabilities::WINDOWS_ABI]),
            }),
            "cgroup" => Box::new(ToyLayer {
                id: "cgroup",
                slot: Slot::Supervision,
                provides: CapabilitySet::of([capabilities::CGROUP_SUPERVISED]),
                requires: CapabilitySet::new(),
            }),
            other => return Err(CoreError::UnknownLayer(LayerId(other.to_string()))),
        };
        Ok(layer)
    }

    fn known_layer_ids(&self) -> Vec<LayerId> {
        ["bwrap", "soldier", "proton", "wine", "steamapi-native", "steamapi-emu", "cgroup"]
            .into_iter()
            .map(|s| LayerId(s.to_string()))
            .collect()
    }
}

pub struct FakeRunnerCatalog {
    runners: Vec<ToyRunner>,
}

impl FakeRunnerCatalog {
    pub fn new() -> Self {
        Self {
            runners: vec![
                ToyRunner {
                    id: "native-linux",
                    target: TargetKind::NativeLinux,
                },
                ToyRunner {
                    id: "windows",
                    target: TargetKind::Windows,
                },
            ],
        }
    }
}

impl RunnerCatalog for FakeRunnerCatalog {
    fn find_for_target(&self, target: &TargetKind) -> Option<&dyn Runner> {
        self.runners
            .iter()
            .find(|r| r.accepts(target))
            .map(|r| r as &dyn Runner)
    }

    fn resolve_runner(&self, id: &RunnerId) -> Option<&dyn Runner> {
        self.runners
            .iter()
            .find(|r| r.id() == *id)
            .map(|r| r as &dyn Runner)
    }
}

pub fn game_id() -> GameId {
    GameId("toy-game".to_string())
}

pub fn mode_id(name: &str) -> ModeId {
    ModeId(name.to_string())
}

pub fn base_game(target: TargetKind) -> PartialGameConfig {
    PartialGameConfig {
        target: Some(target),
        root: Some(PathBuf::from("/games/toy-game")),
        executable: Some(PathBuf::from("toy-game.bin")),
        args: Some(vec![]),
        command_prefix: None,
        env: None,
        modes: None,
    }
}

pub fn with_mode(mut game: PartialGameConfig, id: &str, mode: PartialModeConfig) -> PartialGameConfig {
    game.modes
        .get_or_insert_with(BTreeMap::new)
        .insert(mode_id(id), mode);
    game
}

pub fn mode_with_layers(layers: &[LayerRef]) -> PartialModeConfig {
    PartialModeConfig {
        enabled: Some(true),
        layers: Some(layers.to_vec()),
        args: None,
        env: None,
    }
}

/// What a real deployment's `defaults/global.toml` always carries: the
/// mandatory Sandbox slot. Tests that aren't specifically exercising the
/// mandatory-slot check itself should build their `Plan`s against this
/// instead of `GlobalDefaults::default()`.
pub fn default_globals() -> crate::cascade::GlobalDefaults {
    crate::cascade::GlobalDefaults {
        baseline_layers: vec![LayerRef::new("bwrap")],
        ..Default::default()
    }
}
