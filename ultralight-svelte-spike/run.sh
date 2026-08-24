#!/usr/bin/env bash
# Runs the built ultralight-svelte-spike binary with everything it needs:
# - LD_LIBRARY_PATH pointing at the Ultralight SDK's shared libraries
#   (downloaded by ul-next-sys's build script; not on the linker path by default)
# - a working directory containing resources/ (SDK data) and svelte-app/
#   (symlink to renderer/dist/), since the app's platform filesystem is
#   rooted at "." and looks for both there
# - a workaround for a stray __EGL_VENDOR_LIBRARY_FILENAMES env var some
#   shells in this environment have set, which points at an NVIDIA ICD on a
#   system actually running AMD/Intel Mesa and crashes EGL init otherwise
set -euo pipefail

source /goinfre/dderny/putt/cargo-env.sh

TARGET_RELEASE="$CARGO_TARGET_DIR/debug"
BIN="$TARGET_RELEASE/ultralight-svelte-spike"

if [ ! -x "$BIN" ]; then
	echo "Binary not built yet. Run: (cd $(dirname "$0") && cargo build --release)" >&2
	exit 1
fi

SDK_DIR=$(find "$TARGET_RELEASE/build" -maxdepth 1 -iname "ul-next-sys-*" \
	-exec test -e "{}/out/ul-sdk/bin/libUltralight.so" \; -print -quit)/out/ul-sdk

if [ ! -f "$SDK_DIR/bin/libUltralight.so" ]; then
	echo "Could not find the downloaded Ultralight SDK under $TARGET_RELEASE/build/ul-next-sys-*/out/ul-sdk" >&2
	echo "Try: (cd $(dirname "$0") && cargo build --release) to trigger the SDK download." >&2
	exit 1
fi

if [ ! -e "$TARGET_RELEASE/svelte-app" ]; then
	ln -sfn /goinfre/dderny/putt/renderer/dist "$TARGET_RELEASE/svelte-app"
fi
if [ ! -d "$TARGET_RELEASE/resources" ]; then
	mkdir -p "$TARGET_RELEASE/resources"
	cp "$SDK_DIR/resources/"* "$TARGET_RELEASE/resources/"
fi

cd "$TARGET_RELEASE"
exec env \
	-u __EGL_VENDOR_LIBRARY_FILENAMES \
	-u DRI_PRIME \
	__EGL_VENDOR_LIBRARY_DIRS=/usr/share/glvnd/egl_vendor.d \
	LD_LIBRARY_PATH="$SDK_DIR/bin" \
	"$BIN"
