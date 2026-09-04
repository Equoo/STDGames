//! The `normal` profile test actually shells out to `bwrap` and checks the
//! sandboxed process really ran (skipped automatically if `bwrap` isn't on
//! `PATH`, e.g. in a container that doesn't have it installed). The
//! `super-compat` profile is checked structurally instead: without a real
//! rootfs image to bind, there's nothing runnable inside it to execute.

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use stdg_core::{
    ArgValue, BindMode, BindPurpose, Binding, CommandSpec, GameId, LaunchCtx, Layer, ModeId, Plan,
    PathValue, ResolvedConfig, RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{BwrapLayer, SandboxProfile};

fn test_ctx(root: PathBuf) -> LaunchCtx {
    let config = ResolvedConfig {
        game_id: GameId("test-game".to_string()),
        target: TargetKind::NativeLinux,
        root,
        executable: PathBuf::from("run.sh"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
    };
    let plan = Plan {
        game_id: config.game_id.clone(),
        mode_id: ModeId("test".to_string()),
        target: config.target.clone(),
        runner: RunnerId("native-linux".to_string()),
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

fn bwrap_available() -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|dir| dir.join("bwrap").is_file()))
        .unwrap_or(false)
}

#[test]
fn normal_profile_actually_runs_a_command_through_bwrap() {
    if !bwrap_available() {
        eprintln!("skipping: bwrap not found on PATH");
        return;
    }

    let tmp = std::env::temp_dir().join(format!("stdg-bwrap-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("create test dir");
    let script = tmp.join("run.sh");
    std::fs::write(&script, "#!/bin/sh\necho sandboxed-ok\necho \"MY_TEST_VAR=$MY_TEST_VAR\"\n").expect("write script");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("chmod script");

    let ctx = test_ctx(tmp.clone());
    let layer = BwrapLayer::normal();

    let mut inner = CommandSpec::new(PathValue::Host(script));
    inner.cwd = Some(PathValue::Host(tmp.clone()));
    inner.set_env_literal("MY_TEST_VAR", "42");

    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    let program = spec.program.as_ref().expect("program set").effective().to_path_buf();
    let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();

    let output = Command::new(program).args(&args).output().expect("failed to actually run bwrap");

    std::fs::remove_dir_all(&tmp).ok();

    assert!(
        output.status.success(),
        "bwrap exited with {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sandboxed-ok"), "stdout was: {stdout}");
    assert!(stdout.contains("MY_TEST_VAR=42"), "env var did not cross into the sandbox: {stdout}");
}

#[test]
fn container_needs_bindings_become_bind_flags() {
    let mut ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    ctx.bindings.push(Binding {
        source: PathValue::Host(PathBuf::from("/opt/injected/libsteam_api.so")),
        mode: BindMode::ReadOnly,
        purpose: BindPurpose("steamapi-emu-dll".to_string()),
    });

    let layer = BwrapLayer::normal();
    let outcome = layer.wrap(CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))), &ctx).expect("wrap ok");
    let args: Vec<String> = outcome.into_command().args.iter().map(ArgValue::render).collect();

    let pos = args
        .iter()
        .position(|a| a == "/opt/injected/libsteam_api.so")
        .expect("injected binding source present");
    assert_eq!(args[pos - 1], "--ro-bind");
}

#[test]
fn super_compat_profile_binds_the_image_root_as_slash_and_skips_host_usr() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = BwrapLayer::super_compat(PathBuf::from("/opt/stdgames/images/archlinux"));

    let outcome = layer
        .wrap(CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))), &ctx)
        .expect("wrap ok");
    let args: Vec<String> = outcome.into_command().args.iter().map(ArgValue::render).collect();

    let image_pos = args
        .iter()
        .position(|a| a == "/opt/stdgames/images/archlinux")
        .expect("image root bound");
    assert_eq!(args[image_pos - 1], "--ro-bind");
    assert_eq!(args[image_pos + 1], "/");

    // The host's own /usr is never bound in this profile: the image is
    // supposed to be self-contained.
    assert!(!args.windows(2).any(|w| w[0] == "--ro-bind" && w[1] == "/usr"));
}

#[test]
fn preflight_rejects_super_compat_without_an_image_root() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = BwrapLayer {
        profile: SandboxProfile::SuperCompat,
        image_root: None,
    };
    assert!(layer.preflight(&ctx).is_err());
}

#[test]
fn preflight_rejects_super_compat_with_a_nonexistent_image_root() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = BwrapLayer::super_compat(PathBuf::from("/does/not/exist/anywhere"));
    assert!(layer.preflight(&ctx).is_err());
}
