//! Structural tests: no real Proton/Wine build is installed in this dev
//! environment (they're multi-hundred-MB distributions, not something to
//! vendor for a test), so these check the invocation shape against Valve's
//! documented `proton run` contract rather than real execution.

use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::{
    ArgValue, CommandSpec, GameId, LaunchCtx, Layer, ModeId, Plan, PathValue, ResolvedConfig,
    RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{ProtonLayer, WineLayer};

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

fn rendered_args(spec: &CommandSpec) -> Vec<String> {
    spec.args.iter().map(ArgValue::render).collect()
}

#[test]
fn proton_wrap_invokes_the_entry_point_script_with_run() {
    let ctx = test_ctx();
    let layer = ProtonLayer {
        version: "9.0-4".to_string(),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: Some(PathBuf::from("/opt/steam/Steam")),
    };

    let inner = CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/test-game.exe")));
    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(spec.program.as_ref().unwrap().effective(), PathBuf::from("/opt/protons/ge10.25/proton"));
    assert_eq!(rendered_args(&spec), vec!["run", "/games/test-game/test-game.exe"]);
    assert_eq!(spec.env.get("STEAM_COMPAT_DATA_PATH").map(|v| v.render()), Some("/opt/prefixes/ge10.25".to_string()));
    assert_eq!(spec.env.get("WINEPREFIX").map(|v| v.render()), Some("/opt/prefixes/ge10.25".to_string()));
    assert_eq!(spec.env.get("STEAM_COMPAT_CLIENT_INSTALL_PATH").map(|v| v.render()), Some("/opt/steam/Steam".to_string()));
}

#[test]
fn proton_preserves_inner_args_and_env() {
    let ctx = test_ctx();
    let layer = ProtonLayer {
        version: "9.0-4".to_string(),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: None,
    };

    let mut inner = CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/test-game.exe")));
    inner.push_arg_literal("-fullscreen");
    inner.set_env_literal("SteamAppId", "480");

    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(rendered_args(&spec), vec!["run", "/games/test-game/test-game.exe", "-fullscreen"]);
    assert_eq!(spec.env.get("SteamAppId").map(|v| v.render()), Some("480".to_string()));
}

#[test]
fn wine_uses_the_same_proton_script_contract() {
    let ctx = test_ctx();
    let layer = WineLayer {
        version: "custom".to_string(),
        wine_path: PathBuf::from("/opt/protons/wine"),
        prefix_path: PathBuf::from("/opt/prefixes/wine"),
        steam_client_path: None,
    };

    let inner = CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/test-game.exe")));
    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(spec.program.as_ref().unwrap().effective(), PathBuf::from("/opt/protons/wine/proton"));
    assert_eq!(spec.env.get("WINEPREFIX").map(|v| v.render()), Some("/opt/prefixes/wine".to_string()));
}

#[test]
fn container_needs_covers_install_prefix_and_steam_client() {
    let layer = ProtonLayer {
        version: "9.0-4".to_string(),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: Some(PathBuf::from("/opt/steam/Steam")),
    };

    let needs = layer.container_needs();
    let sources: Vec<PathBuf> = needs.iter().map(|b| b.source.host().to_path_buf()).collect();

    assert!(sources.contains(&PathBuf::from("/opt/protons/ge10.25")), "proton install must be bound: {sources:?}");
    assert!(sources.contains(&PathBuf::from("/opt/prefixes/ge10.25")), "prefix must be bound: {sources:?}");
    assert!(sources.contains(&PathBuf::from("/opt/steam/Steam")), "steam client must be bound: {sources:?}");
}

#[test]
fn preflight_rejects_a_missing_proton_build() {
    let ctx = test_ctx();
    let layer = ProtonLayer {
        version: "9.0-4".to_string(),
        proton_path: PathBuf::from("/definitely/not/installed"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: None,
    };
    let err = layer.preflight(&ctx).expect_err("proton build does not exist");
    assert!(err.message.contains("not found"));
}

#[test]
fn prepare_creates_the_prefix_directory() {
    let mut ctx = test_ctx();
    ctx.dry_run = false;
    let prefix = std::env::temp_dir().join(format!("stdg-compat-test-prefix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&prefix);

    let layer = ProtonLayer {
        version: "9.0-4".to_string(),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: prefix.clone(),
        steam_client_path: None,
    };

    let _guard = layer.prepare(&mut ctx).expect("prepare should create the prefix dir");
    assert!(prefix.is_dir());

    std::fs::remove_dir_all(&prefix).ok();
}

#[test]
fn prepare_does_not_touch_disk_during_a_dry_run() {
    let mut ctx = test_ctx();
    ctx.dry_run = true;
    let prefix = std::env::temp_dir().join(format!("stdg-compat-test-dryrun-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&prefix);

    let layer = ProtonLayer {
        version: "9.0-4".to_string(),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: prefix.clone(),
        steam_client_path: None,
    };

    let _guard = layer.prepare(&mut ctx).expect("prepare should succeed even without touching disk");
    assert!(!prefix.exists(), "dry_run must not create the prefix directory");
}
