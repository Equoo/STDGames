use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::{
    ArgValue, GameId, LaunchCtx, ModeId, PathValue, Plan, ResolvedConfig, RunKind, Runner,
    RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::WindowsRunner;

fn ctx() -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("g".to_string()),
        target: TargetKind::Windows,
        root: PathBuf::from("/games/g"),
        executable: PathBuf::from("G.exe"),
        args: vec!["-dx11".to_string()],
        command_prefix: vec![],
        env: BTreeMap::new(),
    };
    LaunchCtx {
        plan: Plan {
            game_id: config.game_id.clone(),
            mode_id: ModeId("m".to_string()),
            target: config.target.clone(),
            runner: RunnerId("windows".to_string()),
            slots: BTreeMap::new(),
            config,
        },
        session: SessionInfo {
            id: SessionId("s".to_string()),
            tmp_dir: PathBuf::from("/nonexistent"),
        },
        bindings: Vec::new(),
        run_kind: RunKind::DryRun,
    }
}

#[test]
fn accepts_only_windows_targets() {
    assert!(WindowsRunner.accepts(&TargetKind::Windows));
    assert!(!WindowsRunner.accepts(&TargetKind::NativeLinux));
}

#[test]
fn builds_the_bare_unwrapped_executable() {
    let spec = WindowsRunner.build(&ctx()).expect("build");
    assert_eq!(
        spec.program,
        Some(PathValue::Host(PathBuf::from("/games/g/G.exe")))
    );
    let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
    assert_eq!(args, vec!["-dx11"]);
    assert_eq!(spec.cwd, Some(PathValue::Host(PathBuf::from("/games/g"))));
}
