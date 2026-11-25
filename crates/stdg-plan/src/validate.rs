//! Slot assignment and plan validation.

use std::collections::BTreeMap;

use stdg_core::{CapabilitySet, CoreError, LayerCatalog, LayerId, LayerRef, ResolvedConfig, Slot};

use crate::SlotMap;

/// Slots every plan must fill.
const MANDATORY_SLOTS: &[Slot] = &[Slot::Sandbox];

/// Assigns slots, then checks mandatory slots and capabilities.
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

/// Errors if a mandatory slot is empty.
pub fn check_mandatory_slots(slots: &SlotMap) -> Result<(), CoreError> {
    for &slot in MANDATORY_SLOTS {
        if !slots.contains_key(&slot) {
            return Err(CoreError::MissingMandatorySlot(slot));
        }
    }
    Ok(())
}

/// Assigns layers to slots; later tiers override earlier ones.
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

/// Checks every `requires()` is covered by some `provides()`.
pub fn check_capabilities(
    config: &ResolvedConfig,
    slots: &SlotMap,
    catalog: &dyn LayerCatalog,
) -> Result<(), CoreError> {
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
