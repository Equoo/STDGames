use std::{collections::HashMap, path::PathBuf, process::Command};
use uzers::get_current_uid;

use crate::{
    config::CONFIG,
    debug::log_step,
    execution::{GameExecution, Overlay},
    utils::copy_savedata,
};

impl GameExecution {
    pub fn junest_cmd(environ: HashMap<String, String>, overlay: &Option<Overlay>) -> Command {
        log_step("junest_cmd", format!("junest_bin={} overlay_src={:?} overlay_dst={:?}",
            CONFIG.junest_bin,
            overlay.as_ref().map(|o| &o.src),
            overlay.as_ref().map(|o| &o.dst),
        ));

        let mut cmd = Command::new(CONFIG.junest_bin.clone());

        let work_dir = if let Some(overlay) = overlay {
            overlay.src.iter().for_each(|src| {
                match copy_savedata(src, &overlay.dst) {
                    Ok(stats) => log_step("junest_cmd", format!("copy_savedata {} -> {}: {:?}", src, overlay.dst, stats)),
                    Err(e) => log_step("junest_cmd", format!("copy_savedata {} -> {} FAILED: {}", src, overlay.dst, e)),
                }
            });
            PathBuf::from(format!("/tmp/{}/stdgames/work", CONFIG.username))
      } else {
            PathBuf::from(&CONFIG.user_home)
        };

        cmd.env("JUNEST_HOME", CONFIG.temp_junest_home_dir.clone());
        cmd.env("PYTHONPATH", "/usr/lib/python3/dist-packages");
        cmd.envs(environ).current_dir(work_dir);

        let user = CONFIG.username.clone();
        let overlay_str = if let Some(o) = overlay {
            // Ensure the overlay directories exist. These are host-side paths:
            // bwrap's --overlay takes RWSRC/WORKDIR as outside-the-sandbox
            // paths (resolved here, on the host, before the sandbox exists),
            // so no /tmp/{user}/{user}/... doubling is needed for them - only
            // the overlay DEST argument is sandbox-relative (see the bare
            // "/tmp/stdgames/work" above).
            [
                &format!("/tmp/{user}"),
                &format!("/tmp/{user}/stdgames/rw"),
                &format!("/tmp/{user}/stdgames/overlay_work"),
                &format!("/tmp/{user}/stdgames/work"),
            ]
            .iter()
            .for_each(|folder| {
                std::fs::create_dir_all(folder).unwrap_or_else(|e| {
                    log_step("junest_cmd", format!("FAILED to create directory {}: {}", folder, e));
                });
            });

            format!(
                "--overlay-src {}
				--overlay /tmp/{user}/stdgames/rw /tmp/{user}/stdgames/overlay_work /tmp/stdgames/work",
                o.src.join(" --overlay-src ")
            )
        } else {
            String::new()
        };
        log_step("junest_cmd", format!("overlay_str: {}", overlay_str.replace('\n', " ").trim()));

        let uid = get_current_uid();
        cmd.arg("-b").arg(format!(
            "--bind /sgoinfre /sgoinfre
				--uid {uid}
				--bind /goinfre /goinfre
				--bind /media /media
				--bind /tmp/{} /tmp
				--bind {} /usr
				--bind /tmp/.X11-unix /tmp/.X11-unix
				--bind /run/user/{uid}/pulse/native /run/pulse/native
				{overlay_str}
			",
            CONFIG.username, CONFIG.junest_bind
        ));

        if overlay.is_some() {
            cmd.arg(CONFIG.overlay.clone());
            cmd.arg(format!("/tmp/stdgames/rw")); // really weird
            cmd.arg(overlay.as_ref().unwrap().dst.clone());
        }

        log_step("junest_cmd", format!("built command: {:?}", cmd));

        cmd
    }
}

