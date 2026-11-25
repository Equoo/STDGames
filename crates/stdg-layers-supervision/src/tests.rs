use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    GameId, LaunchCtx, Layer, ModeId, Plan, ResolvedConfig, RunKind, RunnerId, SessionId,
    SessionInfo, Slot, TargetKind,
};

use crate::{SupervisionLayer, unified_cgroup_path};

fn ctx(run_kind: RunKind) -> LaunchCtx {
    let config = ResolvedConfig {
        game_id: GameId("g".to_string()),
        target: TargetKind::NativeLinux,
        root: PathBuf::from("/games/g"),
        executable: PathBuf::from("g"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
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
            tmp_dir: PathBuf::from("/nonexistent/session"),
        },
        bindings: Vec::new(),
        run_kind,
    }
}

#[test]
fn occupies_the_supervision_slot() {
    let layer = SupervisionLayer::new();
    assert_eq!(layer.slot(), Slot::Supervision);
    assert!(layer.provides().contains(capabilities::CGROUP_SUPERVISED));
}

#[test]
fn parses_the_unified_hierarchy_line() {
    let content = "12:cpu:/legacy\n0::/user.slice/app.slice/term.scope\n";
    assert_eq!(
        unified_cgroup_path(content).as_deref(),
        Some("/user.slice/app.slice/term.scope")
    );
    assert_eq!(unified_cgroup_path("3:memory:/x\n"), None);
}

#[test]
fn dry_run_creates_no_cgroup() {
    let parent = tempfile::tempdir().expect("tempdir");
    let layer = SupervisionLayer {
        cgroup_parent: parent.path().to_path_buf(),
    };
    let guard = layer
        .prepare(&mut ctx(RunKind::DryRun))
        .expect("dry run never fails");
    assert_eq!(guard.label(), "subreaper-fallback");
    assert_eq!(
        std::fs::read_dir(parent.path()).expect("readable").count(),
        0
    );
}

#[test]
fn falls_back_to_the_subreaper_without_cgroup_delegation() {
    let not_a_cgroup = tempfile::tempdir().expect("tempdir");
    let layer = SupervisionLayer {
        cgroup_parent: not_a_cgroup.path().to_path_buf(),
    };
    let guard = layer
        .prepare(&mut ctx(RunKind::Live))
        .expect("fallback succeeds");
    assert_eq!(guard.label(), "subreaper-fallback");
}
