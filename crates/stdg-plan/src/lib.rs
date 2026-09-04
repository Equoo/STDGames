//! The planner: turns `(game config, mode)` into a `Plan`.
//!
//! Pure and testable without spawning anything: no process execution, no
//! filesystem side effects beyond what the caller already loaded into
//! memory. `stdg-plan` depends only on `stdg-core`'s traits, never on a
//! concrete runner/layer implementation crate.

mod cascade;
mod validate;

use std::collections::BTreeMap;

use stdg_core::{CoreError, GameId, LayerRef, ModeId, PartialGameConfig, Plan, RunnerCatalog, Slot};

pub use cascade::{resolve_config, GlobalDefaults, ModeLayerDefaults, RunnerDefaults};
pub use validate::{assign_slots, check_capabilities, check_mandatory_slots, validate_plan};

pub(crate) type SlotMap = BTreeMap<Slot, LayerRef>;

/// Builds a `Plan` for `game_id` in `mode_id`.
///
/// `global`, `runner_defaults`, `game`, and the mode's own override are
/// merged in cascade order (see [`resolve_config`]) into a `ResolvedConfig`.
/// Layers are resolved separately, per slot, from three tiers of
/// increasing precedence — global baseline, this runner's default for this
/// mode, then the game's own override — and checked against the catalogs
/// (see [`validate_plan`]) before the `Plan` is assembled.
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

    // Increasing precedence: global baseline, then this runner's default for
    // this mode, then the game's own override — each later tier overrides
    // the earlier ones on a per-slot basis (see `validate::assign_slots`).
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
