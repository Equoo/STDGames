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

### Conty and user namespaces

Conty mounts its image with FUSE. When `fusermount3` (or `fusermount`) is not
setuid root, the launcher runs Conty through `unshare` so it can mount without
it. Set `STDG_CONTY_USERNS=1` or `0` to force this on or off. On the Nvidia
proprietary driver, user-namespace mode needs a Conty built with
`../container`'s `patch-conty.sh`.

### Steam API modes

| Layer | Library replaced | Directory overlay |
| --- | --- | --- |
| `steamapi-native` | no | no |
| `steamapi-emu` | yes (`replace_lib`, default `true`) | optional (`overlay_dir`) |
| `steamapi-onlinefix` | optional (`replace_lib`, default `false`) | yes (`overlay_dir`, required) |

Every `steam_api64.dll`/`steam_api.dll` (Windows) or `libsteam_api.so` (Linux)
in the game is replaced by `lib_x64` or `lib_x86`, picked from its ELF/PE header;
`target_lib_relpath` pins a single file. `overlay_dir` is laid read-only over
the game dir (needs bwrap ≥ 0.9 in Conty); writes go to the save profile.

### Save profiles

The game dir is never written: it is mounted as an overlay and every write
(Source `.cfg` files, logs...) lands in the current profile. Save locations
come from the [Ludusavi manifest](https://github.com/mtkennerly/ludusavi-manifest)
(matched by `app_id`), then from the game's `[saves]` section, and are
bind-mounted from the profile.

```toml
[saves]
manifest_name = "R.E.P.O."                   # default: looked up by app_id
paths = ["<winAppData>/Studio/Game"]          # Ludusavi syntax, added to the manifest's
profile = "main"                              # default: the current profile
```

Profiles live in `~/.local/share/stdgames/saves/<game>/<profile>/`: `root/`
holds the game dir writes, `paths/` the save dirs. The `default` profile adopts
saves already on disk the first time it runs; other profiles start empty.
A profile is locked while a game uses it: a second launch, `new --from` or
`export` on it fails until the game exits.

```sh
cargo run -p stdg-cli -- saves update-manifest      # download the manifest (needs curl)
cargo run -p stdg-cli -- saves list --game repo
cargo run -p stdg-cli -- saves new --game repo --name speedrun --from default
cargo run -p stdg-cli -- saves use --game repo --name speedrun
cargo run -p stdg-cli -- saves export --game repo --name speedrun --out sr.tar.gz
cargo run -p stdg-cli -- saves import --game repo --name shared --file sr.tar.gz
cargo run -p stdg-cli -- run --game repo --mode steam --profile shared
```

Steam Cloud files (`<root>/userdata/...` in the manifest) are not synced yet.

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
