//! Subreaper fallback when cgroups are unavailable: orphans reparent to us.

use std::io;

/// Marks the current process as a child subreaper.
#[expect(unsafe_code, reason = "prctl has no safe std equivalent")]
pub fn set_child_subreaper() -> io::Result<()> {
    // SAFETY: takes one integer argument; other varargs are ignored.
    let rc = unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Whether the current process is a child subreaper.
#[expect(unsafe_code, reason = "prctl has no safe std equivalent")]
pub fn is_child_subreaper() -> io::Result<bool> {
    let mut value: libc::c_int = 0;
    // SAFETY: writes one c_int through a pointer to a live local.
    let rc = unsafe {
        libc::prctl(
            libc::PR_GET_CHILD_SUBREAPER,
            (&raw mut value) as libc::c_ulong,
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
