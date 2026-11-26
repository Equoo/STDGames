use stdg_core::{LayerRef, Slot, TargetKind};

use super::fixtures::*;
use crate::build_plan;
use crate::cascade::RunnerDefaults;

#[test]
fn native_linux_mode_with_no_extra_layers_still_gets_the_mandatory_sandbox() {
    let game = with_mode(
        base_game(TargetKind::NativeLinux),
        "desktop",
        mode_with_layers(&[]),
    );

    let plan = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &default_globals(),
        &RunnerDefaults::default(),
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build");

    // Sandbox comes from the global baseline, Saves is always there.
    assert_eq!(plan.slots.len(), 2);
    assert!(plan.slots.contains_key(&Slot::Sandbox));
    assert_eq!(
        plan.slots.get(&Slot::Saves).map(|r| r.id.0.as_str()),
        Some("saves")
    );
    assert_eq!(plan.runner.0, "native-linux");
    assert_eq!(plan.target, TargetKind::NativeLinux);
}

#[test]
fn windows_desktop_mode_assigns_compat_slot() {
    let game = with_mode(
        base_game(TargetKind::Windows),
        "desktop",
        mode_with_layers(&[LayerRef::new("proton").with_param("version", "9.0-4")]),
    );

    let plan = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &default_globals(),
        &RunnerDefaults::default(),
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build");

    assert_eq!(plan.slots.len(), 3);
    let compat = plan.slots.get(&Slot::Compat).expect("compat slot set");
    assert_eq!(compat.id.0, "proton");
    assert_eq!(compat.param("version"), Some("9.0-4"));
}

#[test]
fn steam_injected_mode_assigns_compat_and_steamapi_slots() {
    // Only the `SteamApi` slot differs.
    let game = with_mode(
        base_game(TargetKind::Windows),
        "steam-injected",
        mode_with_layers(&[LayerRef::new("proton"), LayerRef::new("steamapi-emu")]),
    );

    let plan = build_plan(
        &game_id(),
        &mode_id("steam-injected"),
        &default_globals(),
        &RunnerDefaults::default(),
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build");

    assert_eq!(plan.slots.len(), 4);
    assert!(plan.slots.contains_key(&Slot::Compat));
    assert!(plan.slots.contains_key(&Slot::SteamApi));
    assert!(plan.slots.contains_key(&Slot::Sandbox));
}

#[test]
fn layers_inside_out_is_ordered_innermost_to_outermost() {
    let game = with_mode(
        base_game(TargetKind::Windows),
        "steam-injected",
        mode_with_layers(&[
            LayerRef::new("proton"),
            LayerRef::new("steamapi-emu"),
            LayerRef::new("cgroup"),
        ]),
    );

    let plan = build_plan(
        &game_id(),
        &mode_id("steam-injected"),
        &default_globals(),
        &RunnerDefaults::default(),
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build");

    // Innermost first: Supervision, SteamApi, Compat, Saves, Sandbox.
    let order: Vec<Slot> = plan.layers_inside_out().map(|(slot, _)| slot).collect();
    assert_eq!(
        order,
        vec![
            Slot::Supervision,
            Slot::SteamApi,
            Slot::Compat,
            Slot::Saves,
            Slot::Sandbox
        ]
    );
}
