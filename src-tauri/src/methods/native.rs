use std::{path::PathBuf, sync::Arc};
use anyhow::Result;
use tokio::process::Command;

use crate::{config::AppConfig, library::Game, methods::{LaunchMode, ModeId}};

pub struct LaunchNative {
    config: Arc<AppConfig>,
    steam_emu_path: Option<PathBuf>,
}

impl LaunchNative {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            steam_emu_path: None,
        }
    }
}

impl LaunchMode for LaunchNative {
    fn build_cmd(&self, game: &Game) -> Command {
        
        Command::new("")
    }

    fn mode_id(&self) -> ModeId { ModeId::Native }
    fn name(&self) -> &'static str { "Native" }
}
