//! The planner's output.

use std::collections::BTreeMap;

use crate::{
    config::ResolvedConfig,
    ids::{GameId, ModeId, RunnerId},
    layer::LayerRef,
    slot::Slot,
    target::TargetKind,
};

/// A validated, fully resolved launch plan.
#[derive(Debug, Clone)]
pub struct Plan {
    /// The game being launched.
    pub game_id: GameId,
    /// The mode it is launched in.
    pub mode_id: ModeId,
    /// The game's target kind.
    pub target: TargetKind,
    /// The runner building the base command.
    pub runner: RunnerId,
    /// At most one layer per slot.
    pub slots: BTreeMap<Slot, LayerRef>,
    /// The fully resolved configuration.
    pub config: ResolvedConfig,
}

impl Plan {
    /// Layers in actual application order (innermost to outermost).
    pub fn layers_inside_out(&self) -> impl Iterator<Item = (Slot, &LayerRef)> {
        Slot::application_order().filter_map(move |slot| self.slots.get(&slot).map(|r| (slot, r)))
    }
}
