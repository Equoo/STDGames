use std::collections::BTreeMap;

use stdg_core::{CapabilitySet, CoreError, LayerCatalog, LayerId, LayerRef, ResolvedConfig, Slot};

use crate::SlotMap;

/// Slots every plan must fill, regardless of mode: there is no "unsandboxed"
/// launch. A game or runner is still free to choose *which* sandbox profile
/// (see `stdg-layers-sandbox`), just not to skip the slot entirely — see
/// `check_mandatory_slots`.
const MANDATORY_SLOTS: &[Slot] = &[Slot::Sandbox];

/// Resolves `tiers` (increasing precedence — e.g. `[global.baseline_layers,
/// runner_defaults.layers_for(mode), mode.layers]`) into a final slot
/// assignment, checks that every `MANDATORY_SLOTS` entry got filled by some
/// tier, then checks that every resolved layer's `requires()` is covered by
/// the union of every layer's `provides()` in the plan.
pub fn validate_plan(
    config: &ResolvedConfig,
    tiers: &[Vec<LayerRef>],
    catalog: &dyn LayerCatalog,
) -> Result<SlotMap, CoreError> {
    let slots = assign_slots(config, tiers, catalog)?;
    check_mandatory_slots(&slots)?;
    check_capabilities(config, &slots, catalog)?;
    Ok(slots)
}

/// Errors with [`CoreError::MissingMandatorySlot`] if any slot in
/// [`MANDATORY_SLOTS`] was left unfilled by every tier. In practice this
/// almost never fires from a game's own config: a default sandbox layer
/// belongs in `defaults/global.toml`'s `baseline_layers`, so every mode of
/// every game gets one unless something more specific overrides it.
pub fn check_mandatory_slots(slots: &SlotMap) -> Result<(), CoreError> {
    for &slot in MANDATORY_SLOTS {
        if !slots.contains_key(&slot) {
            return Err(CoreError::MissingMandatorySlot(slot));
        }
    }
    Ok(())
}

/// Resolves each tier's `LayerRef`s through `catalog` and assigns them to
/// slots. Two layers claiming the same slot *within one tier* is a
/// `SlotConflict` (almost always a config typo — the whole point of a slot
/// is that only one layer can occupy it). Two *different* tiers claiming the
/// same slot is not a conflict: the later, more specific tier simply
/// overrides the earlier one. That's what makes a mode's own `layers` list a
/// genuine override instead of having to restate the runner's (or global)
/// defaults every time a game only wants to change one slot.
pub fn assign_slots(
    config: &ResolvedConfig,
    tiers: &[Vec<LayerRef>],
    catalog: &dyn LayerCatalog,
) -> Result<SlotMap, CoreError> {
    let mut slots: SlotMap = BTreeMap::new();

    for tier in tiers {
        let mut occupants_this_tier: BTreeMap<Slot, LayerId> = BTreeMap::new();

        for layer_ref in tier {
            let layer = catalog.resolve_layer(layer_ref, config)?;
            let slot = layer.slot();

            if let Some(existing) = occupants_this_tier.get(&slot) {
                return Err(CoreError::SlotConflict {
                    slot,
                    first: existing.clone(),
                    second: layer.id(),
                });
            }
            occupants_this_tier.insert(slot, layer.id());
            slots.insert(slot, layer_ref.clone());
        }
    }

    Ok(slots)
}

/// Checks that every layer in the final slot assignment has its `requires()`
/// covered by the union of every layer's `provides()` in the plan (e.g. a
/// DLL-swap SteamApi layer requiring the Windows ABI that only a Compat
/// layer provides).
pub fn check_capabilities(config: &ResolvedConfig, slots: &SlotMap, catalog: &dyn LayerCatalog) -> Result<(), CoreError> {
    let mut provided = CapabilitySet::new();
    let mut resolved = Vec::with_capacity(slots.len());

    for layer_ref in slots.values() {
        let layer = catalog.resolve_layer(layer_ref, config)?;
        provided = provided.union(&layer.provides());
        resolved.push(layer);
    }

    for layer in &resolved {
        let missing = layer.requires().missing_from(&provided);
        if let Some(capability) = missing.iter().next() {
            return Err(CoreError::MissingCapability {
                layer: layer.id(),
                capability,
            });
        }
    }

    Ok(())
}
