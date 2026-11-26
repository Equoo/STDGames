use std::collections::BTreeMap;
use std::path::PathBuf;

use stdg_core::{
    BindMode, BindPurpose, Binding, CommandSpec, CoreError, Diagnostic, GameId, LaunchCtx, Layer,
    LayerId, LayerRef, ModeId, Outcome, PathValue, Plan, ResolvedConfig, RunKind, Runner, RunnerId,
    SessionGuard, SessionId, SessionInfo, Slot, TargetKind,
};
use stdg_registry::Registry;

use crate::cgroup::CgroupSession;
use crate::{
    DisplayCommand, ExecError, SessionTmpDir, new_session_info, run_pipeline, spawn_and_wait,
    to_std_command,
};

#[derive(Debug)]
struct EchoRunner;
impl Runner for EchoRunner {
    fn id(&self) -> RunnerId {
        RunnerId("echo".to_string())
    }
    fn accepts(&self, _target: &TargetKind) -> bool {
        true
    }
    fn build(&self, _ctx: &LaunchCtx) -> Result<CommandSpec, CoreError> {
        let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from("/bin/true")));
        spec.push_arg_literal("game");
        Ok(spec)
    }
}

/// Wraps the command in `wrapper`, and records a binding.
#[derive(Debug)]
struct WrapLayer {
    preflight: Option<Diagnostic>,
}
impl Layer for WrapLayer {
    fn id(&self) -> LayerId {
        LayerId("wrap".to_string())
    }
    fn slot(&self) -> Slot {
        Slot::Compat
    }
    fn preflight(&self, _ctx: &LaunchCtx) -> Result<(), Diagnostic> {
        self.preflight.clone().map_or(Ok(()), Err)
    }
    fn container_needs(&self, _ctx: &LaunchCtx) -> Vec<Binding> {
        vec![Binding {
            source: PathValue::Host(PathBuf::from("/opt/wrapper")),
            mode: BindMode::ReadOnly,
            purpose: BindPurpose("wrapper".to_string()),
        }]
    }
    fn wrap(&self, inner: CommandSpec, _ctx: &LaunchCtx) -> Result<Outcome, CoreError> {
        Ok(Outcome::Direct(inner.wrapped_by(PathValue::Host(
            PathBuf::from("/opt/wrapper"),
        ))))
    }
}

fn registry(preflight: Option<Diagnostic>) -> Registry {
    let mut registry = Registry::new();
    registry.register_runner(Box::new(EchoRunner));
    registry.register_layer(
        LayerId("wrap".to_string()),
        Box::new(move |_, _| {
            Ok(Box::new(WrapLayer {
                preflight: preflight.clone(),
            }))
        }),
    );
    registry
}

fn ctx() -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("g".to_string()),
        target: TargetKind::NativeLinux,
        root: PathBuf::from("/games/g"),
        executable: PathBuf::from("g"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
        saves: stdg_core::SavesConfig::default(),
    };
    let mut slots = BTreeMap::new();
    slots.insert(Slot::Compat, LayerRef::new("wrap"));
    LaunchCtx {
        plan: Plan {
            game_id: config.game_id.clone(),
            mode_id: ModeId("m".to_string()),
            target: config.target.clone(),
            runner: RunnerId("echo".to_string()),
            slots,
            config,
        },
        session: SessionInfo {
            id: SessionId("s".to_string()),
            tmp_dir: PathBuf::from("/nonexistent"),
        },
        bindings: Vec::new(),
        run_kind: RunKind::DryRun,
        wine_prefix: None,
    }
}

#[test]
fn pipeline_wraps_the_runner_command_and_collects_bindings() {
    let output = run_pipeline(&registry(None), ctx()).expect("pipeline succeeds");
    assert_eq!(output.bindings.len(), 1);
    assert_eq!(output.guards.len(), 1);
    let spec = output.outcome.into_command();
    assert_eq!(
        DisplayCommand(&spec).to_string(),
        "/opt/wrapper /bin/true game"
    );
}

#[test]
fn pipeline_stops_on_a_preflight_error() {
    let err = run_pipeline(&registry(Some(Diagnostic::error("broken"))), ctx())
        .expect_err("preflight error aborts");
    assert!(matches!(err, ExecError::Core(CoreError::Preflight { .. })));
}

#[test]
fn pipeline_continues_past_a_preflight_warning() {
    let output = run_pipeline(&registry(Some(Diagnostic::warning("meh"))), ctx())
        .expect("warnings do not abort");
    assert_eq!(output.bindings.len(), 1);
}

#[test]
fn pipeline_rejects_an_unknown_runner() {
    let mut ctx = ctx();
    ctx.plan.runner = RunnerId("missing".to_string());
    let err = run_pipeline(&registry(None), ctx).expect_err("unknown runner");
    assert!(matches!(err, ExecError::Core(CoreError::UnknownRunner(_))));
}

#[test]
fn to_std_command_applies_args_env_and_cwd() {
    let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from("/bin/echo")));
    spec.push_arg_literal("hi");
    spec.set_env_literal("K", "V");
    spec.cwd = Some(PathValue::Host(PathBuf::from("/tmp")));
    let cmd = to_std_command(&spec).expect("program is set");
    assert_eq!(cmd.get_program(), "/bin/echo");
    assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec!["hi"]);
    assert_eq!(cmd.get_current_dir(), Some(PathBuf::from("/tmp").as_path()));
    assert!(
        cmd.get_envs()
            .any(|(k, v)| k == "K" && v.is_some_and(|v| v == "V"))
    );
}

#[test]
fn to_std_command_rejects_a_spec_without_program() {
    let err = to_std_command(&CommandSpec::default()).expect_err("no program");
    assert!(matches!(err, ExecError::MissingProgram));
}

#[test]
fn spawn_and_wait_reports_the_exit_status() {
    let spec = CommandSpec::new(PathValue::Host(PathBuf::from("/bin/true")));
    let status = spawn_and_wait(&spec, &[]).expect("/bin/true runs");
    assert!(status.success());
}

#[test]
fn spawn_and_wait_names_a_missing_program() {
    let spec = CommandSpec::new(PathValue::Host(PathBuf::from("/definitely/not/here")));
    let err = spawn_and_wait(&spec, &[]).expect_err("program does not exist");
    assert!(
        matches!(err, ExecError::Spawn { program, .. } if program.as_path() == std::path::Path::new("/definitely/not/here"))
    );
}

#[test]
fn session_ids_are_unique_and_tmp_dirs_live_under_the_base() {
    let base = tempfile::tempdir().expect("tempdir");
    let a = new_session_info(base.path());
    let b = new_session_info(base.path());
    assert_ne!(a.id, b.id);
    assert!(a.tmp_dir.starts_with(base.path()));
}

#[test]
fn session_tmp_dir_is_removed_on_drop() {
    let base = tempfile::tempdir().expect("tempdir");
    let path = {
        let dir = SessionTmpDir::create(base.path(), "scratch").expect("create");
        assert!(dir.path().is_dir());
        assert_eq!(dir.label(), "session-tmp-dir");
        dir.path().to_path_buf()
    };
    assert!(!path.exists());
}

#[test]
fn cgroup_session_is_none_outside_a_delegated_cgroup() {
    let plain_dir = tempfile::tempdir().expect("tempdir");
    let session = CgroupSession::create(plain_dir.path(), "s").expect("no I/O error");
    assert!(session.is_none());
}

#[test]
fn child_subreaper_flag_can_be_set_and_read_back() {
    crate::subreaper::set_child_subreaper().expect("prctl set");
    assert!(crate::subreaper::is_child_subreaper().expect("prctl get"));
}

#[test]
fn status_file_overrides_the_wrapper_exit_code() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("status");
    let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from("/bin/sh")));
    spec.push_arg_literal("-c");
    spec.push_arg_literal(format!("echo 7 > {}", file.display()));
    spec.status_file = Some(file);
    let status = crate::spawn_and_wait(&spec, &[]).expect("runs");
    assert_eq!(status.code(), Some(7));
}

#[test]
fn missing_status_file_means_the_command_never_ran() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from("/bin/true")));
    spec.status_file = Some(dir.path().join("status"));
    let status = crate::spawn_and_wait(&spec, &[]).expect("runs");
    assert_eq!(status.code(), Some(127));
}

#[test]
fn killed_cgroup_processes_are_waited_for() {
    let parent = PathBuf::from("/sys/fs/cgroup").join(
        std::fs::read_to_string("/proc/self/cgroup")
            .unwrap_or_default()
            .trim()
            .trim_start_matches("0::/"),
    );
    let Ok(Some(cgroup)) = CgroupSession::create(&parent, &format!("test-{}", std::process::id()))
    else {
        return;
    };
    let mut child = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("sleep spawns");
    let added = cgroup.add_pid(child.id()).is_ok();
    if added {
        cgroup.kill_all().expect("kill");
    } else {
        let _ = child.kill();
    }
    let _ = child.wait();
    assert!(cgroup.wait_empty(std::time::Duration::from_secs(5)));
}
