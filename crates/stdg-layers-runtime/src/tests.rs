//! Structural tests only: unlike `stdg-layers-sandbox`'s bwrap layer, there
//! is no `pressure-vessel-wrap` in this dev environment to actually run
//! against (it ships only as part of an installed Steam Linux Runtime).
//! These check the invocation shape built from the depot's documented `run`
//! script contract, not real execution.

use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::{
    ArgValue, CommandSpec, GameId, LaunchCtx, Layer, ModeId, Plan, PathValue, ResolvedConfig,
    RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{PressureVesselLayer, PressureVesselVariant};

fn test_ctx() -> LaunchCtx {
    let config = ResolvedConfig {
        game_id: GameId("test-game".to_string()),
        target: TargetKind::Windows,
        root: PathBuf::from("/games/test-game"),
        executable: PathBuf::from("test-game.exe"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
    };
    let plan = Plan {
        game_id: config.game_id.clone(),
        mode_id: ModeId("test".to_string()),
        target: config.target.clone(),
        runner: RunnerId("windows".to_string()),
        slots: BTreeMap::new(),
        config,
    };
    LaunchCtx {
        plan,
        session: SessionInfo {
            id: SessionId("test-session".to_string()),
            tmp_dir: std::env::temp_dir(),
        },
        bindings: Vec::new(),
        dry_run: false,
    }
}

fn rendered_args(spec: &CommandSpec) -> Vec<String> {
    spec.args.iter().map(ArgValue::render).collect()
}

#[test]
fn wrap_invokes_the_depot_run_script_with_a_separator() {
    let ctx = test_ctx();
    let layer = PressureVesselLayer {
        variant: PressureVesselVariant::Soldier,
        depot_path: PathBuf::from("/opt/steam-runtime/SteamLinuxRuntime_soldier"),
    };

    let inner = CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/test-game.exe")));
    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(
        spec.program.as_ref().unwrap().effective(),
        PathBuf::from("/opt/steam-runtime/SteamLinuxRuntime_soldier/run")
    );
    let args = rendered_args(&spec);
    assert_eq!(args, vec!["--", "/games/test-game/test-game.exe"]);
}

#[test]
fn wrap_preserves_inner_args_env_and_cwd() {
    let ctx = test_ctx();
    let layer = PressureVesselLayer {
        variant: PressureVesselVariant::Sniper,
        depot_path: PathBuf::from("/opt/steam-runtime/SteamLinuxRuntime_sniper"),
    };

    let mut inner = CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/test-game.exe")));
    inner.push_arg_literal("-fullscreen");
    inner.set_env_literal("SteamAppId", "480");
    inner.cwd = Some(PathValue::Host(PathBuf::from("/games/test-game")));

    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(rendered_args(&spec), vec!["--", "/games/test-game/test-game.exe", "-fullscreen"]);
    assert_eq!(spec.env.get("SteamAppId").map(|v| v.render()), Some("480".to_string()));
    assert_eq!(spec.cwd.as_ref().unwrap().effective(), PathBuf::from("/games/test-game"));
}

#[test]
fn ids_match_the_variant() {
    let soldier = PressureVesselLayer {
        variant: PressureVesselVariant::Soldier,
        depot_path: PathBuf::from("/nonexistent"),
    };
    let sniper = PressureVesselLayer {
        variant: PressureVesselVariant::Sniper,
        depot_path: PathBuf::from("/nonexistent"),
    };
    assert_eq!(soldier.id().0, "soldier");
    assert_eq!(sniper.id().0, "sniper");
}

#[test]
fn container_needs_binds_the_depot() {
    let layer = PressureVesselLayer {
        variant: PressureVesselVariant::Soldier,
        depot_path: PathBuf::from("/opt/steam-runtime/SteamLinuxRuntime_soldier"),
    };
    let needs = layer.container_needs();
    assert_eq!(needs.len(), 1);
    assert_eq!(needs[0].source.host(), PathBuf::from("/opt/steam-runtime/SteamLinuxRuntime_soldier"));
}

#[test]
fn preflight_fails_clearly_when_the_depot_is_missing() {
    let ctx = test_ctx();
    let layer = PressureVesselLayer {
        variant: PressureVesselVariant::Soldier,
        depot_path: PathBuf::from("/definitely/not/an/installed/depot"),
    };
    let err = layer.preflight(&ctx).expect_err("depot does not exist");
    assert!(err.message.contains("not found"));
}
