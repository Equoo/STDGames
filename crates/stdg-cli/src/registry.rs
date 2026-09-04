//! Wires every runner/layer implementation crate into one `Registry`. This
//! is the one place in the workspace that links all of them together.

use std::path::PathBuf;

use stdg_core::{CoreError, Layer, LayerId, LayerRef, ResolvedConfig};
use stdg_layers_compat::{ProtonLayer, WineLayer};
use stdg_layers_runtime::{PressureVesselLayer, PressureVesselVariant};
use stdg_layers_sandbox::{BwrapLayer, SandboxProfile};
use stdg_layers_steamapi::{EmuVariant, SteamApiEmuLayer, SteamApiNativeLayer};
use stdg_layers_supervision::SupervisionLayer;
use stdg_registry::Registry;
use stdg_runners_emulator::{EmulatorManifest, EmulatorRunner};
use stdg_runners_native::NativeLinuxRunner;
use stdg_runners_windows::WindowsRunner;

pub fn build_registry() -> Registry {
    let mut registry = Registry::new();

    registry.register_runner(Box::new(NativeLinuxRunner));
    registry.register_runner(Box::new(WindowsRunner));
    registry.register_runner(Box::new(EmulatorRunner::new(load_emulator_manifests())));

    registry.register_layer(
        LayerId("bwrap".to_string()),
        Box::new(|r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            let image_root = r.param("image_root").map(PathBuf::from);
            let profile = match r.param("profile") {
                Some("super-compat") => SandboxProfile::SuperCompat,
                _ => SandboxProfile::Normal,
            };
            Ok(Box::new(BwrapLayer { profile, image_root }))
        }),
    );

    registry.register_layer(
        LayerId("soldier".to_string()),
        Box::new(|r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(PressureVesselLayer {
                variant: PressureVesselVariant::Soldier,
                depot_path: r.param("depot_path").map(PathBuf::from).unwrap_or_default(),
            }))
        }),
    );
    registry.register_layer(
        LayerId("sniper".to_string()),
        Box::new(|r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(PressureVesselLayer {
                variant: PressureVesselVariant::Sniper,
                depot_path: r.param("depot_path").map(PathBuf::from).unwrap_or_default(),
            }))
        }),
    );

    registry.register_layer(
        LayerId("proton".to_string()),
        Box::new(|r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(ProtonLayer {
                version: r.param("version").unwrap_or("default").to_string(),
                proton_path: PathBuf::from(r.param("proton_path").unwrap_or_default()),
                prefix_path: PathBuf::from(r.param("prefix_path").unwrap_or_default()),
                steam_client_path: r.param("steam_client_path").map(PathBuf::from),
            }))
        }),
    );
    registry.register_layer(
        LayerId("wine".to_string()),
        Box::new(|r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(WineLayer {
                version: r.param("version").unwrap_or("default").to_string(),
                wine_path: PathBuf::from(r.param("wine_path").unwrap_or_default()),
                prefix_path: PathBuf::from(r.param("prefix_path").unwrap_or_default()),
                steam_client_path: r.param("steam_client_path").map(PathBuf::from),
            }))
        }),
    );

    registry.register_layer(
        LayerId("steamapi-native".to_string()),
        Box::new(|r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(SteamApiNativeLayer {
                app_id: r.param("app_id").unwrap_or("0").to_string(),
            }))
        }),
    );
    registry.register_layer(
        LayerId("steamapi-emu".to_string()),
        Box::new(|_r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(SteamApiEmuLayer {
                variant: EmuVariant::PlainReplace,
            }))
        }),
    );
    registry.register_layer(
        LayerId("steamapi-emu-over-native".to_string()),
        Box::new(|_r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(SteamApiEmuLayer {
                variant: EmuVariant::OverNative,
            }))
        }),
    );

    registry.register_layer(
        LayerId("cgroup".to_string()),
        Box::new(|_r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
            Ok(Box::new(SupervisionLayer::new()))
        }),
    );

    registry
}

/// Emulators are entirely config-driven: every `*.toml` manifest under
/// `./emulators` becomes an available platform, no code change needed.
fn load_emulator_manifests() -> Vec<EmulatorManifest> {
    let dir = PathBuf::from("emulators");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut manifests = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        match std::fs::read_to_string(&path).ok().and_then(|s| EmulatorManifest::from_toml_str(&s).ok()) {
            Some(manifest) => manifests.push(manifest),
            None => eprintln!("warning: failed to load emulator manifest {}", path.display()),
        }
    }
    manifests
}
