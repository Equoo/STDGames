use crate::command::CommandSpec;

/// Exactly one execution path exists: no re-entrancy, no Steam handoff.
/// Kept as a single-variant enum deliberately, so the type itself documents
/// that design constraint.
#[derive(Debug, Clone)]
pub enum Outcome {
    Direct(CommandSpec),
}

impl Outcome {
    pub fn into_command(self) -> CommandSpec {
        match self {
            Outcome::Direct(spec) => spec,
        }
    }
}
