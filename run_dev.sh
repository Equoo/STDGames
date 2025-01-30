#export RUSTC_WRAPPER=sccache
export CARGO_BUILD_JOBS=$(nproc)
npm run tauri dev