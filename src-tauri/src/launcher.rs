use std::env;
use std::fs;
use nix::unistd::Uid;
use std::process::{Child, Command};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
use crate::game_library::{GameLibrary, GameInfo, ConfigPath};
use crate::utils::copy_recursively;

pub struct Launcher {
	running_process: Arc<Mutex<Option<Child>>>,
	pub library: Arc<GameLibrary>,
}

const GAMES_PATH: &str = "/sgoinfre/stdgames";

async fn copy_config_files(config: &Vec<ConfigPath>, game: &str) -> Result<(), Box<dyn std::error::Error>> {
	let user = env::var("USER").unwrap_or("".to_string());

	for conf in config.clone() {
		let txtpath = format!("/sgoinfre/{user}/.stdgames_saves/{}/{}", game, &conf.user);
		let user_path = Path::new(&txtpath);
		let txtconfig_path = format!("{GAMES_PATH}/{}/{}", game, &conf.original);
		let config_path = Path::new(&txtconfig_path);
		if !config_path.exists() {
			println!("Config file not found: {}", config_path.display());
			continue;
		}
		if !user_path.exists() {
			fs::create_dir_all(format!("/sgoinfre/{user}/.stdgames_saves/{}", game))?;
			copy_recursively(config_path, user_path).await
				.expect("Erreur lors de la copie du répertoire de configuration");
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

	pub async fn launch_game(&self, game: &str) -> Result<(), Box<dyn std::error::Error>> {
		let	data: &GameInfo = match self.library.get_game(game) {
            Some(data) => data,
            None => {
                println!("Game '{}' not found!", game);
                return Ok(());
            },
        };

		println!("Game data: {:?}", data);

		copy_config_files(&data.config, &data.name)
			.await
			.expect("Erreur lors de la copie des fichiers de configuration");

		let user = env::var("USER").unwrap_or("".to_string());
		let junest_home = format!("/goinfre/{user}/.stdgames/junest");
		
		let mut binds: HashMap<String, String> = HashMap::new();
		let mut env_vars: HashMap<String, String> = data.env.clone();
		env_vars.insert("JUNEST_HOME".to_string(), junest_home.to_string());

		const JUNEST_PATH: &str = "/sgoinfre/dderny/junest/bin/junest";
		let game_path = format!("{GAMES_PATH}/{}", &data.name);
		let exec_path = format!("{game_path}/{}", &data.exec_path);

		for conf in data.config.iter() {
			let path = format!("/sgoinfre/{user}/.stdgames_saves/{}/{}", &data.name, conf.user);
			binds.insert(path, format!("{game_path}/{}", conf.original));
		}

		let protonpath = format!("/sgoinfre/stdgames/.data/protons/{}", data.proton);
		let prefix = format!("/sgoinfre/{user}/.stdgames_saves/{}", data.proton);

		const PYTHONPATH: &str = "/usr/lib/python3/dist-packages";
		env_vars.extend(HashMap::from([
			(String::from("PYTHONPATH"), PYTHONPATH.to_string()),
			(String::from("PROTONPATH"), protonpath.clone()),
			(String::from("STEAM_COMPAT_DATA_PATH"), prefix.clone()),
			(String::from("WINEPREFIX"), prefix),
			(String::from("DXVK_ASYNC"), String::from("1"))
		]));

		let game_command = match data.launch_type.as_str() {
			"native" => &exec_path,
			"umu" => {
				env_vars.insert("UMU_RUNTIME_UPDATE".to_string(), "0".to_string());
				env_vars.insert("XDG_DATA_HOME".to_string(), format!("/goinfre/{user}/.stdgames/"));

				&format!("umu-run {exec_path}")
			},
			"epicgame" => {
				&format!("legendary launch --wine {} {}", protonpath, data.exec_path)
			},
			&_ => {
				println!("Unknown launch type: {}", data.launch_type);
				return Ok(());
			},
		};

		let mut binds_str = String::new();
		for (key, value) in binds {
			binds_str.push_str(&format!(" --bind {} {}", key, value));
		}

		if let Err(e) = fs::create_dir_all(format!("/tmp/{user}")) {
			println!("Error creating /tmp/{user}: {}", e);
		}

		let uid = Uid::current().to_string();
		let mut final_command = format!("cd {game_path}/{} && bwrap \
			--bind / /	\
			--bind /etc/group /etc/group --bind /etc/shadow /etc/shadow	\
			--proc /proc --dev /dev --tmpfs /tmp \
			--bind /run/user/{uid}/pulse/native /run/pulse/native {binds_str} {game_command}", data.workdir);

		let junest_env = env::var("JUNEST_ENV").unwrap_or("".to_string());
		if junest_env != "1" {
			final_command = format!("cd {game_path}/{} && {JUNEST_PATH} -b \"\
				--bind /run/user/{uid} /run/user/{uid}	\
				--bind /sgoinfre /sgoinfre				\
				--bind /goinfre /goinfre				\
				--bind /run/user/{uid}/pulse/native /run/pulse/native {binds_str}\" exec {game_command}", data.workdir);
		}

		println!("Launching game: {}", final_command);
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
