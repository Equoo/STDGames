use std::process::ExitStatus;
use anyhow::Result;

mod epic;
mod steam;
mod switch;

pub use epic::EpicStore;
pub use steam::SteamStore;
pub use switch::SwitchStore;

pub trait Store {
    pub fn login() -> Result<()>;
    pub fn open() -> Result<()>;
    pub fn close() -> Result<()>;
    pub fn is_active() -> Result<bool>;
    pub fn wait() -> Result<ExitStatus>;
    pub fn pid() -> Option<u32>;
}
