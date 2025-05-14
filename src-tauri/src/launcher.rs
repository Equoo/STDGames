use std::env;
use std::fs;
use nix::libc::open;
use nix::unistd::Uid;
use serde_json::{Number};
use serde::Deserialize;
use serde::Serialize;
use std::process::{Child, Command};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
use crate::game_library::Rconf;
use crate::game_library::{GameLibrary, GameInfo, ConfigPath};
use crate::utils::copy_recursively;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserData {
  username: String,
  language: String,
  steamid: Number
}

pub struct Launcher {
	running_process: Arc<Mutex<Option<Child>>>,
	pub library: Arc<GameLibrary>,
	pub user_data: UserData
}

pub fn storeData(data: UserData) {
	let user = env::var("USER").unwrap_or("".to_string());
	let path = format!("/sgoinfre/{user}/.stdgames_saves/user_data.json");
	let json = serde_json::to_string(&data).unwrap();
	fs::write(path, json).expect("Unable to write file");
}

pub fn loadData() -> UserData {
	let user = env::var("USER").unwrap_or("".to_string());
	let uid = Uid::current();
	let path = format!("/sgoinfre/{user}/.stdgames_saves/user_data.json");
	if !Path::new(&path).exists() {
		return UserData {
			username: user,
			language: "french".to_string(),
			steamid: (76561197995300747 + uid.as_raw() as u64).into()
		};
	}
	let json = fs::read_to_string(path).expect("Unable to read file");
	let data: UserData = serde_json::from_str(&json).unwrap();
	data
}

const GAMES_PATH: &str = "/sgoinfre/stdgames";

async fn copy_config_files(config: &Option<Vec<ConfigPath>>, game: &str) -> Result<(), Box<dyn std::error::Error>> {
	if config.is_none() {
		return Ok(());
	}
	
	let user = env::var("USER").unwrap_or("".to_string());

	for conf in config.as_ref().unwrap().clone() {
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

async fn copy_rconfig_files(launcher: &Launcher, config: &Option<Vec<Rconf>>, game: &str) -> Result<(), Box<dyn std::error::Error>> {
	if config.is_none() {
		return Ok(());
	}
	
	let user = env::var("USER").unwrap_or("".to_string());

	for conf in config.as_ref().unwrap().clone() {
		let txtpath = format!("/sgoinfre/{user}/.stdgames_saves/{}/{}", game, &conf.dest);
		let user_path = Path::new(&txtpath);
		let txtconfig_path = format!("{GAMES_PATH}/{}/{}", game, &conf.src);
		let config_path = Path::new(&txtconfig_path);
		if !config_path.exists() {
			println!("Config file not found: {}", config_path.display());
			continue;
		}
		fs::create_dir_all(format!("/sgoinfre/{user}/.stdgames_saves/{}", game))?;
		if let Err(e) = fs::copy(config_path, user_path) {
			eprintln!("Warning: failed to copy {:?} to {:?}: {}", config_path, user_path, e);
		}
		// replace all occurences of {username} or {steamid} or {language} in the file
		println!("Replacing variables in file: {}", user_path.display());
		let mut content = fs::read_to_string(user_path)?;
		content = content.replace("{username}", launcher.user_data.username.as_str());
		content = content.replace("{steamid}",launcher.user_data.steamid.to_string().as_str());
		content = content.replace("{language}", launcher.user_data.language.as_str());
		fs::write(user_path, content)?;
	}
	Ok(())
}

impl Launcher {
	pub fn new(library: Arc<GameLibrary>) -> Self {
		Launcher {
			running_process: Arc::new(Mutex::new(None)),
			library: library,
			user_data: loadData()
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

		copy_rconfig_files(self, &data.r_conf, &data.name)
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

		if !data.config.is_none() {
			for conf in data.config.as_ref().unwrap().iter() {
				let path = format!("/sgoinfre/{user}/.stdgames_saves/{}/{}", &data.name, conf.user);
				binds.insert(path, format!("{game_path}/{}", conf.original));
			}
		}

		if !data.r_conf.is_none() {
			for conf in data.r_conf.as_ref().unwrap().iter() {
				let path = format!("/sgoinfre/{user}/.stdgames_saves/{}/{}", &data.name, conf.dest);
				binds.insert(path, format!("{game_path}/{}", conf.src));
			}
		}

		let protonpath = format!("/sgoinfre/stdgames/.ressources/protons/{}", data.proton);
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
			--uid 5 \
			--proc /proc --dev /dev --tmpfs /tmp \
			--bind /run/user/{uid}/pulse/native /run/pulse/native {binds_str} {game_command}", data.workdir.clone().unwrap_or("".to_string()));

		let junest_env = env::var("JUNEST_ENV").unwrap_or("".to_string());
		if junest_env != "1" {
			final_command = format!("cd {game_path}/{} && {JUNEST_PATH} -b \"\
				--bind /run/user/{uid} /run/user/{uid}	\
				--uid 5 \
				--bind /sgoinfre /sgoinfre				\
				--bind /goinfre /goinfre				\
				--bind /run/user/{uid}/pulse/native /run/pulse/native {binds_str}\" exec {game_command}", data.workdir.clone().unwrap_or("".to_string()));
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
