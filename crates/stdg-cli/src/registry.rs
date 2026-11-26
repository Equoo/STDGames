//! Wires every runner/layer implementation crate into one `Registry`.

use std::path::{Path, PathBuf};

use stdg_core::{ConfigError, CoreError, Layer, LayerId, LayerRef, ResolvedConfig};
use stdg_layers_compat::{ProtonLayer, WineLayer};
use stdg_layers_runtime::{PressureVesselLayer, PressureVesselVariant};
use stdg_layers_sandbox::{
    ContyLayer, SandboxLevel, USERNS_ENV, UserNs, fusermount_is_setuid_root, parse_userns_flag,
};
use stdg_layers_steamapi::{
    Arch, LibReplace, Overlay, Platform, ReplaceKind, SteamApiNativeLayer, SteamApiReplaceLayer,
    SteamConfigBind, find_steam_api_libs,
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
        let userns = if userns_enabled()? {
            let current = UserNs::current().map_err(|e| ConfigError::InvalidValue {
                field: USERNS_ENV,
                reason: format!("cannot read the current uid and gid: {e}"),
            })?;
            Some(current)
        } else {
            None
        };
        Ok(Box::new(ContyLayer {
            conty_path: r.param("conty_path").map(PathBuf::from),
            sandbox_level,
            base_dir: r.param("base_dir").map(PathBuf::from),
            userns,
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
            steam_stub_dir: steam_stub_dir(r),
        }))
    });
    register(&mut registry, "wine", |r, _| {
        Ok(Box::new(WineLayer {
            version: r.param("version").map(str::to_string),
            wine_path: required_path(r, "wine_path")?,
            prefix_path: required_path(r, "prefix_path")?,
            steam_client_path: r.param("steam_client_path").map(PathBuf::from),
            steam_stub_dir: steam_stub_dir(r),
        }))
    });

    register(&mut registry, "steamapi-native", |r, c| {
        let app_id = app_id(r, c).ok_or_else(|| ConfigError::MissingLayerParam {
            layer: r.id.clone(),
            param: "app_id",
        })?;
        Ok(Box::new(SteamApiNativeLayer {
            app_id,
            steam_config: SteamConfigBind::detect(),
        }))
    });
    register(&mut registry, "steamapi-emu", |r, c| {
        steamapi_replace(r, c, ReplaceKind::Emu)
    });
    register(&mut registry, "steamapi-onlinefix", |r, c| {
        steamapi_replace(r, c, ReplaceKind::OnlineFix)
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

/// Builds `steamapi-emu` (library on by default) or `steamapi-onlinefix` (overlay on by default).
fn steamapi_replace(r: &LayerRef, c: &ResolvedConfig, kind: ReplaceKind) -> LayerResult {
    let replace_lib = bool_param(r, "replace_lib", kind == ReplaceKind::Emu)?;
    let overlay_source = match kind {
        ReplaceKind::OnlineFix => Some(required_path(r, "overlay_dir")?),
        ReplaceKind::Emu => r.param("overlay_dir").map(PathBuf::from),
    };

    let libs = if replace_lib {
        let platform = Platform::of(&c.target).ok_or_else(|| CoreError::IncompatibleTarget {
            layer: r.id.clone(),
            target: c.target.clone(),
            reason: "only Windows and native Linux games have a Steam API library".to_string(),
        })?;
        let targets = match r.param("target_lib_relpath") {
            Some(rel) => vec![c.root.join(rel)],
            None => find_steam_api_libs(&c.root, platform),
        };
        targets
            .into_iter()
            .map(|target| {
                let arch = Arch::detect(&target);
                LibReplace {
                    replacement: arch.and_then(|a| r.param(a.param())).map(PathBuf::from),
                    arch,
                    target,
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let overlay = match overlay_source {
        Some(source) => Some(Overlay {
            source,
            state_dir: match r.param("overlay_state_dir") {
                Some(dir) => PathBuf::from(dir),
                None => default_overlay_state_dir(r, c)?,
            },
        }),
        None => None,
    };

    Ok(Box::new(SteamApiReplaceLayer {
        kind,
        game_root: c.root.clone(),
        replace_lib,
        libs,
        overlay,
        app_id: app_id(r, c),
    }))
}

/// The layer's `app_id` param, else the game's.
fn app_id(r: &LayerRef, c: &ResolvedConfig) -> Option<String> {
    r.param("app_id")
        .map(str::to_string)
        .or_else(|| c.app_id.clone())
}

/// `$XDG_DATA_HOME` or `~/.local/share`.
fn data_home() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
}

/// `steam_stub_dir` param, else `<data home>/stdgames/steam-stub`.
fn steam_stub_dir(r: &LayerRef) -> Option<PathBuf> {
    r.param("steam_stub_dir")
        .map(PathBuf::from)
        .or_else(|| data_home().map(|d| d.join("stdgames/steam-stub")))
}

/// `<data home>/stdgames/overlays/<game>/<layer>`.
fn default_overlay_state_dir(r: &LayerRef, c: &ResolvedConfig) -> Result<PathBuf, CoreError> {
    let data_home = data_home().ok_or_else(|| ConfigError::MissingLayerParam {
        layer: r.id.clone(),
        param: "overlay_state_dir",
    })?;
    Ok(data_home
        .join("stdgames/overlays")
        .join(&c.game_id.0)
        .join(&r.id.0))
}

/// `STDG_CONTY_USERNS` when set, else on unless fusermount is setuid root.
fn userns_enabled() -> Result<bool, CoreError> {
    let invalid = |reason: String| ConfigError::InvalidValue {
        field: USERNS_ENV,
        reason,
    };
    match std::env::var(USERNS_ENV) {
        Ok(value) => parse_userns_flag(&value).map_err(|e| invalid(e.to_string()).into()),
        Err(std::env::VarError::NotPresent) => Ok(!fusermount_is_setuid_root()),
        Err(e @ std::env::VarError::NotUnicode(_)) => Err(invalid(e.to_string()).into()),
    }
}

fn bool_param(r: &LayerRef, param: &'static str, default: bool) -> Result<bool, CoreError> {
    match r.param(param) {
        None => Ok(default),
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(other) => Err(invalid_param(
            r,
            param,
            other,
            &"expected `true` or `false`",
        )),
    }
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
            app_id: None,
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
            LayerRef::new("steamapi-emu").with_param("lib_x64", "/r"),
            LayerRef::new("steamapi-onlinefix").with_param("overlay_dir", "/fix"),
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
    fn steamapi_native_falls_back_to_game_app_id() {
        let registry = build_registry();
        let r = LayerRef::new("steamapi-native");
        let mut c = config(Path::new("/nonexistent"));
        assert!(registry.resolve_layer(&r, &c).is_err());
        c.app_id = Some("480".to_string());
        assert!(registry.resolve_layer(&r, &c).is_ok());
    }

    fn resolve(r: &LayerRef, root: &Path) -> Vec<stdg_core::Binding> {
        build_registry()
            .resolve_layer(r, &config(root))
            .expect("resolves")
            .container_needs()
    }

    #[test]
    fn emu_replaces_every_found_library_by_default() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("bin")).expect("mkdir");
        std::fs::write(root.path().join("steam_api64.dll"), b"").expect("write");
        std::fs::write(root.path().join("bin/steam_api.dll"), b"").expect("write");
        let r = LayerRef::new("steamapi-emu")
            .with_param("lib_x64", "/emu/x64.dll")
            .with_param("lib_x86", "/emu/x86.dll");
        let hosts: Vec<_> = resolve(&r, root.path())
            .iter()
            .map(|b| b.source.host().to_path_buf())
            .collect();
        assert_eq!(
            hosts,
            vec![PathBuf::from("/emu/x86.dll"), PathBuf::from("/emu/x64.dll")]
        );
    }

    #[test]
    fn onlinefix_overlays_by_default_and_requires_its_dir() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("steam_api64.dll"), b"").expect("write");
        let r = LayerRef::new("steamapi-onlinefix")
            .with_param("overlay_dir", "/fix")
            .with_param("overlay_state_dir", "/state");
        let needs = resolve(&r, root.path());
        assert_eq!(needs.len(), 1, "no library replacement by default");
        assert!(matches!(needs[0].mode, stdg_core::BindMode::Overlay { .. }));

        let err = build_registry()
            .resolve_layer(&LayerRef::new("steamapi-onlinefix"), &config(root.path()))
            .map(|_| ())
            .expect_err("overlay_dir is required");
        assert!(matches!(
            err,
            CoreError::Config(ConfigError::MissingLayerParam {
                param: "overlay_dir",
                ..
            })
        ));
    }

    #[test]
    fn options_can_be_flipped() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("steam_api64.dll"), b"").expect("write");
        let emu = LayerRef::new("steamapi-emu")
            .with_param("replace_lib", "false")
            .with_param("overlay_dir", "/fix")
            .with_param("overlay_state_dir", "/state");
        assert_eq!(resolve(&emu, root.path()).len(), 1);
        let bad = LayerRef::new("steamapi-emu").with_param("replace_lib", "yes");
        assert!(
            build_registry()
                .resolve_layer(&bad, &config(root.path()))
                .is_err()
        );
    }

    #[test]
    fn explicit_target_relpath_wins_over_search() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("steam_api64.dll"), b"").expect("write");
        let r = LayerRef::new("steamapi-emu")
            .with_param("target_lib_relpath", "bin/steam_api64.dll")
            .with_param("lib_x64", "/emu/x64.dll");
        let needs = resolve(&r, root.path());
        assert_eq!(
            needs[0].source.effective(),
            root.path().join("bin/steam_api64.dll")
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
