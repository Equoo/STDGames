
use std::process::{Child, Command, ExitStatus};

use anyhow::Result;

use crate::store::Store;

pub struct EpicStore {
    process: Option<Child>
}

impl Store for EpicStore {
    fn open(&mut self) -> Result<()> {
        self.process = Some(Command::new("heroiclauncher").spawn()?);

        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        if let Some(proc) = self.process.as_mut() {
            proc.kill()?;
        }

        Ok(())
    }

    fn login(&self) -> Result<()> {

        Ok(())
    }

    fn is_active(&mut self) -> Result<bool> {
        if let Some(proc) = self.process.as_mut() {
            Ok(proc.try_wait()?.is_none())
        } else {
            Ok(false)
        }
    }

    fn wait(&mut self) -> Result<ExitStatus> {
        if let Some(proc) = self.process.as_mut() {
            Ok(proc.wait()?)
        } else {
            Err(anyhow::format_err!("Try to wait on epicgame but not open"))
        }
    }

    fn pid(&mut self) -> Option<u32> {
        if let Some(proc) = self.process.as_mut() {
            Some(proc.id())
        } else {
            None
        }
    }
}

struct EpicCredentials {
    username: String,
    token: String
}
