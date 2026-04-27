use std::{str::FromStr, sync::Arc};

use anyhow::Result;

use crate::{execution::GameProcess, library::Game, methods::LaunchMode, store::SteamStore};

pub struct LaunchSteam {
    store: Arc<SteamStore>
}

impl LaunchMode for LaunchSteam {
    fn launch(game: &Game) -> Result<GameProcess> {
        

        Ok()
    }
    fn name() -> String {
        String::from_str("Steam").unwrap()
    }
}
