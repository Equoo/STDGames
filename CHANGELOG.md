# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `${env:NAME}`, `~`, `${game.root}`, `${game.id}`, `${game.executable}`,
  `${mode.id}` and `${runner.id}` expansion in every config string.
- `steamapi-onlinefix` layer (preflight only; file overlay still to do).
- `stdgames run --shell` opens a shell inside the wrapped environment.
- `SandboxLevel` type for Conty's `sandbox_level`, parsed at config load.
- Pinned toolchain, workspace lints, rustfmt/clippy/cargo-deny configs, CI.

### Changed

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

- The stale Tauri/npm `build.yml` workflow.
