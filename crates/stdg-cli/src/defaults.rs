//! Loads `defaults/global.toml` and `defaults/runners/<runner>.toml`.

use std::io;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use serde::de::DeserializeOwned;
use stdg_core::TargetKind;
use stdg_plan::{GlobalDefaults, RunnerDefaults};

pub(crate) fn load_global_defaults() -> anyhow::Result<GlobalDefaults> {
    load_toml_or_default(Path::new("defaults/global.toml"))
}

/// Defaults file for the runner that handles `target`.
pub(crate) fn load_runner_defaults(target: &TargetKind) -> anyhow::Result<RunnerDefaults> {
    load_toml_or_default(&runner_defaults_path(target))
}

fn runner_defaults_path(target: &TargetKind) -> PathBuf {
    let runner_id = match target {
        TargetKind::NativeLinux => "native-linux",
        TargetKind::Windows => "windows",
        TargetKind::Rom(_) => "emulator",
    };
    PathBuf::from("defaults/runners").join(format!("{runner_id}.toml"))
}

fn load_toml_or_default<T: Default + DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(T::default()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_yields_the_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let loaded: GlobalDefaults =
            load_toml_or_default(&dir.path().join("absent.toml")).expect("missing is fine");
        assert_eq!(loaded.baseline_layers.len(), 0);
    }

    #[test]
    fn a_malformed_file_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "baseline_layers = 3").expect("write");
        let err = load_toml_or_default::<GlobalDefaults>(&path).expect_err("malformed");
        assert!(format!("{err:#}").contains("parsing"));
    }

    #[test]
    fn a_valid_file_is_parsed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ok.toml");
        std::fs::write(&path, "baseline_layers = [{ id = \"cgroup\" }]").expect("write");
        let loaded: GlobalDefaults = load_toml_or_default(&path).expect("valid");
        assert_eq!(loaded.baseline_layers.len(), 1);
    }

    #[test]
    fn runner_defaults_file_follows_the_target_kind() {
        assert!(runner_defaults_path(&TargetKind::Windows).ends_with("windows.toml"));
        assert!(runner_defaults_path(&TargetKind::NativeLinux).ends_with("native-linux.toml"));
    }
}
