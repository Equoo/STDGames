use thiserror::Error;

use crate::{
    capability::Capability,
    ids::{LayerId, ModeId, RunnerId},
    slot::Slot,
    target::TargetKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            hint: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("slot {slot:?} has conflicting layers: {first} and {second}")]
    SlotConflict {
        slot: Slot,
        first: LayerId,
        second: LayerId,
    },

    #[error("slot {0:?} is mandatory but no tier (global, runner default, or mode) assigned it a layer")]
    MissingMandatorySlot(Slot),

    #[error("layer {layer} requires capability {capability:?}, which is provided by no layer in the plan")]
    MissingCapability { layer: LayerId, capability: Capability },

    #[error("layer {layer} is incompatible with target {target:?}: {reason}")]
    IncompatibleTarget {
        layer: LayerId,
        target: TargetKind,
        reason: String,
    },

    #[error("no registered runner accepts target {0:?}")]
    NoRunnerForTarget(TargetKind),

    #[error("runner {0} not found in the registry")]
    UnknownRunner(RunnerId),

    #[error("layer {0} not found in the registry")]
    UnknownLayer(LayerId),

    #[error("mode {0} is not enabled for this game")]
    ModeDisabled(ModeId),

    #[error("preflight failed for layer {layer}: {}", diagnostic.message)]
    Preflight { layer: LayerId, diagnostic: Diagnostic },

    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Catch-all for a layer's own `prepare`/`patch`/`wrap` failing for a
    /// reason specific to that layer (a filesystem error setting up its
    /// scratch space, a malformed parameter...) that doesn't warrant its own
    /// `CoreError` variant.
    #[error("layer {layer} failed: {reason}")]
    LayerFailure { layer: LayerId, reason: String },
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required field `{field}` is missing after cascade resolution (game={game_id})")]
    MissingField { game_id: String, field: &'static str },

    #[error("invalid value for `{field}`: {reason}")]
    InvalidValue { field: &'static str, reason: String },
}

pub type Result<T, E = CoreError> = std::result::Result<T, E>;
