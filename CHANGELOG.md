# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Save profiles (`stdg-saves`): save locations from the Ludusavi manifest and
  `[saves] paths`, bind-mounted from `~/.local/share/stdgames/saves/<game>/<profile>`.
  `stdgames saves update-manifest|list|new|use|export|import`, and `--profile`
  on `run` and `explain`. A profile is locked while a game uses it.
- MSRV is now 1.89 (`File::try_lock`).
- `Layer::wine_prefix`, so outer layers know where the Wine user dirs are.
- `steamapi-emu` replaces every Steam API library found in the game with the
  matching-arch replacement (`lib_x64`, `lib_x86`), for Windows and Linux
  games; `overlay_dir` optionally overlays extra files.
- `steamapi-onlinefix` overlays `overlay_dir` on the game dir (writes are kept
  in `overlay_state_dir`, default `~/.local/share/stdgames/overlays/<game>/<layer>`);
  `lib_x64`/`lib_x86` with `replace_lib = "true"` optionally replace the library.
- `BindMode::Overlay`, mounted by Conty with bwrap `--overlay`.
- `${env:NAME}`, `~`, `${game.root}`, `${game.id}`, `${game.executable}`,
  `${mode.id}` and `${runner.id}` expansion in every config string.
- `stdgames run --shell` opens a shell inside the wrapped environment.
- `SandboxLevel` type for Conty's `sandbox_level`, parsed at config load.
- Pinned toolchain, workspace lints, rustfmt/clippy/cargo-deny configs, CI.

### Changed

- The game dir is always read-only: the new `saves` layer (a mandatory
  `Saves` slot, filled in every plan) overlays it and sends every write to the
  current save profile.
- `steamapi-onlinefix` / `steamapi-emu` `overlay_dir` is now a read-only
  layer under the profile's writes; `overlay_state_dir` is removed.
- `Layer::container_needs` takes the `LaunchCtx`.
- Conty's user-namespace mode is no longer a `userns` layer param: it is on
  when fusermount is not setuid root, and `STDG_CONTY_USERNS=1`/`0` forces it.
- Missing required layer parameters (`proton_path`, `prefix_path`,
  `depot_path`, `app_id`, `replacement_lib_path`...) are now configuration
  errors instead of silently defaulting to empty values.
- An invalid `defaults/*.toml` file is now an error instead of a warning.
- `LaunchCtx::dry_run: bool` replaced by `LaunchCtx::run_kind: RunKind`.
- `ProtonLayer::version` / `WineLayer::version` are now `Option<String>`.
- `Layer`, `Runner` and `SessionGuard` now require `Debug`.
- The CLI reports full error chains; `run` exits with the game's exit code
  (or `128 + signal`).
- Warnings from the executor and layers go through `tracing`.

### Fixed

- `steamapi-emu`: `target_lib_relpath` was ignored when searching the game
  root for the library to replace.
- Supervision: failing to enable the subreaper fallback was silently ignored.
- `kill_all` could truncate a pid with an `as` cast and ignored `kill` errors.

### Removed

- `SteamApiEmuLayer`, `EmuVariant`, `SteamApiOnlineFixLayer` and the
  `replacement_lib_path` param (now `lib_x64`/`lib_x86`).
- The stale Tauri/npm `build.yml` workflow.
