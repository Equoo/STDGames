//! What a layer's `wrap` hands to the next layer out.

use crate::command::CommandSpec;

/// Exactly one execution path exists: no re-entrancy, no Steam handoff.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Run this command directly.
    Direct(CommandSpec),
}

impl Outcome {
    /// The command to run.
    pub fn into_command(self) -> CommandSpec {
        match self {
            Outcome::Direct(spec) => spec,
        }
    }
}
