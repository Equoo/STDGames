use stdg_core::{CoreError, LayerRef, ModeId, PartialGameConfig, TargetKind};

use super::fixtures::*;
use crate::build_plan;
use crate::cascade::RunnerDefaults;

fn plan_for(game: &PartialGameConfig, mode: &str) -> Result<stdg_core::Plan, CoreError> {
    build_plan(
        &game_id(),
        &mode_id(mode),
        &default_globals(),
        &RunnerDefaults::default(),
        game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
}

#[test]
fn two_layers_in_the_same_slot_is_rejected() {
    let game = with_mode(
        base_game(TargetKind::Windows),
        "desktop",
        mode_with_layers(&[LayerRef::new("proton"), LayerRef::new("wine")]),
    );

    let err = plan_for(&game, "desktop").expect_err("must reject two Compat layers");
    match err {
        CoreError::SlotConflict { slot, .. } => assert_eq!(slot, stdg_core::Slot::Compat),
        other => panic!("expected SlotConflict, got {other:?}"),
    }
}

#[test]
fn missing_capability_is_rejected() {
    // steamapi-emu requires WINDOWS_ABI, but no Compat layer is present.
    let game = with_mode(
        base_game(TargetKind::Windows),
        "desktop",
        mode_with_layers(&[LayerRef::new("steamapi-emu")]),
    );

    let err = plan_for(&game, "desktop").expect_err("must reject missing capability");
    match err {
        CoreError::MissingCapability { layer, .. } => assert_eq!(layer.0, "steamapi-emu"),
        other => panic!("expected MissingCapability, got {other:?}"),
    }
}

#[test]
fn dll_injection_on_a_native_linux_game_is_rejected() {
    // A native Linux target has no Compat layer to provide WINDOWS_ABI, so
    // the DLL-swap SteamApi variant can never be satisfied for this target.
    let game = with_mode(
        base_game(TargetKind::NativeLinux),
        "desktop-injected",
        mode_with_layers(&[LayerRef::new("steamapi-emu")]),
    );

    let err = plan_for(&game, "desktop-injected").expect_err("must reject DLL injection on native Linux");
    assert!(matches!(err, CoreError::MissingCapability { .. }));
}

#[test]
fn undeclared_mode_is_rejected() {
    let game = base_game(TargetKind::NativeLinux);
    let err = plan_for(&game, "does-not-exist").expect_err("must reject undeclared mode");
    assert_eq!(err.to_string(), CoreError::ModeDisabled(ModeId("does-not-exist".to_string())).to_string());
}

#[test]
fn explicitly_disabled_mode_is_rejected() {
    let mut mode = mode_with_layers(&[]);
    mode.enabled = Some(false);
    let game = with_mode(base_game(TargetKind::NativeLinux), "desktop", mode);

    let err = plan_for(&game, "desktop").expect_err("must reject disabled mode");
    assert!(matches!(err, CoreError::ModeDisabled(_)));
}

#[test]
fn missing_required_field_is_rejected() {
    let mut game = base_game(TargetKind::NativeLinux);
    game.executable = None;
    let game = with_mode(game, "desktop", mode_with_layers(&[]));

    let err = plan_for(&game, "desktop").expect_err("must reject missing executable");
    assert!(matches!(err, CoreError::Config(_)));
}

#[test]
fn mode_without_a_sandbox_layer_anywhere_is_rejected() {
    // No bwrap in global defaults, runner defaults, or the mode itself.
    let game = with_mode(base_game(TargetKind::NativeLinux), "desktop", mode_with_layers(&[]));

    let err = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &crate::cascade::GlobalDefaults::default(),
        &RunnerDefaults::default(),
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect_err("must reject a plan with no Sandbox layer in any tier");

    assert!(matches!(err, CoreError::MissingMandatorySlot(stdg_core::Slot::Sandbox)));
}

#[test]
fn unknown_layer_id_is_rejected() {
    let game = with_mode(
        base_game(TargetKind::NativeLinux),
        "desktop",
        mode_with_layers(&[LayerRef::new("does-not-exist")]),
    );

    let err = plan_for(&game, "desktop").expect_err("must reject unknown layer id");
    assert!(matches!(err, CoreError::UnknownLayer(_)));
}
