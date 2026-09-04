# UML diagrams

PlantUML source for the launch runtime architecture, one file per slice
instead of a single unreadable mega-diagram. Text, not a GUI project file —
diff and edit these like code.

| File | Covers |
|---|---|
| `00-overview.puml` | Crate dependency graph (matches the workspace `Cargo.toml`s) |
| `01-core-domain.puml` | `stdg-core`'s traits (`Layer`, `Runner`, ...), `Plan`, `LaunchCtx`, `CommandSpec`, container types |
| `02-core-config-and-errors.puml` | Config cascade types (`PartialGameConfig` → `ResolvedConfig`) and error enums |
| `03-runners.puml` | `Runner` implementations (native, Windows, emulator) |
| `04-layers.puml` | `Layer` implementations, one per slot — `<<stub>>` marks the remaining `todo!()` one (`SteamApiEmuLayer`, DLL injection). Sandbox, Runtime, and Compat (Proton/Wine) are all real |
| `05-plan-and-exec.puml` | `Planner`, `Registry`, `Pipeline`, cgroup/subreaper supervision |
| `06-cli.puml` | `stdg-cli` wiring and the `explain` command |

Each `.puml` file is self-contained (`@startuml`/`@enduml`) and renders on
its own.

## Rendering

Requires PlantUML (and a JVM); both are already installed here.

```sh
plantuml -tsvg docs/uml/*.puml   # or -tpng
```

Or, for live preview while editing: the "PlantUML" extension in VS Code /
JetBrains IDEs renders on save with no separate render step.

## Keeping these in sync with the code

There's no reverse-engineering step (Rust isn't supported by PlantUML's own
generators) — these were hand-modeled from `crates/*/src/**/*.rs` and need
updating by hand alongside real changes to public types, traits, and their
relationships. Realizations and dependencies are worth double-checking
whenever a `Layer`/`Runner` impl or a trait signature changes.
