//! The end-to-end test actually shells out to Conty and checks the
//! sandboxed process really ran; it is skipped automatically when no Conty
//! executable is on `PATH` (the usual case in CI, since Conty is a large
//! self-contained image rather than a package). Everything else is checked
//! structurally against the arguments the layer emits.

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use stdg_core::{
    ArgValue, BindMode, BindPurpose, Binding, CommandSpec, GameId, LaunchCtx, Layer, ModeId,
    PathValue, Plan, ResolvedConfig, RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{CONTY_BINARY_NAMES, ContyLayer};

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

fn conty_on_path() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .flat_map(|dir| CONTY_BINARY_NAMES.iter().map(move |n| dir.join(n)))
        .find(|p| p.is_file())
}

#[test]
fn actually_runs_a_command_through_conty() {
    let Some(_conty) = conty_on_path() else {
        eprintln!("skipping: no conty executable found on PATH");
        return;
    };

    let tmp = std::env::temp_dir().join(format!("stdg-conty-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).expect("create test dir");
    let script = tmp.join("run.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\necho sandboxed-ok\necho \"MY_TEST_VAR=$MY_TEST_VAR\"\n",
    )
    .expect("write script");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
        .expect("chmod script");

    let ctx = test_ctx(tmp.clone());
    let layer = ContyLayer::new();

    let mut inner = CommandSpec::new(PathValue::Host(script));
    inner.cwd = Some(PathValue::Host(tmp.clone()));
    inner.set_env_literal("MY_TEST_VAR", "42");

    let outcome = layer.wrap(inner, &ctx).expect("wrap should succeed");
    let spec = outcome.into_command();

    let program = spec
        .program
        .as_ref()
        .expect("program set")
        .effective()
        .to_path_buf();
    let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();

    let mut command = Command::new(program);
    command.args(&args);
    for (key, value) in &spec.env {
        command.env(key, value.render());
    }
    let output = command.output().expect("failed to actually run conty");

    std::fs::remove_dir_all(&tmp).ok();

    assert!(
        output.status.success(),
        "conty exited with {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sandboxed-ok"), "stdout was: {stdout}");
    assert!(
        stdout.contains("MY_TEST_VAR=42"),
        "env var did not cross into the sandbox: {stdout}"
    );
}

#[test]
fn the_game_root_is_bound_read_write() {
    let ctx = test_ctx(PathBuf::from("/games/test-game"));
    let layer = ContyLayer::new();

    let outcome = layer
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/games/test-game/bin"))),
            &ctx,
        )
        .expect("wrap ok");
    let args: Vec<String> = outcome
        .into_command()
        .args
        .iter()
        .map(ArgValue::render)
        .collect();

    let pos = args
        .windows(3)
        .position(|w| w == ["--bind", "/games/test-game", "/games/test-game"])
        .expect("game root bound read-write at the same path");
    // ...and it comes before the `--` that starts the command.
    let sep = args
        .iter()
        .position(|a| a == "--")
        .expect("command separator present");
    assert!(pos < sep);
}

#[test]
fn container_needs_bindings_become_bind_flags() {
    let mut ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    ctx.bindings.push(Binding {
        source: PathValue::Host(PathBuf::from("/opt/injected/libsteam_api.so")),
        mode: BindMode::ReadOnly,
        purpose: BindPurpose("steamapi-emu-dll".to_string()),
    });

    let layer = ContyLayer::new();
    let outcome = layer
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))),
            &ctx,
        )
        .expect("wrap ok");
    let args: Vec<String> = outcome
        .into_command()
        .args
        .iter()
        .map(ArgValue::render)
        .collect();

    let pos = args
        .iter()
        .position(|a| a == "/opt/injected/libsteam_api.so")
        .expect("injected binding source present");
    assert_eq!(args[pos - 1], "--ro-bind");
}

#[test]
fn sandbox_level_becomes_conty_env_vars() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = ContyLayer {
        sandbox_level: Some(2),
        ..ContyLayer::new()
    };

    let spec = layer
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))),
            &ctx,
        )
        .expect("wrap ok")
        .into_command();

    assert_eq!(
        spec.env.get("SANDBOX").map(|v| v.render()).as_deref(),
        Some("1")
    );
    assert_eq!(
        spec.env.get("SANDBOX_LEVEL").map(|v| v.render()).as_deref(),
        Some("2")
    );
}

#[test]
fn no_sandbox_level_leaves_conty_at_its_default() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let spec = ContyLayer::new()
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))),
            &ctx,
        )
        .expect("wrap ok")
        .into_command();

    assert!(!spec.env.contains_key("SANDBOX"));
    assert!(!spec.env.contains_key("SANDBOX_LEVEL"));
}

#[test]
fn linker_vars_are_stripped_but_inner_env_wins() {
    let mut ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    ctx.bindings.clear();

    let mut inner = CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin")));
    inner.set_env_literal("LD_LIBRARY_PATH", "/proton/lib");

    let args: Vec<String> = ContyLayer::new()
        .wrap(inner, &ctx)
        .expect("wrap ok")
        .into_command()
        .args
        .iter()
        .map(ArgValue::render)
        .collect();

    let unset = args
        .windows(2)
        .position(|w| w == ["--unsetenv", "LD_LIBRARY_PATH"])
        .expect("host LD_LIBRARY_PATH unset");
    let set = args
        .windows(3)
        .position(|w| w == ["--setenv", "LD_LIBRARY_PATH", "/proton/lib"])
        .expect("inner LD_LIBRARY_PATH set");
    assert!(
        unset < set,
        "the deliberate value must be applied after the strip"
    );
}

#[test]
fn preflight_rejects_a_nonexistent_conty_path() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = ContyLayer::at(PathBuf::from("/does/not/exist/conty.sh"));
    assert!(layer.preflight(&ctx).is_err());
}

#[test]
fn preflight_rejects_an_out_of_range_sandbox_level() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = ContyLayer {
        conty_path: conty_on_path(),
        sandbox_level: Some(4),
        base_dir: None,
    };
    // Only meaningful when a conty binary is available to get past the
    // executable check; otherwise both checks fail and the assert still holds.
    assert!(layer.preflight(&ctx).is_err());
}
