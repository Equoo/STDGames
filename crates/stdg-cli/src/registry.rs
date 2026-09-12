//! Wires every runner/layer implementation crate into one `Registry`. This
//! is the one place in the workspace that links all of them together.

use std::{
    fs,
    path::{Path, PathBuf},
};

use stdg_core::{CoreError, Layer, LayerId, LayerRef, ResolvedConfig};
use stdg_layers_compat::{ProtonLayer, WineLayer};
use stdg_layers_runtime::{PressureVesselLayer, PressureVesselVariant};
use stdg_layers_sandbox::ContyLayer;
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
        LayerId("conty".to_string()),
        Box::new(
            |r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(ContyLayer {
                    conty_path: r.param("conty_path").map(PathBuf::from),
                    sandbox_level: r.param("sandbox_level").and_then(|s| s.parse().ok()),
                    base_dir: r.param("base_dir").map(PathBuf::from),
                }))
            },
        ),
    );

    registry.register_layer(
        LayerId("soldier".to_string()),
        Box::new(
            |r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(PressureVesselLayer {
                    variant: PressureVesselVariant::Soldier,
                    depot_path: r.param("depot_path").map(PathBuf::from).unwrap_or_default(),
                }))
            },
        ),
    );
    registry.register_layer(
        LayerId("sniper".to_string()),
        Box::new(
            |r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(PressureVesselLayer {
                    variant: PressureVesselVariant::Sniper,
                    depot_path: r.param("depot_path").map(PathBuf::from).unwrap_or_default(),
                }))
            },
        ),
    );

    registry.register_layer(
        LayerId("proton".to_string()),
        Box::new(
            |r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(ProtonLayer {
                    version: r.param("version").unwrap_or("default").to_string(),
                    proton_path: PathBuf::from(r.param("proton_path").unwrap_or_default()),
                    prefix_path: PathBuf::from(r.param("prefix_path").unwrap_or_default()),
                    steam_client_path: r.param("steam_client_path").map(PathBuf::from),
                }))
            },
        ),
    );
    registry.register_layer(
        LayerId("wine".to_string()),
        Box::new(
            |r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(WineLayer {
                    version: r.param("version").unwrap_or("default").to_string(),
                    wine_path: PathBuf::from(r.param("wine_path").unwrap_or_default()),
                    prefix_path: PathBuf::from(r.param("prefix_path").unwrap_or_default()),
                    steam_client_path: r.param("steam_client_path").map(PathBuf::from),
                }))
            },
        ),
    );

    registry.register_layer(
        LayerId("steamapi-native".to_string()),
        Box::new(
            |r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(SteamApiNativeLayer {
                    app_id: r.param("app_id").unwrap_or("0").to_string(),
                }))
            },
        ),
    );
    registry.register_layer(
        LayerId("steamapi-emu".to_string()),
        Box::new(
            |r: &LayerRef, c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(SteamApiEmuLayer {
                    variant: EmuVariant::PlainReplace,
                    replacement_lib_path: PathBuf::from(
                        r.param("replacement_lib_path").unwrap_or_default(),
                    ),
                    target_lib_path: emu_target_lib_path(r, c, "steam_api64.dll"),
                }))
            },
        ),
    );

    registry.register_layer(
        LayerId("cgroup".to_string()),
        Box::new(
            |_r: &LayerRef, _c: &ResolvedConfig| -> Result<Box<dyn Layer>, CoreError> {
                Ok(Box::new(SupervisionLayer::new()))
            },
        ),
    );

    registry
}

pub fn find_file(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    let mut subdirs = Vec::new();

    for entry in fs::read_dir(dir).ok()?.flatten() {
        let file_type = entry.file_type().ok()?;
        if file_type.is_dir() {
            subdirs.push(entry.path());
        } else if entry.file_name() == name {
            return Some(entry.path());
        }
    }

    subdirs.iter().find_map(|d| find_file(d, name))
}

/// Where a `steamapi-emu*` layer's replacement library gets bind-mounted:
/// the game's own copy of the Steam API library, at its real on-disk path.
/// `target_lib_relpath` overrides the filename for a game that ships a
/// differently-named copy; most don't need to set it.
fn emu_target_lib_path(r: &LayerRef, c: &ResolvedConfig, default_name: &str) -> PathBuf {
    let relpath = r.param("target_lib_relpath").unwrap_or(default_name);
    let default = c.root.join(relpath);
    find_file(&c.root, default_name).unwrap_or(default)
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
        match std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| EmulatorManifest::from_toml_str(&s).ok())
        {
            Some(manifest) => manifests.push(manifest),
            None => eprintln!(
                "warning: failed to load emulator manifest {}",
                path.display()
            ),
        }
    }
    manifests
}
