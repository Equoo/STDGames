#export RUSTC_WRAPPER=sccache
RUSTFLAGS="-Z threads=$(nproc)"
export CARGO_BUILD_JOBS=$(nproc)
cd src-tauri
#cargo build
cargo run
#npm run tauri dev