//! Wires every runner/layer implementation crate into one `Registry`.

use std::path::{Path, PathBuf};

use stdg_core::{ConfigError, CoreError, Layer, LayerId, LayerRef, ResolvedConfig};
use stdg_layers_compat::{ProtonLayer, WineLayer};
use stdg_layers_runtime::{PressureVesselLayer, PressureVesselVariant};
use stdg_layers_sandbox::{ContyLayer, SandboxLevel};
use stdg_layers_steamapi::{
    EmuVariant, SteamApiEmuLayer, SteamApiNativeLayer, SteamApiOnlineFixLayer,
};
use stdg_layers_supervision::SupervisionLayer;
use stdg_registry::Registry;
use stdg_runners_emulator::{EmulatorManifest, EmulatorRunner};
use stdg_runners_native::NativeLinuxRunner;
use stdg_runners_windows::WindowsRunner;

type LayerResult = Result<Box<dyn Layer>, CoreError>;

pub(crate) fn build_registry() -> Registry {
    let mut registry = Registry::new();

    registry.register_runner(Box::new(NativeLinuxRunner));
    registry.register_runner(Box::new(WindowsRunner));
    registry.register_runner(Box::new(EmulatorRunner::new(load_emulator_manifests(
        Path::new("emulators"),
    ))));

    register(&mut registry, "conty", |r, _| {
        let sandbox_level = r
            .param("sandbox_level")
            .map(|s| {
                s.parse::<SandboxLevel>()
                    .map_err(|e| invalid_param(r, "sandbox_level", s, &e))
            })
            .transpose()?;
        Ok(Box::new(ContyLayer {
            conty_path: r.param("conty_path").map(PathBuf::from),
            sandbox_level,
            base_dir: r.param("base_dir").map(PathBuf::from),
        }))
    });

    register(&mut registry, "soldier", |r, _| {
        Ok(Box::new(PressureVesselLayer {
            variant: PressureVesselVariant::Soldier,
            depot_path: required_path(r, "depot_path")?,
        }))
    });
    register(&mut registry, "sniper", |r, _| {
        Ok(Box::new(PressureVesselLayer {
            variant: PressureVesselVariant::Sniper,
            depot_path: required_path(r, "depot_path")?,
        }))
    });

    register(&mut registry, "proton", |r, _| {
        Ok(Box::new(ProtonLayer {
            version: r.param("version").map(str::to_string),
            proton_path: required_path(r, "proton_path")?,
            prefix_path: required_path(r, "prefix_path")?,
            steam_client_path: r.param("steam_client_path").map(PathBuf::from),
        }))
    });
    register(&mut registry, "wine", |r, _| {
        Ok(Box::new(WineLayer {
            version: r.param("version").map(str::to_string),
            wine_path: required_path(r, "wine_path")?,
            prefix_path: required_path(r, "prefix_path")?,
            steam_client_path: r.param("steam_client_path").map(PathBuf::from),
        }))
    });

    register(&mut registry, "steamapi-native", |r, _| {
        Ok(Box::new(SteamApiNativeLayer {
            app_id: required(r, "app_id")?.to_string(),
        }))
    });
    register(&mut registry, "steamapi-emu", |r, c| {
        Ok(Box::new(SteamApiEmuLayer {
            variant: EmuVariant::PlainReplace,
            replacement_lib_path: required_path(r, "replacement_lib_path")?,
            target_lib_path: emu_target_lib_path(r, c, "steam_api64.dll"),
        }))
    });
    register(&mut registry, "steamapi-onlinefix", |r, c| {
        Ok(Box::new(SteamApiOnlineFixLayer {
            replacement_lib_path: required_path(r, "replacement_lib_path")?,
            target_lib_path: emu_target_lib_path(r, c, "steam_api64.dll"),
        }))
    });

    register(&mut registry, "cgroup", |_, _| {
        Ok(Box::new(SupervisionLayer::new()))
    });

    registry
}

fn register(
    registry: &mut Registry,
    id: &str,
    factory: impl Fn(&LayerRef, &ResolvedConfig) -> LayerResult + Send + Sync + 'static,
) {
    registry.register_layer(LayerId(id.to_string()), Box::new(factory));
}

fn required<'r>(r: &'r LayerRef, param: &'static str) -> Result<&'r str, CoreError> {
    r.param(param).ok_or_else(|| {
        ConfigError::MissingLayerParam {
            layer: r.id.clone(),
            param,
        }
        .into()
    })
}

fn required_path(r: &LayerRef, param: &'static str) -> Result<PathBuf, CoreError> {
    required(r, param).map(PathBuf::from)
}

fn invalid_param(
    r: &LayerRef,
    param: &'static str,
    value: &str,
    reason: &dyn std::fmt::Display,
) -> CoreError {
    ConfigError::InvalidLayerParam {
        layer: r.id.clone(),
        param,
        value: value.to_string(),
        reason: reason.to_string(),
    }
    .into()
}

/// Depth-first search for a file called `name` under `dir`.
fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::debug!("skipping {}: {e}", dir.display());
            return None;
        }
    };

    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            subdirs.push(entry.path());
        } else if entry.file_name() == name {
            return Some(entry.path());
        }
    }

    subdirs.iter().find_map(|d| find_file(d, name))
}

/// Path the replacement library is mounted over.
fn emu_target_lib_path(r: &LayerRef, c: &ResolvedConfig, default_name: &str) -> PathBuf {
    if let Some(relpath) = r.param("target_lib_relpath") {
        return c.root.join(relpath);
    }
    find_file(&c.root, default_name).unwrap_or_else(|| c.root.join(default_name))
}

/// Loads every `*.toml` emulator manifest in `dir`.
fn load_emulator_manifests(dir: &Path) -> Vec<EmulatorManifest> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            tracing::warn!("cannot read emulator manifests in {}: {e}", dir.display());
            return Vec::new();
        }
    };

    let mut manifests = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let loaded = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|s| EmulatorManifest::from_toml_str(&s).map_err(|e| e.to_string()));
        match loaded {
            Ok(manifest) => manifests.push(manifest),
            Err(e) => tracing::warn!("skipping emulator manifest {}: {e}", path.display()),
        }
    }
    manifests
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use stdg_core::{GameId, LayerCatalog, TargetKind};

    use super::*;

    fn config(root: &Path) -> ResolvedConfig {
        ResolvedConfig {
            game_id: GameId("g".to_string()),
            target: TargetKind::Windows,
            root: root.to_path_buf(),
            executable: PathBuf::from("g.exe"),
            args: vec![],
            command_prefix: vec![],
            env: BTreeMap::new(),
        }
    }

    #[test]
    fn missing_required_param_is_a_config_error() {
        let err = build_registry()
            .resolve_layer(&LayerRef::new("proton"), &config(Path::new("/g")))
            .map(|_| ())
            .expect_err("proton_path is required");
        assert!(matches!(
            err,
            CoreError::Config(ConfigError::MissingLayerParam {
                param: "proton_path",
                ..
            })
        ));
    }

    #[test]
    fn invalid_sandbox_level_is_rejected_at_load_time() {
        let r = LayerRef::new("conty").with_param("sandbox_level", "7");
        let err = build_registry()
            .resolve_layer(&r, &config(Path::new("/g")))
            .map(|_| ())
            .expect_err("7 is not a Conty level");
        assert!(matches!(
            err,
            CoreError::Config(ConfigError::InvalidLayerParam {
                param: "sandbox_level",
                ..
            })
        ));
    }

    #[test]
    fn every_layer_with_its_params_resolves() {
        let registry = build_registry();
        let refs = [
            LayerRef::new("conty").with_param("sandbox_level", "2"),
            LayerRef::new("soldier").with_param("depot_path", "/d"),
            LayerRef::new("sniper").with_param("depot_path", "/d"),
            LayerRef::new("proton")
                .with_param("proton_path", "/p")
                .with_param("prefix_path", "/x"),
            LayerRef::new("wine")
                .with_param("wine_path", "/w")
                .with_param("prefix_path", "/x"),
            LayerRef::new("steamapi-native").with_param("app_id", "480"),
            LayerRef::new("steamapi-emu").with_param("replacement_lib_path", "/r"),
            LayerRef::new("steamapi-onlinefix").with_param("replacement_lib_path", "/r"),
            LayerRef::new("cgroup"),
        ];
        for r in refs {
            let layer = registry
                .resolve_layer(&r, &config(Path::new("/nonexistent")))
                .unwrap_or_else(|e| panic!("{} should resolve: {e}", r.id));
            assert_eq!(layer.id(), r.id);
        }
    }

    #[test]
    fn find_file_searches_subdirectories() {
        let root = tempfile::tempdir().expect("tempdir");
        let nested = root.path().join("a/b");
        std::fs::create_dir_all(&nested).expect("mkdir");
        std::fs::write(nested.join("steam_api64.dll"), b"").expect("write");
        assert_eq!(
            find_file(root.path(), "steam_api64.dll"),
            Some(nested.join("steam_api64.dll"))
        );
        assert_eq!(find_file(root.path(), "absent.dll"), None);
    }

    #[test]
    fn explicit_target_relpath_wins_over_search() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("steam_api64.dll"), b"").expect("write");
        let r = LayerRef::new("steamapi-emu").with_param("target_lib_relpath", "bin/custom.dll");
        assert_eq!(
            emu_target_lib_path(&r, &config(root.path()), "steam_api64.dll"),
            root.path().join("bin/custom.dll")
        );
        let r = LayerRef::new("steamapi-emu");
        assert_eq!(
            emu_target_lib_path(&r, &config(root.path()), "steam_api64.dll"),
            root.path().join("steam_api64.dll")
        );
    }

    #[test]
    fn emulator_manifests_skip_invalid_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("ok.toml"),
            "id = \"m\"\nplatforms = [\"gba\"]\nbinary = \"/bin/m\"\nargs = [\"{rom}\"]\n",
        )
        .expect("write");
        std::fs::write(dir.path().join("bad.toml"), "id = 1").expect("write");
        std::fs::write(dir.path().join("notes.txt"), "ignored").expect("write");
        let manifests = load_emulator_manifests(dir.path());
        assert_eq!(manifests.len(), 1);
        assert!(load_emulator_manifests(&dir.path().join("absent")).is_empty());
    }
}
