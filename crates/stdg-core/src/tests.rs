use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::capability::capabilities::{SANDBOXED, STEAM_HANDSHAKE, WINDOWS_ABI};
use crate::{
    ArgValue, CapabilitySet, CommandSpec, EnvValue, GameId, LayerRef, ModeId, Outcome,
    PathListSeparator, PathValue, Plan, ResolvedConfig, RunnerId, Slot, TargetKind,
};

fn translated() -> PathValue {
    PathValue::Translated {
        host: PathBuf::from("/host/lib.so"),
        guest: PathBuf::from("/guest/lib.so"),
    }
}

#[test]
fn path_value_host_and_effective_pick_the_right_side() {
    let host_only = PathValue::Host(PathBuf::from("/a"));
    assert_eq!(host_only.host(), host_only.effective());

    let t = translated();
    assert_eq!(t.host(), PathBuf::from("/host/lib.so"));
    assert_eq!(t.effective(), PathBuf::from("/guest/lib.so"));
}

#[test]
fn capability_set_reports_requirements_missing_from_provided() {
    let required = CapabilitySet::of([WINDOWS_ABI, SANDBOXED]);
    let provided = CapabilitySet::of([SANDBOXED, STEAM_HANDSHAKE]);
    let missing = required.missing_from(&provided);
    assert_eq!(missing.iter().collect::<Vec<_>>(), vec![WINDOWS_ABI]);
    assert!(required.missing_from(&required).is_empty());
}

#[test]
fn capability_set_union_and_insert() {
    let mut a = CapabilitySet::new();
    assert!(a.is_empty());
    a.insert(SANDBOXED);
    let b = CapabilitySet::of([WINDOWS_ABI]);
    let both = a.union(&b);
    assert!(both.contains(SANDBOXED) && both.contains(WINDOWS_ABI));
    assert!(!a.contains(WINDOWS_ABI));
}

#[test]
fn arg_and_env_values_render_the_guest_side_of_paths() {
    assert_eq!(ArgValue::Literal("x".into()).render(), "x");
    assert_eq!(ArgValue::Path(translated()).render(), "/guest/lib.so");
    assert_eq!(EnvValue::Path(translated()).render(), "/guest/lib.so");
    let list = EnvValue::PathList(
        vec![translated(), PathValue::Host(PathBuf::from("/b"))],
        PathListSeparator(':'),
    );
    assert_eq!(list.render(), "/guest/lib.so:/b");
}

#[test]
fn wrapped_by_moves_the_old_program_into_the_arguments() {
    let mut inner = CommandSpec::new(PathValue::Host(PathBuf::from("/game")));
    inner.push_arg_literal("--flag");
    inner.set_env_literal("K", "V");
    inner.cwd = Some(PathValue::Host(PathBuf::from("/cwd")));

    let outer = inner.wrapped_by(PathValue::Host(PathBuf::from("/wrapper")));

    assert_eq!(
        outer.program,
        Some(PathValue::Host(PathBuf::from("/wrapper")))
    );
    let args: Vec<String> = outer.args.iter().map(ArgValue::render).collect();
    assert_eq!(args, vec!["/game", "--flag"]);
    assert_eq!(
        outer.env.get("K").map(EnvValue::render).as_deref(),
        Some("V")
    );
    assert_eq!(outer.cwd, Some(PathValue::Host(PathBuf::from("/cwd"))));
}

#[test]
fn command_spec_setters_replace_previous_env_values() {
    let mut spec = CommandSpec::new(PathValue::Host(PathBuf::from("/p")));
    spec.set_env_literal("K", "old");
    spec.set_env_path("K", PathValue::Host(PathBuf::from("/new")));
    assert_eq!(
        spec.env.get("K").map(EnvValue::render).as_deref(),
        Some("/new")
    );
    spec.push_arg(ArgValue::Literal("a".into()));
    spec.push_arg_path(PathValue::Host(PathBuf::from("/b")));
    assert_eq!(spec.args.len(), 2);
}

#[test]
fn layer_ref_params_round_trip() {
    let r = LayerRef::new("proton").with_param("version", "9");
    assert_eq!(r.id.0, "proton");
    assert_eq!(r.param("version"), Some("9"));
    assert_eq!(r.param("missing"), None);
}

#[test]
fn application_order_is_the_reverse_of_wrapping_order() {
    let order: Vec<Slot> = Slot::application_order().collect();
    assert_eq!(order.first(), Some(&Slot::Tooling));
    assert_eq!(order.last(), Some(&Slot::Sandbox));
    assert_eq!(order.len(), Slot::ORDER.len());
}

#[test]
fn plan_lists_only_filled_slots_innermost_first() {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("g".into()),
        target: TargetKind::Windows,
        root: PathBuf::from("/g"),
        executable: PathBuf::from("g.exe"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
        saves: crate::SavesConfig::default(),
    };
    let mut slots = BTreeMap::new();
    slots.insert(Slot::Sandbox, LayerRef::new("conty"));
    slots.insert(Slot::Compat, LayerRef::new("proton"));
    let plan = Plan {
        game_id: config.game_id.clone(),
        mode_id: ModeId("m".into()),
        target: config.target.clone(),
        runner: RunnerId("windows".into()),
        slots,
        config,
    };
    let ids: Vec<&str> = plan
        .layers_inside_out()
        .map(|(_, r)| r.id.0.as_str())
        .collect();
    assert_eq!(ids, vec!["proton", "conty"]);
}

#[test]
fn outcome_direct_yields_its_command() {
    let spec = CommandSpec::new(PathValue::Host(PathBuf::from("/p")));
    let out = Outcome::Direct(spec.clone()).into_command();
    assert_eq!(out.program, spec.program);
}

#[test]
fn ids_display_their_inner_string() {
    assert_eq!(GameId("g".into()).to_string(), "g");
    assert_eq!(ModeId("m".into()).to_string(), "m");
    assert_eq!(RunnerId("r".into()).to_string(), "r");
    assert_eq!(crate::LayerId("l".into()).to_string(), "l");
    assert_eq!(crate::SessionId("s".into()).to_string(), "s");
}
