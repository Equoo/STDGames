//! Covers the per-slot override merge across the three layer tiers: global
//! baseline, runner defaults for the mode, and the game's own override.

use std::collections::BTreeMap;

use stdg_core::{LayerRef, Slot, TargetKind};

use super::fixtures::*;
use crate::build_plan;
use crate::cascade::{GlobalDefaults, ModeLayerDefaults, RunnerDefaults};

fn runner_defaults_for(mode: &str, layers: &[LayerRef]) -> RunnerDefaults {
    let mut modes = BTreeMap::new();
    modes.insert(
        mode_id(mode),
        ModeLayerDefaults {
            layers: layers.to_vec(),
        },
    );
    RunnerDefaults {
        modes,
        ..RunnerDefaults::default()
    }
}

#[test]
fn runner_default_layer_applies_when_mode_has_no_override() {
    // The game only opts into "desktop"; it never mentions a Compat layer.
    let game = with_mode(base_game(TargetKind::Windows), "desktop", mode_with_layers(&[]));
    let runner_defaults = runner_defaults_for("desktop", &[LayerRef::new("proton").with_param("version", "9.0-4")]);

    let plan = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &default_globals(),
        &runner_defaults,
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build from the runner default alone");

    let compat = plan.slots.get(&Slot::Compat).expect("compat slot set from runner default");
    assert_eq!(compat.id.0, "proton");
    assert_eq!(compat.param("version"), Some("9.0-4"));
}

#[test]
fn game_mode_override_replaces_runner_default_for_same_slot() {
    let game = with_mode(
        base_game(TargetKind::Windows),
        "desktop",
        mode_with_layers(&[LayerRef::new("proton").with_param("version", "staging-8.0")]),
    );
    let runner_defaults = runner_defaults_for("desktop", &[LayerRef::new("proton").with_param("version", "9.0-4")]);

    let plan = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &default_globals(),
        &runner_defaults,
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build");

    // The game's own entry for the Compat slot wins over the runner default,
    // instead of conflicting with it or being silently ignored.
    let compat = plan.slots.get(&Slot::Compat).expect("compat slot set");
    assert_eq!(compat.param("version"), Some("staging-8.0"));
}

#[test]
fn global_baseline_layer_applies_alongside_runner_and_game_layers() {
    let game = with_mode(base_game(TargetKind::Windows), "desktop", mode_with_layers(&[]));
    let global = GlobalDefaults {
        baseline_layers: vec![LayerRef::new("bwrap"), LayerRef::new("cgroup")],
        ..GlobalDefaults::default()
    };
    let runner_defaults = runner_defaults_for("desktop", &[LayerRef::new("proton")]);

    let plan = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &global,
        &runner_defaults,
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect("plan should build");

    assert!(plan.slots.contains_key(&Slot::Sandbox), "global baseline layer should be present");
    assert!(plan.slots.contains_key(&Slot::Supervision), "global baseline layer should be present");
    assert!(plan.slots.contains_key(&Slot::Compat), "runner default layer should be present");
}

#[test]
fn duplicate_slot_within_runner_defaults_tier_is_rejected() {
    // Two Compat layers in the *same* tier is still a config mistake,
    // regardless of which tier it happens in.
    let game = with_mode(base_game(TargetKind::Windows), "desktop", mode_with_layers(&[]));
    let runner_defaults = runner_defaults_for("desktop", &[LayerRef::new("proton"), LayerRef::new("wine")]);

    let err = build_plan(
        &game_id(),
        &mode_id("desktop"),
        &GlobalDefaults::default(),
        &runner_defaults,
        &game,
        &FakeCatalog,
        &FakeRunnerCatalog::new(),
    )
    .expect_err("must reject two Compat layers within the runner-defaults tier");

    assert!(matches!(err, stdg_core::CoreError::SlotConflict { slot: Slot::Compat, .. }));
}
