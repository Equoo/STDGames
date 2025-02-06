use std::env;
use std::fs;
use nix::unistd::Uid;
use std::process::{Child, Command};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
use crate::game_library::{GameLibrary, GameInfo, ConfigPath};
use crate::utils::copy_recursively;
use crate::{JUNEST_HOME, LEGENDARY_LAUNCH, JUNEST_PATH, GAMES_PATH};

pub struct Launcher {
	running_process: Arc<Mutex<Option<Child>>>,
	pub library: Arc<GameLibrary>,
}

fn copy_config_files(config: &Vec<ConfigPath>, game: &str) -> Result<(), Box<dyn std::error::Error>> {
	let user = env::var("USER").unwrap_or("".to_string());

	for conf in config.clone() {
		let txtpath = format!("/sgoinfre/{user}/.stdgames/{}/{}", game, &conf.user);
		let user_path = Path::new(&txtpath);
		let txtconfig_path = format!("{GAMES_PATH}/{}/{}", game, &conf.original);
		let config_path = Path::new(&txtconfig_path);
		if !config_path.exists() {
			println!("Config file not found: {}", config_path.display());
			continue;
		}
		if !user_path.exists() {
			fs::create_dir_all(format!("/sgoinfre/{user}/.stdgames/{}", game))?;
			copy_recursively(config_path, user_path)?;
		}
	}
	Ok(())
}

impl Launcher {
	pub fn new(library: Arc<GameLibrary>) -> Self {
		Launcher {
			running_process: Arc::new(Mutex::new(None)),
			library: library
		}
	}

	pub fn launch_game(&self, game: &str) -> Result<(), Box<dyn std::error::Error>> {
		let PYTHONPATH = env::var("PYTHONPATH").unwrap_or("".to_string());
		let USER = env::var("USER").unwrap_or("".to_string());
		let HOME = env::var("HOME").unwrap_or("".to_string());
		let UMU_PATH = "umu-run";
		let UID = Uid::current().to_string();

		let	data: &GameInfo = match self.library.get_game(game) {
            Some(data) => data,
            None => {
                println!("Game '{}' not found!", game);
                return Ok(());
            },
        };

		copy_config_files(&data.config, &data.name)?;

		let mut env_vars: HashMap<String, String> = data.env.clone();
		let mut binds: HashMap<String, String> = HashMap::new();

		env_vars.insert("JUNEST_HOME".to_string(), JUNEST_HOME.to_string());

		let GAME_PATH = format!("{GAMES_PATH}/{}", &data.name);
		let exec_path = format!("{GAME_PATH}/{}", &data.exec_path);
		let command = match data.launch_type.as_str() {
			"native" => &exec_path,
			"umu" => {
				binds.insert("/goinfre/{USER}/umu".to_string(), "{HOME}/.local/share/umu".to_string());

				env_vars.extend(HashMap::from([
					(String::from("PYTHONPATH"), PYTHONPATH + ":/usr/lib/python3/dist-packages"),
					(String::from("PROTONPATH"), format!("/sgoinfre/dderny/wines/{}", data.proton)),
					(String::from("WINEPREFIX"), format!("/sgoinfre/{USER}/.stdgames/{}", data.proton))
				]));

				&format!("{UMU_PATH} \"{exec_path}\"")
			},
			"epicgame" => {
				&format!("{} {}", LEGENDARY_LAUNCH, data.exec_path)
			},
			&_ => {
				println!("Unknown launch type: {}", data.launch_type);
				return Ok(());
			},
		};

		let mut bindsstr = String::new();
		for (key, value) in binds {
			bindsstr.push_str(&format!(" --bind {} {}", key, value));
		}

		let final_command = &format!("cd {GAME_PATH} && {JUNEST_PATH} -b \"\
			--bind /run/user/{UID} /run/user/{UID}	\
			--bind /sgoinfre /sgoinfre				\
			--bind /goinfre /goinfre				\
			--bind /run/user/{UID}/pulse/native /run/pulse/native {bindsstr}\" exec {command}");

		println!("Launching game: {}", command);
		let process = Command::new("sh")
        	.arg("-c")
			.arg(final_command)
			.envs(&env_vars)
			.spawn()
			.expect("Failed to launch the game");
		{
			let mut running_process = self.running_process.lock().unwrap();
			*running_process = Some(process);
		}

		println!("Game launched!");
		Ok(())
	}

	pub fn is_game_running(&self) -> bool {
		let mut running_process = self.running_process.lock().unwrap(); // Get a mutable lock
		if let Some(child) = running_process.as_mut() {
			match child.try_wait() {
				Ok(Some(status)) => {
					println!("Game exited with: {:?}", status);
					*running_process = None; // Clear the process after it exits
					false
				}
				Ok(None) => true,
				Err(e) => {
					eprintln!("Error checking process status: {:?}", e);
					false
				}
			}
		} else {
			false
		}
	}
}
