
pub mod stages;

use std::{collections::HashMap, process::Command};
use tracing::{info, warn};
use fs_extra::error::Result;


struct Overlay {
    reads: Vec<String>,
    write: String,
}

#[derive(Default)]
struct RuntimeBuilder {
    name: String,
    workdir: PathBuf,
    arguments: Vec<String>,
    environs: HashMap<String, String>,
    overlays: Vec<Overlay>,
    stages: Vec<String>,
    loop_hooks: Vec<Fn<Result<()>>>,
    post_hooks: Vec<Fn<Result<()>>>,
    audit_log: Path
}

impl RuntimeBuilder {
    pub fn new(name: String, workdir: PathBuf) -> Self {
        info!("=== Runtime Building ===");
        let mut obj = Self::default();
        obj.name = name;
        obj.workdir = workdir;
        obj
    }

    fn arg(mut self, arg: String) -> Self {
        self.arguments.push(arg);
        self
    }
    fn args(mut self, mut args: Vec<String>) -> Self {
        self.arguments.append(&mut args);
        self
    }
    fn env(mut self, key: String, val: String) -> Self {
        self.environs.insert(key, val);
        self
    }
    fn envs(mut self, envs: &HashMap<String, String>) -> Self {
        self.environs.extend(envs.iter());
        self
    }
    
    pub async fn dry_execute(self) -> Result<()> {

    }

    pub async fn execute(self) -> Result<()> {
        info!("=== Launch Runtime ===", self.stages.len());



        info!("=== Launch Complete ===");

        Ok(())
    }
}

struct Runtime {

}
