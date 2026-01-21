
use std::collections::HashMap;
use std::os::unix::process::CommandExt;

use clap::{Parser, Subcommand};
use anyhow::Result;
use std::io::{self, Write};

use crate::config::CONFIG;
use crate::library::Game;
use crate::execution::{GameExecution, GameProcess};

#[derive(Parser, Debug)]
#[command(
	name = "stdgames",
	version,
	about = "Stdgames launcher by zsonie, tdaclin and dderny.",
	author = "zsonie, tdaclin, dderny"
)]
pub struct Cli {
	/// Use a custom config file
	#[arg(short, long, value_name = "FILE")]
	pub config: Option<String>,

	#[command(subcommand)]
	pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Run a game from the stdgames repository
	Run {
		/// Game name
		game: String,
	},

	/// Run bash with the game's config
	Bash {
		/// Game name
		game: String,
	},

	/// Run a game with a custom config file
	RunConfig {
		/// Path to TOML config file
		file: String,
	},

	/// Run bash with a custom config file
	BashConfig {
		/// Path to TOML config file
		file: String,
	},

	/// Enter the Junest environment
	Junest,
}

fn get_game<'a>(library: &'a Vec<Game>, name: &'a String) -> Result<&'a Game> {
	library.iter()
		.find(|g| &g.slug == name)
		.ok_or_else(|| anyhow::anyhow!("Game '{}' not found in library", name))
}









use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use std::collections::HashSet;


use x11::xlib::*;
use std::ptr;
use std::ffi::CString;

#[derive(Debug)]
struct WindowInfo {
    id: Window,
    pid: u32,
    width: u32,
    height: u32,
    area: u32,
}

fn get_largest_window_x11(root_pid: u32) -> Option<String> {
    let all_pids = get_all_descendant_pids(root_pid);
    
    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return None;
        }
        
        let root = XDefaultRootWindow(display);
        let atom_pid = XInternAtom(
            display,
            CString::new("_NET_WM_PID").unwrap().as_ptr(),
            0
        );
        
        let mut windows = Vec::new();
        find_all_windows_by_pids(display, root, atom_pid, &all_pids, &mut windows);
        
        XCloseDisplay(display);
        
        if windows.is_empty() {
            return None;
        }
        
        // Find the window with the largest area
        let largest = windows.iter().max_by_key(|w| w.area)?;
        
        println!("Found {} windows, selecting largest:", windows.len());
        println!("  Window ID: 0x{:x}", largest.id);
        println!("  PID: {}", largest.pid);
        println!("  Size: {}x{} (area: {})", largest.width, largest.height, largest.area);
        
        Some(format!("0x{:x}", largest.id))
    }
}

unsafe fn find_all_windows_by_pids(
    display: *mut Display,
    window: Window,
    atom_pid: Atom,
    target_pids: &HashSet<u32>,
    results: &mut Vec<WindowInfo>,
) {
    // Check this window
    let mut actual_type = 0;
    let mut actual_format = 0;
    let mut nitems = 0;
    let mut bytes_after = 0;
    let mut prop: *mut u8 = ptr::null_mut();
    
    unsafe {
        if XGetWindowProperty(
            display,
            window,
            atom_pid,
            0,
            1,
            0,
            XA_CARDINAL,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop,
        ) == 0 && !prop.is_null()
        {
            let window_pid = *(prop as *const u32);
            XFree(prop as *mut _);
            
            if target_pids.contains(&window_pid) {
                // Get window geometry
                if let Some((width, height)) = get_window_geometry_x11(display, window) {
                    results.push(WindowInfo {
                        id: window,
                        pid: window_pid,
                        width,
                        height,
                        area: width * height,
                    });
                }
            }
        }
        
        // Check children recursively
        let mut root_return = 0;
        let mut parent_return = 0;
        let mut children: *mut Window = ptr::null_mut();
        let mut nchildren = 0;
        
        if XQueryTree(
            display,
            window,
            &mut root_return,
            &mut parent_return,
            &mut children,
            &mut nchildren,
        ) != 0 && !children.is_null()
        {
            for i in 0..nchildren {
                let child = *children.offset(i as isize);
                find_all_windows_by_pids(display, child, atom_pid, target_pids, results);
            }
            XFree(children as *mut _);
        }
    }
}

unsafe fn get_window_geometry_x11(display: *mut Display, window: Window) -> Option<(u32, u32)> {
    let mut root_return = 0;
    let mut x_return = 0;
    let mut y_return = 0;
    let mut width_return = 0;
    let mut height_return = 0;
    let mut border_width_return = 0;
    let mut depth_return = 0;
    
    unsafe {
        if XGetGeometry(
            display,
            window,
            &mut root_return,
            &mut x_return,
            &mut y_return,
            &mut width_return,
            &mut height_return,
            &mut border_width_return,
            &mut depth_return,
        ) != 0
        {
            Some((width_return, height_return))
        } else {
            None
        }
    }
}

fn get_all_descendant_pids(pid: u32) -> HashSet<u32> {
    let mut pids = HashSet::new();
    pids.insert(pid);
    
    if let Ok(output) = std::process::Command::new("pgrep")
        .args(&["-P", &pid.to_string()])
        .output()
    {
        if output.status.success() {
            let children = String::from_utf8_lossy(&output.stdout);
            for child_pid_str in children.lines() {
                if let Ok(child_pid) = child_pid_str.parse::<u32>() {
                    let descendants = get_all_descendant_pids(child_pid);
                    pids.extend(descendants);
                }
            }
        }
    }
    
    pids
}

// Usage with polling:
fn wait_for_largest_window_x11(root_pid: u32, timeout_secs: u64) -> Option<String> {
    let start = std::time::Instant::now();
    
    while start.elapsed().as_secs() < timeout_secs {
        if let Some(window_id) = get_largest_window_x11(root_pid) {
            return Some(window_id);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    
    None
}















pub fn init_cli(
	cli: &Cli,
	library: &Vec<Game>,
	_game_exec: &mut GameExecution,
) -> Result<()> {
	GameExecution::setup(|progress: f32| {
		let progress = progress / 100.0;
		let filled = (progress * 50.0) as usize;
		let empty = 50 - filled;
		
		print!("\rSetup progress: [{}{}] {:.1}%", 
			"█".repeat(filled),
			"░".repeat(empty),
			progress * 100.0);
		
		io::stdout().flush().unwrap();
	})?;

	let vars = CONFIG.clone().build_vars();

	match &cli.command {
		Some(Commands::Run { game }) => {
			let mut launch_data = get_game(library, &game)?.launch.clone();
			launch_data.replace_vars(&vars);
			
			let mut child = GameExecution::build_command(&game, &launch_data)?
				.spawn()?;
            
            let pid = child.id();
            thread::sleep(Duration::from_millis(30000));

            match unsafe {  get_largest_window_x11(pid) } {
                Some(window_id) => {
                    println!("window id: {}", window_id);

                    // use it with ffmpeg
                    let _ffmpeg = Command::new("ffmpeg")
                        .args(&[
                            "-f", "x11grab",
                            "-framerate", "60",
                            "-window_id", &window_id,
                            "-i", ":0.0",
                            "-pix_fmt", "yuv420p",
                            "-vsync", "0",
                            "-fflags", "nobuffer",
                            "-c:v", "libx264",
                            "-preset", "ultrafast",
                            "-tune", "zerolatency",
                            "-bf", "0",
                            "-g", "15",
                            "-keyint_min", "15",
                            "-sc_threshold", "0",
                            "-b:v", "16m",
                            "-maxrate", "16m",
                            "-bufsize", "16m",
                            "-x264-params", "repeat-headers=1:rc-lookahead=0:no-scenecut=1:vbv-init=0",
                            "-f", "rtsp",
                            "-rtsp_transport", "tcp",
                            "rtsp://127.0.0.1:8554/1"
                        ])
                        .spawn()
                        .expect("Failed to start ffmpeg");
                }
                None => eprintln!("Could not find window for PID {}", pid),
            }

            // Wait for child to finish
            let _ = child.wait();

		}
		Some(Commands::Bash { game }) => {
			let mut launch_data = get_game(library, &game)?.launch.clone();
			launch_data.start = ["/bin/bash".to_string()].to_vec();
			launch_data.noruntime = Some(true);
			launch_data.replace_vars(&vars);
			
			let err = GameExecution::build_command(&game, &launch_data)?
				.exec();
			println!("Error running bash: {}", err);
		}
		Some(Commands::RunConfig { file }) => {
			println!("Running game with config file: {}", file);
		}
		Some(Commands::BashConfig { file }) => {
			println!("Running bash with config file: {}", file);
		}
		Some(Commands::Junest) => {
			let err = GameExecution::junest_cmd(HashMap::new(), &None)
				.arg("/bin/bash")
				.exec();
			println!("Error running Junest: {}", err);
		}
		None => {}
	}

	Ok(())
}
