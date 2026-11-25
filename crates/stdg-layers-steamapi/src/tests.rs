use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::capability::capabilities;
use stdg_core::{
    BindMode, EnvValue, GameId, LaunchCtx, Layer, ModeId, PathValue, Plan, ResolvedConfig, RunKind,
    RunnerId, SessionId, SessionInfo, Slot, TargetKind,
};

use crate::{
    Arch, LibReplace, Overlay, Platform, ReplaceKind, SteamApiNativeLayer, SteamApiReplaceLayer,
    find_steam_api_libs,
};

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

fn layer(
    kind: ReplaceKind,
    libs: Vec<LibReplace>,
    overlay: Option<Overlay>,
) -> SteamApiReplaceLayer {
    SteamApiReplaceLayer {
        kind,
        game_root: PathBuf::from("/games/test-game"),
        replace_lib: !libs.is_empty(),
        libs,
        overlay,
        app_id: None,
    }
}

#[test]
fn replace_patch_exports_app_id_when_set() {
    let ctx = test_ctx();
    let mut spec = stdg_core::CommandSpec::new(PathValue::Host(PathBuf::from("/g/game.exe")));
    layer(ReplaceKind::Emu, Vec::new(), None)
        .patch(&mut spec, &ctx)
        .expect("patch should succeed");
    assert!(!spec.env.contains_key("SteamAppId"));

    let mut with_id = layer(ReplaceKind::Emu, Vec::new(), None);
    with_id.app_id = Some("480".to_string());
    with_id
        .patch(&mut spec, &ctx)
        .expect("patch should succeed");
    assert_eq!(
        spec.env.get("SteamGameId").map(EnvValue::render).as_deref(),
        Some("480")
    );
}

fn lib(replacement: Option<PathBuf>) -> LibReplace {
    LibReplace {
        target: PathBuf::from("/games/test-game/steam_api64.dll"),
        arch: Some(Arch::X64),
        replacement,
    }
}

#[test]
fn platform_picks_the_library_names() {
    assert_eq!(
        Platform::of(&TargetKind::Windows).map(Platform::lib_names),
        Some(&["steam_api64.dll", "steam_api.dll"][..])
    );
    assert_eq!(
        Platform::of(&TargetKind::NativeLinux).map(Platform::lib_names),
        Some(&["libsteam_api.so"][..])
    );
}

#[test]
fn finds_every_copy_and_detects_its_arch() {
    let root = tempfile::tempdir().expect("tempdir");
    let x86 = root.path().join("lib/x86");
    let x64 = root.path().join("lib/x86_64");
    std::fs::create_dir_all(&x86).expect("mkdir");
    std::fs::create_dir_all(&x64).expect("mkdir");
    let mut elf32 = b"\x7fELF\x01".to_vec();
    elf32.resize(64, 0);
    let mut elf64 = b"\x7fELF\x02".to_vec();
    elf64.resize(64, 0);
    std::fs::write(x86.join("libsteam_api.so"), elf32).expect("write");
    std::fs::write(x64.join("libsteam_api.so"), elf64).expect("write");

    let found = find_steam_api_libs(root.path(), Platform::Linux);
    let arches: Vec<_> = found.iter().map(|p| Arch::detect(p)).collect();
    assert_eq!(arches, vec![Some(Arch::X86), Some(Arch::X64)]);
}

#[test]
fn pe_header_gives_the_arch() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("whatever.dll");
    let mut pe = vec![0u8; 0x86];
    pe[..2].copy_from_slice(b"MZ");
    pe[0x3c] = 0x80;
    pe[0x80..0x84].copy_from_slice(b"PE\0\0");
    pe[0x84..0x86].copy_from_slice(&0x014c_u16.to_le_bytes());
    std::fs::write(&path, pe).expect("write");
    assert_eq!(Arch::detect(&path), Some(Arch::X86));
    assert_eq!(
        Arch::detect(&dir.path().join("steam_api64.dll")),
        Some(Arch::X64)
    );
}

#[test]
fn emu_binds_the_replacement_over_the_game_library() {
    let l = layer(
        ReplaceKind::Emu,
        vec![lib(Some(PathBuf::from("/opt/emu/steam_api64.dll")))],
        None,
    );
    let needs = l.container_needs();
    assert_eq!(needs.len(), 1);
    assert_eq!(
        needs[0].source,
        PathValue::Translated {
            host: PathBuf::from("/opt/emu/steam_api64.dll"),
            guest: PathBuf::from("/games/test-game/steam_api64.dll"),
        }
    );
    assert_eq!(needs[0].mode, BindMode::ReadOnly);
    assert_eq!(l.id().0, "steamapi-emu");
    assert_eq!(l.slot(), Slot::SteamApi);
    assert!(l.provides().contains(capabilities::STEAM_HANDSHAKE));
}

#[test]
fn overlay_comes_before_library_binds() {
    let l = layer(
        ReplaceKind::OnlineFix,
        vec![lib(Some(PathBuf::from("/opt/emu/steam_api64.dll")))],
        Some(Overlay {
            source: PathBuf::from("/fix"),
            state_dir: PathBuf::from("/state"),
        }),
    );
    let needs = l.container_needs();
    assert_eq!(
        needs[0].mode,
        BindMode::Overlay {
            upper: PathBuf::from("/state/upper"),
            work: PathBuf::from("/state/work"),
        }
    );
    assert_eq!(
        needs[0].source.effective(),
        PathBuf::from("/games/test-game")
    );
    assert_eq!(needs[1].mode, BindMode::ReadOnly);
}

#[test]
fn preflight_requires_a_replacement_per_found_library() {
    let missing = layer(ReplaceKind::Emu, vec![lib(None)], None);
    let err = missing.preflight(&test_ctx()).expect_err("no lib_x64");
    assert!(err.hint.is_some_and(|h| h.contains("lib_x64")));

    let replacement = tempfile::NamedTempFile::new().expect("temp file");
    let ok = layer(
        ReplaceKind::Emu,
        vec![lib(Some(replacement.path().to_path_buf()))],
        None,
    );
    assert!(ok.preflight(&test_ctx()).is_ok());
}

#[test]
fn preflight_rejects_emu_without_any_game_library() {
    let mut l = layer(ReplaceKind::Emu, vec![], None);
    l.replace_lib = true;
    assert!(l.preflight(&test_ctx()).is_err());
}

#[test]
fn preflight_rejects_a_missing_overlay_dir() {
    let l = layer(
        ReplaceKind::OnlineFix,
        vec![],
        Some(Overlay {
            source: PathBuf::from("/definitely/not/here"),
            state_dir: PathBuf::from("/state"),
        }),
    );
    assert!(l.preflight(&test_ctx()).is_err());
}

#[test]
fn prepare_creates_overlay_state_only_when_live() {
    let state = tempfile::tempdir().expect("tempdir");
    let fix = tempfile::tempdir().expect("tempdir");
    let l = layer(
        ReplaceKind::OnlineFix,
        vec![],
        Some(Overlay {
            source: fix.path().to_path_buf(),
            state_dir: state.path().join("s"),
        }),
    );
    let mut ctx = test_ctx();
    l.prepare(&mut ctx).expect("dry run");
    assert!(!state.path().join("s").exists());

    ctx.run_kind = RunKind::Live;
    l.prepare(&mut ctx).expect("live");
    assert!(state.path().join("s/upper").is_dir());
    assert!(state.path().join("s/work").is_dir());
}
