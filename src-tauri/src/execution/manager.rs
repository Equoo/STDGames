use anyhow::{Context, Result};

use crate::{
    config::CONFIG,
    debug::log_step,
    execution::{GameExecution, GameProcess},
    library::Game,
};

fn get_game<'a>(library: &'a Vec<Game>, name: &'a String) -> Result<&'a Game> {
    library
        .iter()
        .find(|g| &g.slug == name)
        .ok_or_else(|| anyhow::anyhow!("Game '{}' not found in library", name))
}

impl GameExecution {
    pub fn start(&mut self, name: &str) -> Result<()> {
        // add possibility to launch via steam

        log_step("manager", format!("start requested for '{name}'"));

        let host_vars = CONFIG.clone().build_vars();
        let sandbox_vars = CONFIG.clone().build_sandbox_vars();
        let mut launch_data = get_game(&self.library, &name.to_string())?.launch.clone();
        launch_data.replace_vars(&host_vars, &sandbox_vars);
        let mut command = GameExecution::build_command(name, &launch_data)?;
        let child = match command.spawn() {
            Ok(child) => {
                log_step("manager", format!("'{name}' spawned with pid {}", child.id()));
                child
            }
            Err(e) => {
                log_step("manager", format!("'{name}' FAILED to spawn: {e}"));
                return Err(e).with_context(|| format!("Failed to spawn game '{name}'"));
            }
        };
        self.running = Some(GameProcess {
            name: name.to_string(),
            process: child,
        });

        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if self.running.is_some() {
            match self.running.as_mut().unwrap().process.try_wait() {
                Ok(Some(status)) => {
                    log_step("manager", format!(
                        "'{}' exited with {status}",
                        self.running.as_ref().unwrap().name,
                    ));
                    // Clear the finished process so we don't log/re-check it
                    // on every future poll: try_wait() caches and keeps
                    // returning the same exit status forever once a child
                    // has exited, so without this is_running() would report
                    // (and log) the same exit on every call from here on.
                    self.running = None;
                    false
                }
                Ok(None) => true,
                Err(e) => {
                    log_step("manager", format!(
                        "'{}' error while waiting: {e}",
                        self.running.as_ref().unwrap().name,
                    ));
                    self.running = None;
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        if self.is_running() {
            self.running
                .as_mut()
                .unwrap()
                .process
                .kill()
                .context("Failed to kill the running game process")?;
            self.running = None;
        }
        Ok(())
    }

    pub fn kill(&mut self) -> Result<()> {
        if self.is_running() {
            self.running
                .as_mut()
                .unwrap()
                .process
                .kill()
                .context("Failed to kill the running game process")?;
            self.running = None;
        }
        Ok(())
    }
}

