#export RUSTC_WRAPPER=sccache
RUSTFLAGS="-Z threads=$(nproc)"
export CARGO_BUILD_JOBS=$(nproc)
npm run tauri dev