use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::{
    ArgValue, CoreError, GameId, LaunchCtx, ModeId, PathValue, Plan, PlatformId, ResolvedConfig,
    RunKind, Runner, RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{EmulatorManifest, EmulatorRunner};

const MANIFEST: &str = r#"
id = "mgba"
platforms = ["gba"]
binary = "/usr/bin/mgba"
args = ["-f", "{rom}"]
"#;

fn runner() -> EmulatorRunner {
    EmulatorRunner::new(vec![
        EmulatorManifest::from_toml_str(MANIFEST).expect("valid manifest"),
    ])
}

fn ctx(target: TargetKind) -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("g".to_string()),
        target,
        root: PathBuf::from("/roms"),
        executable: PathBuf::from("g.gba"),
        args: vec!["--extra".to_string()],
        command_prefix: vec![],
        env: BTreeMap::new(),
    };
    LaunchCtx {
        plan: Plan {
            game_id: config.game_id.clone(),
            mode_id: ModeId("m".to_string()),
            target: config.target.clone(),
            runner: RunnerId("emulator".to_string()),
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

fn gba() -> TargetKind {
    TargetKind::Rom(PlatformId("gba".to_string()))
}

#[test]
fn accepts_only_platforms_with_a_manifest() {
    assert!(runner().accepts(&gba()));
    assert!(!runner().accepts(&TargetKind::Rom(PlatformId("snes".to_string()))));
    assert!(!runner().accepts(&TargetKind::Windows));
}

#[test]
fn substitutes_the_rom_path_into_the_argument_template() {
    let spec = runner().build(&ctx(gba())).expect("build");
    assert_eq!(
        spec.program,
        Some(PathValue::Host(PathBuf::from("/usr/bin/mgba")))
    );
    let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
    assert_eq!(args, vec!["-f", "/roms/g.gba", "--extra"]);
}

#[test]
fn rejects_a_non_rom_target() {
    let err = runner()
        .build(&ctx(TargetKind::Windows))
        .expect_err("not a ROM");
    assert!(matches!(err, CoreError::NoRunnerForTarget(_)));
}

#[test]
fn manifest_without_required_fields_is_rejected() {
    assert!(EmulatorManifest::from_toml_str("id = \"x\"").is_err());
}
