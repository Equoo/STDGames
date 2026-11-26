//! Structural tests against the `proton run` contract.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use stdg_core::{
    ArgValue, CommandSpec, GameId, LaunchCtx, Layer, ModeId, PathValue, Plan, ResolvedConfig,
    RunKind, RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{ProtonLayer, SteamClient, WineLayer, find_steam_client_install};

fn test_ctx() -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("test-game".to_string()),
        target: TargetKind::Windows,
        root: PathBuf::from("/games/test-game"),
        executable: PathBuf::from("test-game.exe"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
        saves: stdg_core::SavesConfig::default(),
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
        wine_prefix: None,
    }
}

fn rendered_args(spec: &CommandSpec) -> Vec<String> {
    spec.args.iter().map(ArgValue::render).collect()
}

#[test]
fn proton_wrap_invokes_the_entry_point_script_with_run() {
    let ctx = test_ctx();
    let layer = ProtonLayer {
        version: Some("9.0-4".to_string()),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: Some(PathBuf::from("/opt/steam/Steam")),
        steam_stub_dir: None,
    };

    let inner = CommandSpec::new(PathValue::Host(PathBuf::from(
        "/games/test-game/test-game.exe",
    )));
    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(
        spec.program.as_ref().unwrap().effective(),
        PathBuf::from("/opt/protons/ge10.25/proton")
    );
    assert_eq!(
        rendered_args(&spec),
        vec!["run", "/games/test-game/test-game.exe"]
    );
    assert_eq!(
        spec.env
            .get("STEAM_COMPAT_DATA_PATH")
            .map(stdg_core::EnvValue::render),
        Some("/opt/prefixes/ge10.25".to_string())
    );
    assert_eq!(
        spec.env.get("WINEPREFIX").map(stdg_core::EnvValue::render),
        Some("/opt/prefixes/ge10.25".to_string())
    );
    assert_eq!(
        spec.env
            .get("STEAM_COMPAT_CLIENT_INSTALL_PATH")
            .map(stdg_core::EnvValue::render),
        Some("/opt/steam/Steam".to_string())
    );
}

#[test]
fn proton_preserves_inner_args_and_env() {
    let ctx = test_ctx();
    let layer = ProtonLayer {
        version: Some("9.0-4".to_string()),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: None,
        steam_stub_dir: None,
    };

    let mut inner = CommandSpec::new(PathValue::Host(PathBuf::from(
        "/games/test-game/test-game.exe",
    )));
    inner.push_arg_literal("-fullscreen");
    inner.set_env_literal("SteamAppId", "480");

    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(
        rendered_args(&spec),
        vec!["run", "/games/test-game/test-game.exe", "-fullscreen"]
    );
    assert_eq!(
        spec.env.get("SteamAppId").map(stdg_core::EnvValue::render),
        Some("480".to_string())
    );
}

#[test]
fn wine_uses_the_same_proton_script_contract() {
    let ctx = test_ctx();
    let layer = WineLayer {
        version: None,
        wine_path: PathBuf::from("/opt/protons/wine"),
        prefix_path: PathBuf::from("/opt/prefixes/wine"),
        steam_client_path: None,
        steam_stub_dir: None,
    };

    let inner = CommandSpec::new(PathValue::Host(PathBuf::from(
        "/games/test-game/test-game.exe",
    )));
    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    assert_eq!(
        spec.program.as_ref().unwrap().effective(),
        PathBuf::from("/opt/protons/wine/proton")
    );
    assert_eq!(
        spec.env.get("WINEPREFIX").map(stdg_core::EnvValue::render),
        Some("/opt/prefixes/wine".to_string())
    );
}

#[test]
fn container_needs_covers_install_prefix_and_steam_client() {
    let layer = ProtonLayer {
        version: Some("9.0-4".to_string()),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: Some(PathBuf::from("/opt/steam/Steam")),
        steam_stub_dir: None,
    };

    let needs = layer.container_needs(&test_ctx());
    let sources: Vec<PathBuf> = needs
        .iter()
        .map(|b| b.source.host().to_path_buf())
        .collect();

    assert!(
        sources.contains(&PathBuf::from("/opt/protons/ge10.25")),
        "proton install must be bound: {sources:?}"
    );
    assert!(
        sources.contains(&PathBuf::from("/opt/prefixes/ge10.25")),
        "prefix must be bound: {sources:?}"
    );
    assert!(
        sources.contains(&PathBuf::from("/opt/steam/Steam")),
        "steam client must be bound: {sources:?}"
    );
}

#[test]
fn preflight_rejects_a_missing_proton_build() {
    let ctx = test_ctx();
    let layer = ProtonLayer {
        version: Some("9.0-4".to_string()),
        proton_path: PathBuf::from("/definitely/not/installed"),
        prefix_path: PathBuf::from("/opt/prefixes/ge10.25"),
        steam_client_path: None,
        steam_stub_dir: None,
    };
    let err = layer
        .preflight(&ctx)
        .expect_err("proton build does not exist");
    assert!(err.message.contains("not found"));
}

#[test]
fn prepare_creates_the_prefix_directory() {
    let mut ctx = test_ctx();
    ctx.run_kind = RunKind::Live;
    let tmp = tempfile::tempdir().expect("tempdir");
    let prefix = tmp.path().join("prefix");

    let layer = ProtonLayer {
        version: Some("9.0-4".to_string()),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: prefix.clone(),
        steam_client_path: None,
        steam_stub_dir: None,
    };

    let _guard = layer
        .prepare(&mut ctx)
        .expect("prepare should create the prefix dir");
    assert!(prefix.is_dir());
}

#[test]
fn prepare_does_not_touch_disk_during_a_dry_run() {
    let mut ctx = test_ctx();
    ctx.run_kind = RunKind::DryRun;
    let tmp = tempfile::tempdir().expect("tempdir");
    let prefix = tmp.path().join("prefix");

    let layer = ProtonLayer {
        version: Some("9.0-4".to_string()),
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: prefix.clone(),
        steam_client_path: None,
        steam_stub_dir: None,
    };

    let _guard = layer
        .prepare(&mut ctx)
        .expect("prepare should succeed even without touching disk");
    assert!(
        !prefix.exists(),
        "a dry run must not create the prefix directory"
    );
}

#[test]
fn steam_stub_is_used_when_no_client_is_found() {
    let stub = PathBuf::from("/stub/steam");
    let resolved = SteamClient::resolve(None, Some(&stub));
    match find_steam_client_install() {
        Some(installed) => assert_eq!(resolved, Some(SteamClient::Real(installed))),
        None => assert_eq!(resolved, Some(SteamClient::Stub(stub))),
    }
}

#[test]
fn configured_client_wins_over_the_stub() {
    let resolved = SteamClient::resolve(Some(Path::new("/opt/steam")), Some(Path::new("/stub")));
    assert_eq!(
        resolved,
        Some(SteamClient::Real(PathBuf::from("/opt/steam")))
    );
}

#[test]
fn prepare_creates_the_steam_stub_when_used() {
    if find_steam_client_install().is_some() {
        return;
    }
    let mut ctx = test_ctx();
    ctx.run_kind = RunKind::Live;
    let tmp = tempfile::tempdir().expect("tempdir");
    let stub = tmp.path().join("steam-stub");
    let layer = ProtonLayer {
        version: None,
        proton_path: PathBuf::from("/opt/protons/ge10.25"),
        prefix_path: tmp.path().join("prefix"),
        steam_client_path: None,
        steam_stub_dir: Some(stub.clone()),
    };

    let _guard = layer.prepare(&mut ctx).expect("prepare ok");
    assert!(stub.is_dir());
    let spec = layer
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/g/game.exe"))),
            &ctx,
        )
        .expect("wrap ok")
        .into_command();
    assert_eq!(
        spec.env
            .get("STEAM_COMPAT_CLIENT_INSTALL_PATH")
            .map(stdg_core::EnvValue::render),
        Some(stub.display().to_string())
    );
    assert!(
        layer
            .container_needs(&test_ctx())
            .iter()
            .any(|b| b.source.host() == stub)
    );
}

#[test]
fn steam_install_comes_from_the_running_exe_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let install = tmp.path().join("Steam");
    std::fs::create_dir_all(install.join("linux64")).expect("mkdir");
    std::fs::write(install.join("linux64/steamclient.so"), b"").expect("write");

    for bin in ["steamrt64/steam", "ubuntu12_32/steam"] {
        assert_eq!(
            crate::steam_install_of(&install.join(bin)),
            Some(install.clone())
        );
    }
    assert_eq!(
        crate::steam_install_of(&install.join("ubuntu12_32/other")),
        None
    );
    assert_eq!(crate::steam_install_of(&tmp.path().join("bin/steam")), None);
}
