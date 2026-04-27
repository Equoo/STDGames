use std::{path::PathBuf, str::FromStr, sync::Arc};

use anyhow::Result;

use crate::{execution::GameProcess, library::Game, methods::LaunchMode};

pub struct LaunchNative {
    steam_emu_path: Option<PathBuf>,
}

impl LaunchMode for LaunchSteam {
    fn launch(game: &Game) -> Result<GameProcess> {
        
        Ok()
    }
    fn name() -> String {
        String::from_str("Steam").unwrap()
    }
}
