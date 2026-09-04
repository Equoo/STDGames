use std::collections::BTreeMap;

use crate::binding::PathValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgValue {
    Literal(String),
    Path(PathValue),
}

impl ArgValue {
    /// Flattens to the literal string a process actually receives as this
    /// argument. Shared by `explain`'s display and any layer (e.g. bwrap)
    /// that needs to re-emit an inner command's args as plain strings.
    pub fn render(&self) -> String {
        match self {
            ArgValue::Literal(s) => s.clone(),
            ArgValue::Path(p) => p.effective().display().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathListSeparator(pub char);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvValue {
    Literal(String),
    Path(PathValue),
    /// e.g. LD_PRELOAD, LD_LIBRARY_PATH: a list of paths joined by a separator.
    PathList(Vec<PathValue>, PathListSeparator),
}

impl EnvValue {
    /// Flattens to the literal string a process actually receives for this
    /// environment variable.
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
    pub program: Option<PathValue>,
    pub args: Vec<ArgValue>,
    pub env: BTreeMap<String, EnvValue>,
    pub cwd: Option<PathValue>,
}

impl CommandSpec {
    pub fn new(program: PathValue) -> Self {
        Self {
            program: Some(program),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    pub fn push_arg(&mut self, arg: ArgValue) {
        self.args.push(arg);
    }

    pub fn push_arg_literal(&mut self, s: impl Into<String>) {
        self.args.push(ArgValue::Literal(s.into()));
    }

    pub fn push_arg_path(&mut self, p: PathValue) {
        self.args.push(ArgValue::Path(p));
    }

    pub fn set_env_literal(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.env.insert(key.into(), EnvValue::Literal(val.into()));
    }

    pub fn set_env_path(&mut self, key: impl Into<String>, val: PathValue) {
        self.env.insert(key.into(), EnvValue::Path(val));
    }

    /// Turns `self` into the argument list of a new command whose program is
    /// `new_program`. Used by layers that restructure the invocation (e.g.
    /// Proton turning `<exe>` into `proton run <exe>`).
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
