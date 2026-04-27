use anyhow::Result;

use crate::{execution::GameProcess, library::Game};

mod epic;
mod native;
mod onlinefix;
mod steam;
mod switch;

pub enum LaunchMethod {
    Steam,
    SteamOnlineFix,
    Native,
    Epic,
    Switch
}

pub trait LaunchMode {
    pub fn launch(game: &Game) -> Result<GameProcess>;
    pub fn name() -> String;
}
