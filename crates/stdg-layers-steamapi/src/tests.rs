use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, GameId, LaunchCtx, Layer, ModeId, PathValue, Plan, ResolvedConfig, RunnerId,
    SessionId, SessionInfo, TargetKind,
};

use crate::{EmuVariant, SteamApiEmuLayer, SteamApiNativeLayer};

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
        dry_run: true,
    }
}

// -- SteamApiNativeLayer -----------------------------------------------

#[test]
fn native_preflight_is_a_noop_during_a_dry_run() {
    let ctx = test_ctx();
    let layer = SteamApiNativeLayer {
        app_id: "480".to_string(),
    };
    // Regardless of what's actually installed/running on the machine
    // executing this test, a dry run must never fail preflight.
    assert!(layer.preflight(&ctx).is_ok());
}

#[test]
fn native_patch_sets_the_app_id_env_vars() {
    let ctx = test_ctx();
    let layer = SteamApiNativeLayer {
        app_id: "480".to_string(),
    };
    let mut spec = stdg_core::CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/test-game.exe")));
    layer.patch(&mut spec, &ctx).expect("patch should succeed");

    assert_eq!(spec.env.get("SteamAppId").map(|v| v.render()), Some("480".to_string()));
    assert_eq!(spec.env.get("SteamGameId").map(|v| v.render()), Some("480".to_string()));
}

// -- SteamApiEmuLayer -----------------------------------------------------

fn temp_file(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("stdg-steamapi-test-{}-{}", std::process::id(), name));
    std::fs::write(&path, b"fake replacement lib").expect("write temp fixture file");
    path
}

#[test]
fn emu_container_needs_shadows_the_target_with_the_replacement() {
    let replacement = temp_file("libsteam_api.so");
    let layer = SteamApiEmuLayer {
        variant: EmuVariant::OverNative,
        replacement_lib_path: replacement.clone(),
        target_lib_path: PathBuf::from("/games/test-game/libsteam_api.so"),
    };

    let needs = layer.container_needs();
    assert_eq!(needs.len(), 1);
    assert_eq!(
        needs[0].source,
        PathValue::Translated {
            host: replacement.clone(),
            guest: PathBuf::from("/games/test-game/libsteam_api.so"),
        }
    );
    assert_eq!(needs[0].mode, BindMode::ReadOnly);
    assert_eq!(needs[0].purpose.0, "steamapi-emu-over-native-dll");

    std::fs::remove_file(&replacement).ok();
}

#[test]
fn emu_plain_replace_requires_windows_abi_but_over_native_does_not() {
    let plain = SteamApiEmuLayer {
        variant: EmuVariant::PlainReplace,
        replacement_lib_path: PathBuf::from("/opt/emu/steam_api64.dll"),
        target_lib_path: PathBuf::from("/games/test-game/steam_api64.dll"),
    };
    assert!(plain.requires().contains(capabilities::WINDOWS_ABI));

    let over_native = SteamApiEmuLayer {
        variant: EmuVariant::OverNative,
        replacement_lib_path: PathBuf::from("/opt/emu/libsteam_api.so"),
        target_lib_path: PathBuf::from("/games/test-game/libsteam_api.so"),
    };
    assert!(over_native.requires().is_empty());
}

#[test]
fn emu_both_variants_provide_the_steam_handshake() {
    let layer = SteamApiEmuLayer {
        variant: EmuVariant::PlainReplace,
        replacement_lib_path: PathBuf::from("/opt/emu/steam_api64.dll"),
        target_lib_path: PathBuf::from("/games/test-game/steam_api64.dll"),
    };
    assert!(layer.provides().contains(capabilities::STEAM_HANDSHAKE));
}

#[test]
fn emu_preflight_rejects_a_missing_replacement_library() {
    let ctx = test_ctx();
    let layer = SteamApiEmuLayer {
        variant: EmuVariant::OverNative,
        replacement_lib_path: PathBuf::from("/definitely/not/installed/libsteam_api.so"),
        target_lib_path: PathBuf::from("/games/test-game/libsteam_api.so"),
    };
    let err = layer.preflight(&ctx).expect_err("replacement library does not exist");
    assert!(err.message.contains("not found"));
}

#[test]
fn emu_preflight_accepts_an_existing_replacement_library() {
    let ctx = test_ctx();
    let replacement = temp_file("preflight-ok.so");
    let layer = SteamApiEmuLayer {
        variant: EmuVariant::OverNative,
        replacement_lib_path: replacement.clone(),
        target_lib_path: PathBuf::from("/games/test-game/libsteam_api.so"),
    };
    assert!(layer.preflight(&ctx).is_ok());

    std::fs::remove_file(&replacement).ok();
}
