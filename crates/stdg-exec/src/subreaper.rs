//! Fallback supervision path used when delegated cgroups (`cgroup.rs`) are
//! not available: marking the current process PR_SET_CHILD_SUBREAPER makes
//! the kernel re-parent orphaned descendants (e.g. a Wine helper process
//! whose immediate parent died) to us instead of to PID 1, so they stay
//! reachable for a clean kill on session teardown.

use std::io;

pub fn set_child_subreaper() -> io::Result<()> {
    // SAFETY: PR_SET_CHILD_SUBREAPER takes a single integer argument (1 to
    // enable); the remaining prctl varargs are ignored by the kernel for
    // this option.
    let rc = unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub fn is_child_subreaper() -> io::Result<bool> {
    let mut value: libc::c_int = 0;
    // SAFETY: PR_GET_CHILD_SUBREAPER writes the current flag into the
    // pointer given as the second argument.
    let rc = unsafe {
        libc::prctl(
            libc::PR_GET_CHILD_SUBREAPER,
            &mut value as *mut libc::c_int as libc::c_ulong,
            0,
            0,
            0,
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(value != 0)
}
