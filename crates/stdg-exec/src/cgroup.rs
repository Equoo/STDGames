//! Per-session cgroup v2 leaf, so the whole process tree can be killed.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use stdg_core::SessionGuard;

/// A per-session cgroup v2 leaf, killed and removed on drop.
#[derive(Debug)]
pub struct CgroupSession {
    path: PathBuf,
}

impl CgroupSession {
    /// `parent` must be a delegated, writable cgroup v2 directory.
    pub fn create(parent: &Path, session_name: &str) -> io::Result<Option<Self>> {
        if !is_cgroup_v2_delegated(parent) {
            return Ok(None);
        }
        let path = parent.join(format!("stdgames-{session_name}"));
        fs::create_dir(&path)?;
        Ok(Some(Self { path }))
    }

    /// Moves process `pid` into this cgroup.
    pub fn add_pid(&self, pid: u32) -> io::Result<()> {
        fs::write(self.path.join("cgroup.procs"), pid.to_string())
    }

    /// The processes currently in this cgroup.
    pub fn pids(&self) -> io::Result<Vec<u32>> {
        let content = fs::read_to_string(self.path.join("cgroup.procs"))?;
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.trim().parse().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("bad pid {line:?} in cgroup.procs: {e}"),
                    )
                })
            })
            .collect()
    }

    /// Sends SIGKILL to every process still in the cgroup.
    pub fn kill_all(&self) -> io::Result<()> {
        for pid in self.pids()? {
            kill(pid, libc::SIGKILL)?;
        }
        Ok(())
    }
}

/// Sends `signal` to `pid`. A process that no longer exists is not an error.
#[expect(unsafe_code, reason = "libc::kill has no safe std equivalent")]
fn kill(pid: u32, signal: libc::c_int) -> io::Result<()> {
    let pid = libc::pid_t::try_from(pid).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("pid {pid} out of range"),
        )
    })?;
    // SAFETY: kill() takes two integers and shares no memory.
    let rc = unsafe { libc::kill(pid, signal) };
    if rc == 0 {
        return Ok(());
    }
    let err = io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::ESRCH) {
        return Ok(());
    }
    Err(err)
}

fn is_cgroup_v2_delegated(parent: &Path) -> bool {
    parent.join("cgroup.controllers").is_file() && parent.join("cgroup.procs").is_file()
}

impl Drop for CgroupSession {
    fn drop(&mut self) {
        // Drop can't propagate errors, so log them.
        if let Err(e) = self.kill_all() {
            tracing::warn!(
                "could not kill processes in cgroup {}: {e}",
                self.path.display()
            );
        }
        if let Err(e) = fs::remove_dir(&self.path) {
            tracing::warn!("could not remove cgroup {}: {e}", self.path.display());
        }
    }
}

impl SessionGuard for CgroupSession {
    fn label(&self) -> &'static str {
        "cgroup-session"
    }

    fn adopt_pid(&self, pid: u32) {
        if let Err(e) = self.add_pid(pid) {
            tracing::warn!(
                "could not add pid {pid} to cgroup {}: {e}",
                self.path.display()
            );
        }
    }
}
