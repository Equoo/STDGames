# STDGames launcher runtime

Plans and launches games (native Linux, Windows through Proton/Wine, ROMs
through emulators) inside a Conty sandbox, with optional Steam Linux
Runtime, Steam API emulation and cgroup supervision layers.

A launch is a **plan**: one *runner* builds the base command, and at most one
*layer* per *slot* (Sandbox, Runtime, Compat, SteamApi, Supervision,
Tooling) wraps it. Plans are resolved from a cascade of TOML files:
`defaults/global.toml` → `defaults/runners/<runner>.toml` → the game's own
file → the selected mode.

## Layout

| Crate | Role |
| --- | --- |
| `stdg-core` | Data model and traits, no I/O |
| `stdg-plan` | Config cascade and plan validation |
| `stdg-registry` | Runner/layer catalog |
| `stdg-exec` | Pipeline execution, process spawning, cgroups |
| `stdg-runners-*` | Native, Windows and emulator runners |
| `stdg-layers-*` | Sandbox, runtime, compat, Steam API and supervision layers |
| `stdg-cli` | The `stdgames` binary |


## Build

The toolchain is pinned in `rust-toolchain.toml`; `rustup` installs it on
first use.

```sh
cargo build --release
```

## Run

From the repository root (paths are resolved relative to it):

```sh
# Print the resolved plan without launching anything
cargo run -p stdg-cli -- explain --game windows-example --mode desktop

# Launch for real
cargo run -p stdg-cli -- run --game repo --mode steam

# Open a shell inside the fully wrapped environment instead of the game
cargo run -p stdg-cli -- run --game repo --mode steam --shell
```

Game configs are looked up in `games/` then `examples/games/`.

### Steam API modes

| Layer | Library replaced | Directory overlay |
| --- | --- | --- |
| `steamapi-native` | no | no |
| `steamapi-emu` | yes (`replace_lib`, default `true`) | optional (`overlay_dir`) |
| `steamapi-onlinefix` | optional (`replace_lib`, default `false`) | yes (`overlay_dir`, required) |

Every `steam_api64.dll`/`steam_api.dll` (Windows) or `libsteam_api.so` (Linux)
in the game is replaced by `lib_x64` or `lib_x86`, picked from its ELF/PE header;
`target_lib_relpath` pins a single file. The overlay is mounted on the game dir
(needs bwrap ≥ 0.9 in Conty); the game dir is never written, writes go to
`overlay_state_dir` (default `~/.local/share/stdgames/overlays/<game>/<layer>`).

### Variables in config strings

Every string (`root`, `executable`, `args`, `env`, `command_prefix`, layer
params) is expanded after the cascade:

| Syntax | Value |
| --- | --- |
| `${env:NAME}`, leading `~` | environment variable / `$HOME` |
| `${game.id}`, `${game.root}`, `${game.executable}` | resolved game config |
| `${mode.id}`, `${runner.id}` | the plan |
| `$$` | a literal `$` |

```toml
prefix_path = "~/.local/share/stdgames/prefixes/${game.id}"
```

An unknown or unset variable is a configuration error.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). CI runs exactly:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
cargo machete
```
