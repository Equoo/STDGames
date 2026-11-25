//! Errors and diagnostics shared by every crate in the workspace.

use thiserror::Error;

use crate::{
    capability::Capability,
    ids::{LayerId, ModeId, RunnerId},
    slot::Slot,
    target::TargetKind,
};

/// How serious a [`Diagnostic`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Reported, but the launch goes ahead.
    Warning,
    /// Aborts the launch.
    Error,
}

/// A human-readable finding from a layer's `preflight`.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Whether this blocks the launch.
    pub severity: Severity,
    /// What is wrong.
    pub message: String,
    /// How to fix it, when known.
    pub hint: Option<String>,
}

impl Diagnostic {
    /// A blocking diagnostic.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            hint: None,
        }
    }

    /// A non-blocking diagnostic.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            hint: None,
        }
    }

    /// Attaches a hint on how to fix the problem.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Errors raised while planning or running a launch.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    /// Two layers of the same tier claim the same slot.
    #[error("slot {slot:?} has conflicting layers: {first} and {second}")]
    SlotConflict {
        /// The contested slot.
        slot: Slot,
        /// The layer that claimed it first.
        first: LayerId,
        /// The layer that claimed it second.
        second: LayerId,
    },

    /// A mandatory slot was left empty by every tier.
    #[error(
        "slot {0:?} is mandatory but no tier (global, runner default, or mode) assigned it a layer"
    )]
    MissingMandatorySlot(Slot),

    /// A layer requires a capability no layer in the plan provides.
    #[error(
        "layer {layer} requires capability {capability:?}, which is provided by no layer in the plan"
    )]
    MissingCapability {
        /// The layer with the unmet requirement.
        layer: LayerId,
        /// The missing capability.
        capability: Capability,
    },

    /// A layer cannot run for this target kind.
    #[error("layer {layer} is incompatible with target {target:?}: {reason}")]
    IncompatibleTarget {
        /// The offending layer.
        layer: LayerId,
        /// The game's target.
        target: TargetKind,
        /// Why they are incompatible.
        reason: String,
    },

    /// No registered runner accepts this target.
    #[error("no registered runner accepts target {0:?}")]
    NoRunnerForTarget(TargetKind),

    /// The plan names a runner the registry does not know.
    #[error("runner {0} not found in the registry")]
    UnknownRunner(RunnerId),

    /// The plan names a layer the registry does not know.
    #[error("layer {0} not found in the registry")]
    UnknownLayer(LayerId),

    /// The requested mode is missing or disabled for this game.
    #[error("mode {0} is not enabled for this game")]
    ModeDisabled(ModeId),

    /// A layer's preflight reported a blocking [`Diagnostic`].
    #[error("preflight failed for layer {layer}: {}", diagnostic.message)]
    Preflight {
        /// The layer whose preflight failed.
        layer: LayerId,
        /// What it reported.
        diagnostic: Diagnostic,
    },

    /// The configuration is invalid.
    #[error("invalid configuration")]
    Config(#[from] ConfigError),

    /// A layer-specific failure.
    #[error("layer {layer} failed: {reason}")]
    LayerFailure {
        /// The failing layer.
        layer: LayerId,
        /// What went wrong.
        reason: String,
    },
}

/// Errors in the configuration files themselves.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// A field no tier of the cascade provided.
    #[error("required field `{field}` is missing after cascade resolution (game={game_id})")]
    MissingField {
        /// The game being resolved.
        game_id: String,
        /// The missing field.
        field: &'static str,
    },

    /// A field holds a value that cannot be used.
    #[error("invalid value for `{field}`: {reason}")]
    InvalidValue {
        /// The offending field.
        field: &'static str,
        /// Why it is invalid.
        reason: String,
    },

    /// A layer was referenced without one of its required parameters.
    #[error("layer {layer} requires parameter `{param}`")]
    MissingLayerParam {
        /// The layer being built.
        layer: LayerId,
        /// The missing parameter.
        param: &'static str,
    },

    /// A layer parameter holds a value that cannot be used.
    #[error("layer {layer}: invalid value {value:?} for parameter `{param}`: {reason}")]
    InvalidLayerParam {
        /// The layer being built.
        layer: LayerId,
        /// The offending parameter.
        param: &'static str,
        /// The value as written in the configuration.
        value: String,
        /// Why it is invalid.
        reason: String,
    },

    /// A `${...}` reference that cannot be expanded.
    #[error("{location}: {reason}")]
    Expansion {
        /// Where the value came from (e.g. `layer proton, param prefix_path`).
        location: String,
        /// What went wrong.
        reason: String,
    },
}

/// `Result` defaulting to [`CoreError`].
pub type Result<T, E = CoreError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use super::*;

    #[test]
    fn with_hint_keeps_message_and_severity() {
        let d = Diagnostic::warning("w").with_hint("h");
        assert_eq!(d.severity, Severity::Warning);
        assert_eq!(d.message, "w");
        assert_eq!(d.hint.as_deref(), Some("h"));
        assert_eq!(Diagnostic::error("e").severity, Severity::Error);
    }

    #[test]
    fn config_error_is_chained_not_repeated() {
        let err = CoreError::from(ConfigError::MissingField {
            game_id: "g".to_string(),
            field: "root",
        });
        assert_eq!(err.to_string(), "invalid configuration");
        let source = err.source().expect("config error is the source");
        assert!(source.to_string().contains("`root`"));
    }
}
