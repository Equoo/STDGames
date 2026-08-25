use anyhow::{Result, anyhow};
use std::{collections::BTreeMap, path::PathBuf, str::FromStr, sync::Arc};
use tokio::process::{Child, Command};

use crate::{
    config::AppConfig,
    library::Game,
    methods::{LaunchMode, ModeId, native::LaunchNative},
};

pub struct GameProcessManager {
    config: Arc<AppConfig>,
    modes: BTreeMap<ModeId, Box<dyn LaunchMode>>,
    child: Option<Child>,
    game_id: Option<&'static str>,
}

impl GameProcessManager {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let mut modes: BTreeMap<ModeId, Box<dyn LaunchMode>> = BTreeMap::new();

        modes.insert(ModeId::Native, Box::new(LaunchNative::new(config.clone())));
        // TODO: ...

        // TODO: Keep tracking of child after launcher closing
        Self {
            config,
            modes,
            child: None,
            game_id: None,
        }
    }

    fn game_path(game: &'static str) -> Result<PathBuf> {
        // TODO: If installed -> install folder
        Ok(PathBuf::from_str(&format!(
            "{}/{}",
            CONFIG.games_dir, game
        ))?)
    }

    pub async fn launch(&mut self, game_name: &'static str, mode: &ModeId) -> Result<()> {
        if self.is_running()? {
            anyhow!("A game is already running: {}", self.game_id.unwrap());
        }

        let game: &Game; // TODO: get from library
        let game_path = Self::game_path(game);

        if let Some(launch_mode) = self.modes.get(mode) {
            let cmd: Command = launch_mode.build_cmd(game);
        } else {
            anyhow!("Unimplemented mode: {:?}", mode);
        }

        Ok(())
    }

    pub async fn kill(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            child.kill().await?;
        }
        Ok(())
    }
    pub fn is_running(&mut self) -> Result<bool> {
        if let Some(child) = self.child.as_mut() {
            Ok(child.try_wait()?.is_none())
        } else {
            Ok(false)
        }
    }
    pub fn get_running(&self) -> Option<&'static str> {
        self.game_id
    }
}
