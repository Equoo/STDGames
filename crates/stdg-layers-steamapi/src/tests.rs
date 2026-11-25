use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, EnvValue, GameId, LaunchCtx, Layer, ModeId, PathValue, Plan, ResolvedConfig, RunKind,
    RunnerId, SessionId, SessionInfo, Slot, TargetKind,
};

use crate::{EmuVariant, SteamApiEmuLayer, SteamApiNativeLayer, SteamApiOnlineFixLayer};

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
            tmp_dir: PathBuf::from("/nonexistent/session"),
        },
        bindings: Vec::new(),
        run_kind: RunKind::DryRun,
    }
}

#[test]
fn native_preflight_is_a_noop_during_a_dry_run() {
    let ctx = test_ctx();
    let layer = SteamApiNativeLayer {
        app_id: "480".to_string(),
    };
    // A dry run never fails preflight.
    assert!(layer.preflight(&ctx).is_ok());
}

#[test]
fn native_patch_sets_the_app_id_env_vars() {
    let ctx = test_ctx();
    let layer = SteamApiNativeLayer {
        app_id: "480".to_string(),
    };
    let mut spec = stdg_core::CommandSpec::new(PathValue::Host(PathBuf::from(
        "/games/test-game/test-game.exe",
    )));
    layer.patch(&mut spec, &ctx).expect("patch should succeed");

    assert_eq!(
        spec.env.get("SteamAppId").map(EnvValue::render).as_deref(),
        Some("480")
    );
    assert_eq!(
        spec.env.get("SteamGameId").map(EnvValue::render).as_deref(),
        Some("480")
    );
}

fn emu_layer(replacement: PathBuf) -> SteamApiEmuLayer {
    SteamApiEmuLayer {
        variant: EmuVariant::PlainReplace,
        replacement_lib_path: replacement,
        target_lib_path: PathBuf::from("/games/test-game/steam_api64.dll"),
    }
}

#[test]
fn emu_container_needs_shadows_the_target_with_the_replacement() {
    let layer = emu_layer(PathBuf::from("/opt/emu/steam_api64.dll"));
    let needs = layer.container_needs();
    assert_eq!(needs.len(), 1);
    assert_eq!(
        needs[0].source,
        PathValue::Translated {
            host: PathBuf::from("/opt/emu/steam_api64.dll"),
            guest: PathBuf::from("/games/test-game/steam_api64.dll"),
        }
    );
    assert_eq!(needs[0].mode, BindMode::ReadOnly);
    assert_eq!(needs[0].purpose.0, "steamapi-emu-dll");
}

#[test]
fn emu_plain_replace_requires_nothing_and_provides_the_steam_handshake() {
    let layer = emu_layer(PathBuf::from("/opt/emu/steam_api64.dll"));
    assert!(layer.requires().is_empty());
    assert!(layer.provides().contains(capabilities::STEAM_HANDSHAKE));
    assert_eq!(layer.slot(), Slot::SteamApi);
}

#[test]
fn emu_preflight_rejects_a_missing_replacement_library() {
    let layer = emu_layer(PathBuf::from("/definitely/not/installed/steam_api64.dll"));
    let err = layer
        .preflight(&test_ctx())
        .expect_err("replacement library does not exist");
    assert!(err.message.contains("not found"));
}

#[test]
fn emu_preflight_accepts_an_existing_replacement_library() {
    let replacement = tempfile::NamedTempFile::new().expect("temp file");
    let layer = emu_layer(replacement.path().to_path_buf());
    assert!(layer.preflight(&test_ctx()).is_ok());
}

#[test]
fn onlinefix_preflight_checks_the_replacement_library() {
    let replacement = tempfile::NamedTempFile::new().expect("temp file");
    let present = SteamApiOnlineFixLayer {
        replacement_lib_path: replacement.path().to_path_buf(),
        target_lib_path: PathBuf::from("/games/test-game/steam_api64.dll"),
    };
    assert!(present.preflight(&test_ctx()).is_ok());

    let missing = SteamApiOnlineFixLayer {
        replacement_lib_path: PathBuf::from("/definitely/not/installed/steam_api64.dll"),
        target_lib_path: PathBuf::from("/games/test-game/steam_api64.dll"),
    };
    assert!(missing.preflight(&test_ctx()).is_err());
    assert_eq!(missing.id().0, "steamapi-onlinefix");
}
