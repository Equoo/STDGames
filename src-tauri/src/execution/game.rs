use std::{collections::HashMap, env, error::Error, path::{Path, PathBuf}, process::{Child, Command}};
use anyhow::anyhow;

use crate::{
	config::{CONFIG, Config},
	library::GameLaunchData,
};





impl GameExecution {
	pub fn new() -> Self {
		Self { running: None }
	}

	pub fn setup() -> Result<(), Box<dyn Error>> {
		if !is_authorized() {
			return Err(Box::new(AppError::Unauthorized));
		}
		
		let config = CONFIG.clone();
		
		copy_directory(
			Path::new(&config.junest_home_dir),
			Path::new(&config.temp_junest_home_dir),
			|_| {},
		).expect("Failed to copy Junest home directory");

		Ok(())
	}

	pub fn junest_run(
		mut command: Vec<String>,
		mut environ: HashMap<String, String>,
		overlay: &Option<Overlay>,
	) -> Command {
		let conf = CONFIG.clone();
		let uid: u32 = unsafe { libc::getuid() };

		environ.insert("JUNEST_HOME".to_string(), conf.temp_junest_home_dir.clone());

		let user = conf.username.clone();
		[
			&format!("/tmp/{user}"),
			&format!("/tmp/{user}/stdgames/rw"),
			&format!("/tmp/{user}/stdgames/overlay_work"),
			&format!("/tmp/{user}/stdgames/work"),
		].iter()
		.for_each(|folder| {
			std::fs::create_dir_all(folder).unwrap_or_else(|e| {
				eprintln!("Failed to create directory {}: {}", folder, e);
			});
		});

		let overlay_str = if let Some(o) = overlay {
			command.insert(0, conf.overlay.clone());
			command.push(format!("/tmp/{user}/stdgames/rw").to_string());
			command.push(o.dst.clone());
			format!(
				"--overlay-src {}
				--overlay /tmp/{user}/stdgames/rw /tmp/{user}/stdgames/overlay_work /tmp/{user}/stdgames/work",
				o.src.join(" ")
			)
		} else {
			String::new()
		};

		let work_dir = if overlay.is_some() {
			PathBuf::from(format!("/tmp/{}/stdgames/work", conf.username))
		} else {
			PathBuf::from(&conf.user_home)
		};

		let mut cmd = Command::new(conf.junest_bin.clone());
		cmd.envs(environ)
			.current_dir(work_dir)
			.arg("-b")
			.arg(format!(
				"--bind /sgoinfre /sgoinfre
				--bind /goinfre /goinfre
				--bind /media /media
				--bind /tmp/{} /tmp
				--bind {} /usr
				--bind /tmp/.X11-unix /tmp/.X11-unix
				--bind /run/user/{uid}/pulse/native /run/pulse/native
				{overlay_str}
			", conf.username, conf.junest_bind
			))
			.args(command);
		cmd
	}

	fn get_abs_command(mut command: Vec<String>) -> Vec<String> {
		command[0] = if Path::new(&command[0]).is_absolute() {
			command[0].clone()
		} else {
			format!("/tmp/{}/stdgames/work/{}", CONFIG.username, command[0])
		};
		command
	}

	fn found_and_replace(
		strs: &mut Vec<String>,
		vars: &HashMap<String, String>,
	) {
		for i in 0..strs.len() {
			let mut new_value = strs[i].clone();
			for (vk, vv) in vars {
				let mut vkk = vk.clone();
				vkk.insert(0, '$');
				new_value = new_value.replace(&vkk, vv);
			}
			strs[i] = new_value;
		}
	}

	pub fn run(
		&mut self,
		name: &String,
		game: &GameLaunchData,
	) -> Result<Command, Box<dyn Error>> {
		let conf: Config = CONFIG.clone();

		let mut vars = CONFIG.clone().into_env_vars()
			.iter().map(|(k, v)| (k.to_string(), v.to_string()))
			.collect::<HashMap<String, String>>();
		vars.extend(env::vars().map(|(k, v)| (k, v)));

		let mut environ = game.environs.clone().unwrap_or_default();
		let mut updates = Vec::new();

		for (k, v) in &environ {
			let mut new_value = v.clone();
			for (vk, vv) in &vars {
				let mut vkk = vk.clone();
				vkk.insert(0, '$');
				new_value = new_value.replace(&vkk, vv);
			}
			updates.push((k.clone(), new_value));
		}

		// Apply changes after iteration
		for (k, new_value) in updates {
			environ.insert(k, new_value);
		}

		let mut command = game.start.clone();
		Self::found_and_replace(&mut command, &vars);
		command = Self::get_abs_command(command);

		command = match game.method.as_str() {
			"native" => command,
			_ => {
				// A proton version
				let save_dir = format!("{}/{}", conf.user_save_dir, game.method);
				environ.extend(
					[("PROTONPATH".to_string(), conf.protons_dir),
					("PYTHONPATH".to_string(), "/usr/lib/python3/dist-packages".to_string()),
					("STEAM_COMPAT_DATA_PATH".to_string(), save_dir.clone()),
					("WINEPREFIX".to_string(), game.method.clone()),
					("DXVK_ASYNC".to_string(), "1".to_string()),
					("GAMEID".to_string(), "0".to_string()),
					("UMU_RUNTIME_UPDATE".to_string(), "0".to_string())]
				);
				std::fs::create_dir_all(&save_dir).unwrap_or_else(|e| {
					eprintln!("Failed to create directory {}: {}", save_dir, e);
				});
				command.insert(0, "umu-run".to_string());
				command
			}
		};

		let mut overlay_src = game.overlays.clone();
		Self::found_and_replace(&mut overlay_src, &vars);
		let overlay = Some(Overlay{
			src: overlay_src,
			dst: format!("{}/{}", conf.user_save_dir.clone(), name)
		});

		if game.prestart.is_some() {
			let prestart_command = Self::get_abs_command(game.prestart.clone().unwrap_or_default());
			Self::junest_run(prestart_command, environ.clone(), &overlay)
				.spawn()
				.map_err(|e| anyhow!("Failed to run prestart command: {}", e))?;
		}

		Ok(Self::junest_run(command, environ, &overlay))
	}

	pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
		if let Some(game_process) = &mut self.running {
			game_process.process.kill()?;
			self.running = None;
		}
		Ok(())
	}
}
