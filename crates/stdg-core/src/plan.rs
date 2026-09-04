use std::collections::BTreeMap;

use crate::{
    config::ResolvedConfig,
    ids::{GameId, ModeId, RunnerId},
    layer::LayerRef,
    slot::Slot,
    target::TargetKind,
};

/// Planner output: fully resolved and validated, ready for `stdg-exec` or
/// for `explain` display. No `Option` fields here.
#[derive(Debug, Clone)]
pub struct Plan {
    pub game_id: GameId,
    pub mode_id: ModeId,
    pub target: TargetKind,
    pub runner: RunnerId,
    pub slots: BTreeMap<Slot, LayerRef>,
    pub config: ResolvedConfig,
}

impl Plan {
    /// Layers in actual application order (innermost to outermost).
    pub fn layers_inside_out(&self) -> impl Iterator<Item = (Slot, &LayerRef)> {
        Slot::application_order().filter_map(move |slot| self.slots.get(&slot).map(|r| (slot, r)))
    }
}
