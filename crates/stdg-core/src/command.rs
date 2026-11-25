//! The command being built as it flows through the layer pipeline.

use std::collections::BTreeMap;

use crate::binding::PathValue;

#[derive(Debug, Clone, PartialEq, Eq)]
/// One command-line argument.
pub enum ArgValue {
    /// Passed through unchanged.
    Literal(String),
    /// A path, rendered from the consuming process's point of view.
    Path(PathValue),
}

impl ArgValue {
    /// The argument as the process receives it.
    pub fn render(&self) -> String {
        match self {
            ArgValue::Literal(s) => s.clone(),
            ArgValue::Path(p) => p.effective().display().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Separator between the entries of an [`EnvValue::PathList`] (e.g. `:`).
pub struct PathListSeparator(pub char);

#[derive(Debug, Clone, PartialEq, Eq)]
/// The value of one environment variable.
pub enum EnvValue {
    /// Passed through unchanged.
    Literal(String),
    /// A single path.
    Path(PathValue),
    /// Paths joined by a separator (e.g. `LD_LIBRARY_PATH`).
    PathList(Vec<PathValue>, PathListSeparator),
}

impl EnvValue {
    /// The value as the process receives it.
    pub fn render(&self) -> String {
        match self {
            EnvValue::Literal(s) => s.clone(),
            EnvValue::Path(p) => p.effective().display().to_string(),
            EnvValue::PathList(paths, sep) => paths
                .iter()
                .map(|p| p.effective().display().to_string())
                .collect::<Vec<_>>()
                .join(&sep.0.to_string()),
        }
    }
}

/// The command being built as it flows through the layer pipeline.
#[derive(Debug, Clone, Default)]
pub struct CommandSpec {
    /// The executable. `None` only for a spec still being assembled.
    pub program: Option<PathValue>,
    /// Arguments after the program.
    pub args: Vec<ArgValue>,
    /// Environment variables set on top of the inherited environment.
    pub env: BTreeMap<String, EnvValue>,
    /// Working directory; inherited when `None`.
    pub cwd: Option<PathValue>,
}

impl CommandSpec {
    /// A command running `program` with no args, env or cwd.
    #[must_use]
    pub fn new(program: PathValue) -> Self {
        Self {
            program: Some(program),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    /// Appends an argument.
    pub fn push_arg(&mut self, arg: ArgValue) {
        self.args.push(arg);
    }

    /// Appends a literal argument.
    pub fn push_arg_literal(&mut self, s: impl Into<String>) {
        self.args.push(ArgValue::Literal(s.into()));
    }

    /// Appends a path argument.
    pub fn push_arg_path(&mut self, p: PathValue) {
        self.args.push(ArgValue::Path(p));
    }

    /// Sets a literal environment variable, replacing any previous value.
    pub fn set_env_literal(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.env.insert(key.into(), EnvValue::Literal(val.into()));
    }

    /// Sets a path environment variable, replacing any previous value.
    pub fn set_env_path(&mut self, key: impl Into<String>, val: PathValue) {
        self.env.insert(key.into(), EnvValue::Path(val));
    }

    /// Makes `self` the arguments of `new_program`.
    #[must_use]
    pub fn wrapped_by(mut self, new_program: PathValue) -> Self {
        let mut args = Vec::new();
        if let Some(program) = self.program.take() {
            args.push(ArgValue::Path(program));
        }
        args.append(&mut self.args);

        Self {
            program: Some(new_program),
            args,
            env: self.env,
            cwd: self.cwd,
        }
    }
}
