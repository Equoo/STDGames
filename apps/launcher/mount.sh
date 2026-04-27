
mount_filesystem() {
	header "Mounting server filesystem"

	step 1 "Unmount old connections" # TODO Use old connection, check if work else umount and new

	mount | grep "@$SSHFS_HOST:" | awk '{print $3}' | while read -r mountpoint; do
		debug "Unmounting $mountpoint"
		fusermount3 -u "$mountpoint"
	done

	step 2 "Mounting"
	debug "Mounting at $DATA_DIR"

	mkdir -p $DATA_DIR
	mkdir -p $HOME/.ssh
	cp $SSHKEY $HOME/.ssh/stdgame
    chmod 600 $HOME/.ssh/stdgame
    chmod 700 $HOME/.ssh

	if [ $SSH ]; then
		debug "Trying to mount from ssh"
		output=$(
			$SSHFS -p $SSHFS_PORT $SSHFS_USER@$SSHFS_HOST:$HOST_DIR $DATA_DIR \
				-o ssh_command="ssh -i $HOME/.ssh/stdgame" \
				-o MACs=umac-64-etm@openssh.com \
				-o StrictHostKeyChecking=no \
				-o UserKnownHostsFile=/dev/null \
				-o Ciphers=aes128-gcm@openssh.com \
				-o Compression=no \
				-o cache=yes \
				-o kernel_cache \
				-o cache_timeout=86400 \
				-o attr_timeout=86400 \
				-o entry_timeout=86400 \
				-o max_read=524288 \
				-o ServerAliveInterval=15 \
				-o dcache_max_size=10000 \
				-o NumberOfPasswordPrompts=0 \
				-o ControlMaster=auto \
				-o ControlPath=/tmp/sshfs-ctl-%r@%h:%p \
				-o ControlPersist=yes \
				-o max_conns=8 \
				-o reconnect 2>&1
		)
		if [ $? -ne 0 ]; then
			fatal "$output"
		fi
	else
		debug "Trying to mount without ssh"
		# socat tcp-listen:1234,reuseaddr,fork  exec:/usr/lib/openssh/sftp-server
		$SSHFS -o directport=$PORT $HOST:$FOLDER $DEST
	fi

	success "Mounting complete"
}
