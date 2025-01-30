use std::env;
use std::fs;
use std::io;
use std::collections::HashMap;
use serde::Deserialize;
use once_cell::sync::Lazy;
use std::process::{Child, Command};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::os::unix::fs::symlink;
use std::path::Path;

const S: &str = " ";
const JUNEST_PATH: &str = "/sgoinfre/dderny/junest/bin/junest";
const UMU_PATH: &str = "umu-run";
const LEGENDARY_LAUNCH: &str = "legenday launch";
const GAMES_PATH: &str = "/sgoinfre/42GamingNight/.STDGames/";
const DATAUSER_PATH: &str = "/sgoinfre/.stdgames";

fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source path {:?} does not exist", src),
        ));
    }

    if src.is_file() {
        // If src is a file, directly copy it to the destination.
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?; // Ensure parent directory exists.
        }
        fs::copy(src, dst)?;
    } else if src.is_dir() {
        // If src is a directory, create the corresponding directory at the destination.
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = dst.join(entry.file_name());

            // Recursively copy each entry.
            copy_recursively(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConfigPath {
	original: String,
	user: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GameInfo {
	name: String,
	display_name: String,
	cover: String,
	genres: String,
	launch_type: String,
	env: HashMap<String, String>,
	proton:	String,
	exec_path:	String,
	config: Vec<ConfigPath>,
}

struct GameLibrary {
	games: Vec<GameInfo>,
}

impl GameLibrary {
	fn new() -> Result<GameLibrary, Box<dyn std::error::Error>> {
        let file_content = match fs::read_to_string("/sgoinfre/dderny/games.json") {
            Ok(content) => content,
            Err(e) => {
                println!("Error reading file: {}", e);
                return Err(Box::new(e));
            }
        };
        let games: Vec<GameInfo> = serde_json::from_str(&file_content)?;
        Ok(GameLibrary { games })
    }

	fn get_game(&self, name: &str) -> Option<&GameInfo> {
        self.games.iter().find(|game| game.name == name)
    }
}

struct Launcher {
	running_process: Arc<Mutex<Option<Child>>>,
	library: Arc<GameLibrary>,
}

impl Launcher {
	fn new(library: Arc<GameLibrary>) -> Self {
		Launcher {
			running_process: Arc::new(Mutex::new(None)),
			library: library
		}
	}

	fn launch_game(&self, game: &str) -> Result<(), Box<dyn std::error::Error>> {
		let	data: &GameInfo = match self.library.get_game(game) {
            Some(data) => data,
            None => {
                println!("Game '{}' not found!", game);
                return Ok(());
            },
        };

		let junest_path = env::var("HOME").unwrap_or("".to_string()) + "/.junest";
		if !Path::new(&junest_path).exists() {
			let _ = symlink("/sgoinfre/dderny/.junest", junest_path);
		}

		for conf in data.config.clone() {
			let user_path = Path::new(&conf.user);
			if !user_path.exists() {
				fs::create_dir_all(env::var("HOME").unwrap_or("".to_string()) + DATAUSER_PATH + &conf.user)?;
				copy_recursively(Path::new(&conf.original), user_path)?;
			}
		}

		let mut env_vars: HashMap<String, String> = data.env.clone();
		let mut binds: HashMap<String, String> = HashMap::new();

		let exec_path = GAMES_PATH.to_owned() + &data.name + &data.exec_path;
		let command = match data.launch_type.as_str() {
			"native" => &exec_path,
			"umu" => {
				env_vars.extend(HashMap::from([
					(String::from("PYTHONPATH"), env::var("PYTHONPATH").unwrap_or("".to_string()) + ":/usr/lib/python3/dist-packages"),
					(String::from("PROTONPATH"), "/sgoinfre/dderny/.steam/root/compatibilitytools.d/".to_string()),
					(String::from("WINEPREFIX"), env::var("HOME").unwrap_or("".to_string()) + "/sgoinfre/.stdgames/umu"),
					//("STORE", "egs"),
					//("GAMEID", "value3"),
					//("WINEDLLOVERRIDES", "value3"),
				]));

				binds.insert("/tmp/$USER".to_string(), "/tmp".to_string());
				binds.insert("/tmp/.X11-unix/X0".to_string(), "/tmp/$USER/.X11-unix/X0".to_string());

				&(UMU_PATH.to_owned() + S + &exec_path)
			},
			"epicgame" => {
				&(LEGENDARY_LAUNCH.to_owned() + S + &data.exec_path)
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

		println!("Launching game: {}", JUNEST_PATH.to_owned() + "bwrap --bind / / hello");
		let process = Command::new("sh")
        	.arg("-c")
			.arg(JUNEST_PATH.to_owned() + S + "-b \"--bind /sgoinfre /sgoinfre" + &bindsstr + "\"" + S + command)
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

	fn is_game_running(&self) -> bool {
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

static LAUNCHER: Lazy<Arc<Launcher>> = Lazy::new(|| {
    let library = Arc::new(GameLibrary::new().expect("Failed to create game library"));
    Arc::new(Launcher::new(library))
});

#[tauri::command]
fn launch_game(game: String) {
	let launcher = &LAUNCHER;
    let _ = launcher.launch_game(&game);
}

#[tauri::command]
fn game_state() -> bool {
	let launcher = &LAUNCHER;
    launcher.is_game_running()
}

#[tauri::command]
fn get_game_library() -> Vec<GameInfo> {
    let _launcher = &LAUNCHER;
    _launcher.library.games.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![game_state, get_game_library, launch_game])
		.run(tauri::generate_context!())
		.expect("Erreur lors du lancement de Tauri");
}
