
DATA_DIR=${DATA_DIR:-$(mktemp -d)}
LAUNCHER_BIN=$BASEDIR/launcher.bin
CACHE=$HOME/.cache/stdgame
JUNEST_DIR=$CACHE/junest
JUNEST_HOME=$CACHE/junest_home
JUNEST=$JUNEST_DIR/bin/junest
SSHFS=${SSHFS:-$BASEDIR/sshfs-bundle/sshfs}
SSHKEY=$BASEDIR/keys/ssh_key

SSH=1
SSHFS_PORT=44424
SSHFS_HOST=games.a2itech.fr
SSHFS_USER=std
HOST_DIR=/shared
