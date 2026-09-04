//! Home-grown supervision, no external `reaper` binary: a dedicated cgroup
//! v2 leaf per session, so every descendant of the launched process —
//! including orphaned Wine helper processes reparented away from it — can
//! be found and killed together.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use stdg_core::SessionGuard;

pub struct CgroupSession {
    path: PathBuf,
}

impl CgroupSession {
    /// `parent` must be a delegated cgroup v2 directory the caller already
    /// has write access to (typically the user's systemd --user scope,
    /// e.g. `/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice`).
    /// Returns `Ok(None)` — not an error — when cgroup v2 delegation is not
    /// usable here, so the caller can fall back to
    /// `subreaper::set_child_subreaper`.
    pub fn create(parent: &Path, session_name: &str) -> io::Result<Option<Self>> {
        if !is_cgroup_v2_delegated(parent) {
            return Ok(None);
        }
        let path = parent.join(format!("stdgames-{session_name}"));
        fs::create_dir(&path)?;
        Ok(Some(Self { path }))
    }

    pub fn add_pid(&self, pid: u32) -> io::Result<()> {
        fs::write(self.path.join("cgroup.procs"), pid.to_string())
    }

    pub fn pids(&self) -> io::Result<Vec<u32>> {
        let content = fs::read_to_string(self.path.join("cgroup.procs"))?;
        Ok(content.lines().filter_map(|line| line.trim().parse().ok()).collect())
    }

    /// Sends SIGKILL to every process still in the cgroup. Best-effort: a
    /// process that exited between the read and the kill is not an error.
    pub fn kill_all(&self) -> io::Result<()> {
        for pid in self.pids()? {
            // SAFETY: kill() is called with a plain pid_t and a signal
            // number, no memory is shared with the kernel.
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGKILL);
            }
        }
        Ok(())
    }
}

fn is_cgroup_v2_delegated(parent: &Path) -> bool {
    parent.join("cgroup.controllers").is_file() && parent.join("cgroup.procs").is_file()
}

impl Drop for CgroupSession {
    fn drop(&mut self) {
        let _ = self.kill_all();
        let _ = fs::remove_dir(&self.path);
    }
}

impl SessionGuard for CgroupSession {
    fn label(&self) -> &str {
        "cgroup-session"
    }

    fn adopt_pid(&self, pid: u32) {
        if let Err(e) = self.add_pid(pid) {
            eprintln!("warning: could not add pid {pid} to cgroup {}: {e}", self.path.display());
        }
    }
}
