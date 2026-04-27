
check_docker() {
    local ok=true

    # Check if docker is available
    if ! command -v docker &>/dev/null; then
        error "docker: command not found"
        return 1
    fi

    # Check if daemon is reachable
    if ! docker info &>/dev/null; then
        error "docker daemon: unreachable (not running or no permission)"
        return 1
    fi

    # Check if daemon runs as root
    local rootless
    rootless=$(docker info --format '{{.SecurityOptions}}' 2>/dev/null)
    if echo "$rootless" | grep -q "rootless"; then
    	warn "docker daemon: running in rootless mode"
    else
		info "docker daemon: running as root"
    fi

    $ok
}

check_fuse() {
    # Check if /dev/fuse exists
    if [ ! -e /dev/fuse ]; then
        error "fuse: /dev/fuse not found"
        return 1
    fi

    # Check if we can read/write /dev/fuse
    if [ ! -r /dev/fuse ] || [ ! -w /dev/fuse ]; then
        error "fuse: no read/write access to /dev/fuse"
        return 1
    fi

    # Check if fusermount is available
    if command -v fusermount &>/dev/null; then
        info "fuse: fusermount found ($(command -v fusermount))"
    elif command -v fusermount3 &>/dev/null; then
        info "fuse: fusermount3 found ($(command -v fusermount3))"
    else
        warn "fuse: fusermount not found (sshfs/fuse mounts may still work as root)"
    fi

	local mnt
	mnt=$(mktemp -d)
	if echo "" | timeout 2s $SSHFS -o slave : "$mnt" 2>&1 | grep -qv "fuse\|FUSE\|permission\|Permission"; then
		info "fuse: operational"
	else
		local err
		err=$(echo "" | timeout 2s $SSHFS -o slave : "$mnt" 2>&1 || true)
		if echo "$err" | grep -qi "fuse\|permission denied.*fuse\|/dev/fuse"; then
			error "fuse: not available — $err"
			return 1
		else
			info "fuse: operational (sftp error expected)"
		fi
	fi
	rmdir "$mnt" 2>/dev/null

    # Check kernel module
    if cat /proc/filesystems 2>/dev/null | grep -q fuse; then
        info "fuse: kernel module loaded"
    else
        warn "fuse: fuse not listed in /proc/filesystems"
    fi
}

