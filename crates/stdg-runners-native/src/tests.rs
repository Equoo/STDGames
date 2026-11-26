use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::{
    ArgValue, GameId, LaunchCtx, ModeId, PathValue, Plan, ResolvedConfig, RunKind, Runner,
    RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::NativeLinuxRunner;

fn ctx(command_prefix: Vec<String>) -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("g".to_string()),
        target: TargetKind::NativeLinux,
        root: PathBuf::from("/games/g"),
        executable: PathBuf::from("bin/g"),
        args: vec!["--fullscreen".to_string()],
        command_prefix,
        env: BTreeMap::from([("K".to_string(), "V".to_string())]),
        saves: stdg_core::SavesConfig::default(),
    };
    LaunchCtx {
        plan: Plan {
            game_id: config.game_id.clone(),
            mode_id: ModeId("m".to_string()),
            target: config.target.clone(),
            runner: RunnerId("native-linux".to_string()),
            slots: BTreeMap::new(),
            config,
        },
        session: SessionInfo {
            id: SessionId("s".to_string()),
            tmp_dir: PathBuf::from("/nonexistent"),
        },
        bindings: Vec::new(),
        run_kind: RunKind::DryRun,
        wine_prefix: None,
    }
}

fn args(spec: &stdg_core::CommandSpec) -> Vec<String> {
    spec.args.iter().map(ArgValue::render).collect()
}

#[test]
fn accepts_only_native_linux_targets() {
    assert!(NativeLinuxRunner.accepts(&TargetKind::NativeLinux));
    assert!(!NativeLinuxRunner.accepts(&TargetKind::Windows));
}

#[test]
fn builds_the_executable_from_the_game_root() {
    let spec = NativeLinuxRunner.build(&ctx(vec![])).expect("build");
    assert_eq!(
        spec.program,
        Some(PathValue::Host(PathBuf::from("/games/g/bin/g")))
    );
    assert_eq!(args(&spec), vec!["--fullscreen"]);
    assert_eq!(spec.cwd, Some(PathValue::Host(PathBuf::from("/games/g"))));
    assert!(spec.env.contains_key("K"));
}

#[test]
fn command_prefix_becomes_the_program() {
    let prefix = vec!["gamemoderun".to_string(), "-x".to_string()];
    let spec = NativeLinuxRunner.build(&ctx(prefix)).expect("build");
    assert_eq!(
        spec.program,
        Some(PathValue::Host(PathBuf::from("gamemoderun")))
    );
    assert_eq!(args(&spec), vec!["-x", "/games/g/bin/g", "--fullscreen"]);
}
