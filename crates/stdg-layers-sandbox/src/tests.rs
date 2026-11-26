//! Tests for the Conty layer; the end-to-end one needs Conty on `PATH`.

use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use stdg_core::{
    ArgValue, BindMode, BindPurpose, Binding, CommandSpec, GameId, LaunchCtx, Layer, ModeId,
    PathValue, Plan, ResolvedConfig, RunKind, RunnerId, SessionId, SessionInfo, TargetKind,
};

use crate::{
    CONTY_BINARY_NAMES, ContyLayer, InvalidUserNsFlag, SandboxLevel, UserNs, is_setuid_root,
    parse_userns_flag,
};

fn test_ctx(root: PathBuf) -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("test-game".to_string()),
        target: TargetKind::NativeLinux,
        root,
        executable: PathBuf::from("run.sh"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
        saves: stdg_core::SavesConfig::default(),
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
            tmp_dir: PathBuf::from("/nonexistent/session"),
        },
        bindings: Vec::new(),
        run_kind: RunKind::Live,
        wine_prefix: None,
    }
}

fn conty_on_path() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .flat_map(|dir| CONTY_BINARY_NAMES.iter().map(move |n| dir.join(n)))
        .find(|p| p.is_file())
}

#[test]
#[ignore = "needs a real Conty on PATH; run with `cargo test -- --ignored`"]
fn actually_runs_a_command_through_conty() {
    let Some(_conty) = conty_on_path() else {
        eprintln!("skipping: no conty executable found on PATH");
        return;
    };

    let tmp_dir = tempfile::tempdir().expect("create test dir");
    let tmp = tmp_dir.path().to_path_buf();
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
        sandbox_level: Some(SandboxLevel::Processes),
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
        spec.env
            .get("SANDBOX")
            .map(stdg_core::EnvValue::render)
            .as_deref(),
        Some("1")
    );
    assert_eq!(
        spec.env
            .get("SANDBOX_LEVEL")
            .map(stdg_core::EnvValue::render)
            .as_deref(),
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
fn userns_runs_conty_through_unshare_and_maps_the_user_back() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = ContyLayer {
        userns: Some(UserNs {
            uid: 1000,
            gid: 100,
        }),
        ..ContyLayer::at(PathBuf::from("/opt/conty.sh"))
    };
    let spec = layer
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))),
            &ctx,
        )
        .expect("wrap ok")
        .into_command();

    let program = spec.program.as_ref().map(|p| p.effective().to_path_buf());
    assert!(program.is_some_and(|p| p.ends_with("unshare")));
    let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
    assert_eq!(
        args[..9],
        [
            "--map-root-user",
            "--mount",
            "--",
            "/opt/conty.sh",
            "--unshare-user",
            "--uid",
            "1000",
            "--gid",
            "100"
        ]
    );
    assert_eq!(
        spec.env
            .get("ALLOW_ROOT")
            .map(stdg_core::EnvValue::render)
            .as_deref(),
        Some("1")
    );
}

#[test]
fn inner_command_records_its_exit_code() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let spec = ContyLayer::new()
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/game/bin"))),
            &ctx,
        )
        .expect("wrap ok")
        .into_command();

    let status_file = ctx.session.tmp_dir.join("conty/status");
    assert_eq!(spec.status_file.as_deref(), Some(status_file.as_path()));
    let args: Vec<String> = spec.args.iter().map(ArgValue::render).collect();
    let sep = args.iter().position(|a| a == "--").expect("-- separator");
    assert_eq!(
        args[sep + 1..],
        [
            "/bin/sh",
            "-c",
            r#""$@"; echo $? > "$0""#,
            &status_file.display().to_string(),
            "/game/bin"
        ]
    );
}

#[test]
fn preflight_rejects_a_nonexistent_conty_path() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let layer = ContyLayer::at(PathBuf::from("/does/not/exist/conty.sh"));
    assert!(layer.preflight(&ctx).is_err());
}

#[test]
fn sandbox_level_parses_only_conty_levels() {
    assert_eq!("1".parse(), Ok(SandboxLevel::UserFiles));
    assert_eq!(" 3 ".parse(), Ok(SandboxLevel::Network));
    assert!("0".parse::<SandboxLevel>().is_err());
    assert!("4".parse::<SandboxLevel>().is_err());
    assert!("high".parse::<SandboxLevel>().is_err());
}

#[test]
fn preflight_accepts_an_existing_conty_path() {
    let ctx = test_ctx(PathBuf::from("/nonexistent-game-root"));
    let fake = tempfile::NamedTempFile::new().expect("temp file");
    let layer = ContyLayer::at(fake.path().to_path_buf());
    assert!(layer.preflight(&ctx).is_ok());
}

#[test]
fn root_overlay_replaces_the_plain_root_bind() {
    let mut ctx = test_ctx(PathBuf::from("/games/g"));
    ctx.bindings.push(Binding {
        source: PathValue::Translated {
            host: PathBuf::from("/fix"),
            guest: PathBuf::from("/games/g"),
        },
        mode: BindMode::OverlayLower,
        purpose: BindPurpose("fix".to_string()),
    });
    ctx.bindings.push(root_upper("/state/upper"));
    ctx.bindings.push(Binding {
        source: PathValue::Translated {
            host: PathBuf::from("/emu/steam_api64.dll"),
            guest: PathBuf::from("/games/g/steam_api64.dll"),
        },
        mode: BindMode::ReadOnly,
        purpose: BindPurpose("dll".to_string()),
    });

    let args: Vec<String> = ContyLayer::new()
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/games/g/x"))),
            &ctx,
        )
        .expect("wrap ok")
        .into_command()
        .args
        .iter()
        .map(ArgValue::render)
        .collect();

    assert!(
        !args
            .windows(3)
            .any(|w| w == ["--bind", "/games/g", "/games/g"])
    );
    let overlay = args
        .windows(9)
        .position(|w| {
            w == [
                "--overlay-src",
                "/games/g",
                "--overlay-src",
                "/fix",
                "--overlay",
                "/state/upper",
                "/state/work",
                "/games/g",
                "--ro-bind",
            ]
        })
        .expect("overlay (game first, fix on top), then the lib bind");
    assert!(overlay < args.iter().position(|a| a == "--").expect("separator"));
}

#[test]
fn userns_flag_accepts_numbers_and_words() {
    assert_eq!(parse_userns_flag("1"), Ok(true));
    assert_eq!(parse_userns_flag("true"), Ok(true));
    assert_eq!(parse_userns_flag(" 0 "), Ok(false));
    assert_eq!(parse_userns_flag("false"), Ok(false));
}

#[test]
fn userns_flag_rejects_other_values() {
    assert_eq!(
        parse_userns_flag("yes"),
        Err(InvalidUserNsFlag("yes".to_string()))
    );
}

#[test]
fn plain_or_missing_file_is_not_setuid_root() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("fusermount3");
    std::fs::write(&file, "").expect("write");
    // Setuid bit set, but owned by this user rather than root.
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o4755)).expect("chmod");
    assert!(!is_setuid_root(&file));
    assert!(!is_setuid_root(&dir.path().join("missing")));
}

fn root_upper(upper: &str) -> Binding {
    Binding {
        source: PathValue::Host(PathBuf::from("/games/g")),
        mode: BindMode::Overlay {
            upper: PathBuf::from(upper),
            work: PathBuf::from("/state/work"),
        },
        purpose: BindPurpose("saves-root".to_string()),
    }
}

fn wrap_args(ctx: &LaunchCtx) -> Result<Vec<String>, stdg_core::CoreError> {
    Ok(ContyLayer::new()
        .wrap(
            CommandSpec::new(PathValue::Host(PathBuf::from("/games/g/x"))),
            ctx,
        )?
        .into_command()
        .args
        .iter()
        .map(ArgValue::render)
        .collect())
}

#[test]
fn lowers_without_an_upper_mount_read_only() {
    let mut ctx = test_ctx(PathBuf::from("/games/g"));
    ctx.bindings.push(Binding {
        source: PathValue::Translated {
            host: PathBuf::from("/fix"),
            guest: PathBuf::from("/games/g"),
        },
        mode: BindMode::OverlayLower,
        purpose: BindPurpose("fix".to_string()),
    });
    let args = wrap_args(&ctx).expect("wrap ok");
    assert!(args.windows(6).any(|w| w
        == [
            "--overlay-src",
            "/games/g",
            "--overlay-src",
            "/fix",
            "--ro-overlay",
            "/games/g"
        ]));
}

#[test]
fn two_writable_overlays_on_one_dir_are_an_error() {
    let mut ctx = test_ctx(PathBuf::from("/games/g"));
    ctx.bindings.push(root_upper("/a"));
    ctx.bindings.push(root_upper("/b"));
    assert!(wrap_args(&ctx).is_err());
}
