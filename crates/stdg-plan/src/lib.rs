//! The planner: turns `(game config, mode)` into a `Plan`.

mod cascade;
mod validate;

use std::collections::BTreeMap;

use stdg_core::{
    CoreError, GameId, LayerRef, ModeId, PartialGameConfig, Plan, RunnerCatalog, Slot,
};

pub use cascade::{GlobalDefaults, ModeLayerDefaults, RunnerDefaults, resolve_config};
pub use validate::{assign_slots, check_capabilities, check_mandatory_slots, validate_plan};

/// The final layer assignment: at most one layer per slot.
pub type SlotMap = BTreeMap<Slot, LayerRef>;

/// Builds a `Plan` for `game_id` in `mode_id`.
pub fn build_plan(
    game_id: &GameId,
    mode_id: &ModeId,
    global: &GlobalDefaults,
    runner_defaults: &RunnerDefaults,
    game: &PartialGameConfig,
    catalog: &dyn stdg_core::LayerCatalog,
    runners: &dyn RunnerCatalog,
) -> Result<Plan, CoreError> {
    let mode = match game.modes.as_ref().and_then(|modes| modes.get(mode_id)) {
        Some(mode) if mode.enabled != Some(false) => mode.clone(),
        _ => return Err(CoreError::ModeDisabled(mode_id.clone())),
    };

    let config = resolve_config(game_id, global, runner_defaults, game, &mode)?;

    let runner = runners
        .find_for_target(&config.target)
        .ok_or_else(|| CoreError::NoRunnerForTarget(config.target.clone()))?;

    // Later tiers override earlier ones per slot.
    let tiers: [Vec<LayerRef>; 3] = [
        global.baseline_layers.clone(),
        runner_defaults.layers_for(mode_id),
        mode.layers.unwrap_or_default(),
    ];
    let slots = validate_plan(&config, &tiers, catalog)?;

    Ok(Plan {
        game_id: game_id.clone(),
        mode_id: mode_id.clone(),
        target: config.target.clone(),
        runner: runner.id(),
        slots,
        config,
    })
}

#[cfg(test)]
mod tests;
