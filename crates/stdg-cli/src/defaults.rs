//! Loads the two lower cascade tiers (see `stdg_plan::cascade`) from disk:
//! `defaults/global.toml` and `defaults/runners/<runner-id>.toml`. Missing
//! files are not an error — they just mean that tier contributes nothing,
//! so a runner without its own defaults file behaves exactly as if it had
//! an empty one.

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use stdg_core::TargetKind;
use stdg_plan::{GlobalDefaults, RunnerDefaults};

pub fn load_global_defaults() -> GlobalDefaults {
    load_toml_or_default(&PathBuf::from("defaults/global.toml"))
}

/// The runner a target resolves to is only known once `build_plan` looks it
/// up in the registry, but the *filename* to load defaults from only needs
/// the target kind, so this mirrors each runner's `accepts()` without
/// instantiating one.
pub fn load_runner_defaults(target: &TargetKind) -> RunnerDefaults {
    let runner_id = match target {
        TargetKind::NativeLinux => "native-linux",
        TargetKind::Windows => "windows",
        TargetKind::Rom(_) => "emulator",
    };
    load_toml_or_default(&PathBuf::from("defaults/runners").join(format!("{runner_id}.toml")))
}

fn load_toml_or_default<T: Default + DeserializeOwned>(path: &PathBuf) -> T {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return T::default();
    };
    match toml::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("warning: failed to parse {}: {e}", path.display());
            T::default()
        }
    }
}
