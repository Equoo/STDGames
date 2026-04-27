use std::{collections::HashMap, process::{Child, ExitStatus}};

use anyhow::Result;

use crate::{library::Game, methods::LaunchMode};

pub struct GameExec {
    cmd: Vec<String>,
    precmd: Vec<String>,
    env: HashMap<String, String>
}

impl GameExec {
    pub fn new(cmd: Vec<String>) -> Self {
        GameExec{cmd, precmd: Vec::new(), env: HashMap::new()}
    }

    pub fn add_precmd(&mut self, precmd: Vec<String>) -> &mut Self {
        self.precmd = precmd;
        self
    }

    pub fn add_environs(&mut self, environs: HashMap<String, String>) -> &mut Self {
        self.env = environs;
        self
    }

    pub fn add_(&mut self, environs: HashMap<String, String>) -> &mut Self {
        self.env = environs;
        self
    }
}


//#[derive(Clone)]
pub struct GameProcess {
	process: Child,
	pub name: String,
    pub method: LaunchMode
}

impl GameProcess {
    pub fn kill(mut self) -> Result<()> {
        self.process.kill()?;

        Ok(())
    }

    pub fn wait(mut self) -> Result<ExitStatus> {
        Ok(self.process.wait()?)
    }

    pub fn is_running(mut self) -> Result<bool> {
        Ok(self.process.try_wait()?.is_none())
    }

    pub fn pid(self) -> u32 {
        self.process.id()
    }
}

