# Rust Project Bible

## 1. Core principles

Every rule below comes from these seven. When a rule doesn't fit your case, fall back on them.

1. **The compiler is your reviewer.** Encode invariants in types so wrong code doesn't compile. A runtime check is the fallback.
2. **`main` is always green.** `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` pass on every commit.
3. **No silent failure.** Every error is handled, propagated with `?`, or explicitly ignored with a comment saying why.
4. **Make it work, make it right, then make it fast.** Optimise only what a profiler or benchmark points at.
5. **Small, boring, explicit.** Prefer plain functions and enums over clever generics, macros and trait towers.
6. **Every `unsafe`, `unwrap` and `clone` is a decision.** Each one must be justifiable in a code review.
7. **Reproducible builds.** Pinned toolchain, committed `Cargo.lock` for binaries, CI runs the same commands as you.

## 2. Toolchain & project layout

Pin the toolchain in the repo so everyone builds with the same compiler.

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.100.0"          # a stable version, bumped on purpose
components = ["rustfmt", "clippy"]
```

- Use the latest edition (`edition = "2024"`) for new crates.
- Set `rust-version` (MSRV) in `Cargo.toml` and test it in CI if you publish a library.
- Commit `Cargo.lock` for binaries and workspaces. For a library, committing it is fine too; CI should still test with the latest dependencies.
- More than one crate → a **Cargo workspace** with shared `[workspace.dependencies]`, `[workspace.lints]` and `[workspace.package]`. Member crates inherit with `foo.workspace = true`.
- Keep `main.rs` thin. Logic goes in `lib.rs` so it can be tested and reused.
- One module = one responsibility. Split a file once it passes roughly 500–800 lines or mixes concerns.
- Layout convention:

```
my-project/
├── Cargo.toml          # workspace root
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── deny.toml
├── crates/
│   ├── core/           # pure logic, no I/O
│   ├── cli/            # binary, argument parsing, wiring
│   └── ...
├── tests/              # integration tests
├── benches/            # criterion benchmarks
└── examples/
```

- Keep the core crate free of I/O where you can. Pure code is easier to test, fuzz and reuse.

## 3. Formatting & linting

Formatting is never discussed in review: `rustfmt` decides. Lints live in `Cargo.toml`, so everyone gets the same warnings.

- Run `cargo fmt` before every commit. CI runs `cargo fmt --all --check`.
- Run `cargo clippy --all-targets --all-features` locally. CI adds `-- -D warnings`.
- Don't put `#![deny(warnings)]` in source: a new compiler version would break builds. Deny warnings in CI only.
- To silence a lint, use `#[expect(clippy::lint_name, reason = "...")]` on the smallest possible item. It warns once the lint no longer fires, unlike `#[all

A good starting lint set:

```toml
# Cargo.toml (or [workspace.lints] in a workspace)
[lints.rust]
unsafe_code = "forbid"          # change to "deny" in the one crate that needs unsafe
missing_docs = "warn"           # libraries only
unused_must_use = "deny"

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
todo = "warn"
dbg_macro = "warn"
print_stdout = "warn"           # libraries: use tracing/log instead
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

- `pedantic` is noisy at first. Turn off individual lints that fight your codebase, each with a reason, rather than dropping the whole group.
- Tests can opt out of `unwrap_used`: add `allow-unwrap-in-tests = true` to `clippy.toml`.

## 4. Types, ownership & API design

Make invalid states unrepresentable. If a value can be wrong, give it a type that can't be.

**Types**

- Use newtypes for domain values: `struct UserId(u64)`, `struct Meters(f64)`. No bare `u64` flowing through ten functions.
- Use enums instead of booleans and magic strings: `enum Mode { Fast, Safe }`, not `fast: bool`.
- Parse, don't validate. Turn raw input into a checked type once, at the boundary (`TryFrom`, `FromStr`), and trust it afterwards.
- Use `Option<T>` for "may be absent", never sentinel values like `-1` or `""`.
- Put `#[must_use]` on functions whose result must not be ignored.
- Mark public enums and structs `#[non_exhaustive]` when you expect to add variants or fields.
- Derive what's cheap and correct: `Debug` always, then `Clone, PartialEq, Eq, Hash, Default` where they make sense.

**Ownership**

- Take arguments by reference: `&str` over `&String`, `&[T]` over `&Vec<T>`, `impl AsRef<Path>` for paths.
- Take ownership (`String`, `Vec<T>`) only when the function stores the value.
- Every `.clone()` should be deliberate. Cloning an `Arc` is cheap; cloning a large `Vec` in a loop is not.
- Fight the borrow checker with a better design, not with `Rc<RefCell<_>>` everywhere. Interior mutability is a last resort.
- Prefer iterators (`iter().map().filter().collect()`) to index loops: they skip bounds checks and read better.

**API surface**

- Everything is private by default. Use `pub(crate)` before `pub`.
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/): `as_`/`to_`/`into_` naming, `new` constructors, builders for more than 3–4 optional parameters.
- Implement standard traits (`From`, `Display`, `Iterator`, `Default`) instead of inventing ad-hoc methods.
- Use generics with trait bounds for static dispatch; use `dyn Trait` only when you need heterogeneous collections or smaller binaries.

## 5. Error handling

Use `Result` for errors a caller can handle. Reserve `panic!` for bugs: broken invariants that should never happen.

| Context | Use | Why |
| --- | --- | --- |
| Library crate | `thiserror`, one error `enum` per module or domain | Callers can `match` on variants |
| Binary / application | `anyhow` (or `eyre`) with `.context("...")` | Rich messages, no boilerplate |
| Tests | `unwrap()` / `expect()`, or `-> anyhow::Result<()>` | A panic *is* the failure report |
| Truly impossible case | `expect("why this cannot fail")` | The message documents the invariant |

- No `unwrap()` in library or production code paths. If you're sure it can't fail, use `expect` and say why.
- Propagate with `?`. Add context at each layer that knows something the error doesn't: `fs::read(&path).with_context(|| format!("reading config {}", path.display()))?`.
- Never return `Result<T, String>` or `Box<dyn Error>` from a library's public API.
- Never swallow an error: `let _ = do_thing();` needs a comment saying why it's OK to ignore.
- Error messages are lowercase, without trailing punctuation, and don't repeat the source error (chain it with `#[source]` / `#[from]`).
- Validate input at the boundary and return errors there. Deeper code can then rely on the types.
- Integer overflow panics in debug and wraps in release. Use `checked_*`, `saturating_*` or `wrapping_*` when it matters, and avoid `as` casts that can truncate: prefer `try_from`.

## 6. Unsafe code

The default is `unsafe_code = "forbid"`. Unsafe is allowed only where safe Rust can't express what's needed: FFI, hardware/MMIO, or a measured hot path.

- Isolate it in one small module or crate, behind a **safe** API. Callers must not be able to trigger UB through that API.
- Every `unsafe` block gets a `// SAFETY:` comment saying why each invariant holds. Every `unsafe fn` gets a `# Safety` doc section listing what the caller must guarantee.
- Enable `unsafe_op_in_unsafe_fn` (default in edition 2024) and `clippy::undocumented_unsafe_blocks`.
- Keep each block as small as possible: one operation, not a whole function.
- Run the tests of unsafe code under Miri: `cargo +nightly miri test`.
- For FFI, generate bindings with `bindgen` and put them in a `-sys` crate, then write a safe wrapper on top.
- Embedded / `no_std`: prefer the PAC/HAL crates' safe register API over raw pointer writes.

## 7. Concurrency & async

Use async for I/O-bound work with many connections, and threads (or `rayon`) for CPU-bound work. Don't mix them carelessly.

**Threads & shared state**

- Prefer message passing (`std::sync::mpsc`, `crossbeam-channel`, `tokio::sync::mpsc`) over shared mutable state.
- When you must share: `Arc<Mutex<T>>` or `Arc<RwLock<T>>`. Keep the lock scope tiny, and never hold a lock while doing I/O or calling unknown code.
- Always take several locks in the same order, to avoid deadlocks.
- Use atomics only for simple counters and flags. Use `Ordering::SeqCst` unless you can explain why a weaker ordering is correct.
- For data-parallel loops, use `rayon` (`par_iter()`) instead of hand-rolled threads.

**Async**

- Pick one runtime per project (usually `tokio`) and stick to it.
- Never block inside `async`: no `std::thread::sleep`, no blocking file or network calls, no heavy CPU loops. Use `tokio::task::spawn_blocking` for those.
- Don't hold a `std::sync::MutexGuard` across `.await`. Restructure the code, or use `tokio::sync::Mutex` if you truly must.
- Every spawned task has an owner. Keep the `JoinHandle` (or use a `JoinSet`), and handle its error and cancellation.
- Put timeouts on network calls (`tokio::time::timeout`).
- Remember that a future can be dropped at any `.await`. Code must stay consistent if it stops there (cancellation safety, especially inside `select!`).
- Use bounded channels so a slow consumer applies backpressure instead of eating memory.

## 8. Testing

Every bug fix comes with a test that failed before the fix. Every public function has at least one test or doc example.

| Kind | Where | Tests what |
| --- | --- | --- |
| Unit | `#[cfg(test)] mod tests` at the bottom of the file | Private logic, edge cases |
| Integration | `tests/*.rs` | The public API, as a user sees it |
| Doc tests | `///` examples | That the docs compile and stay true |
| Property | `proptest` / `quickcheck` | Invariants over random inputs (parsers, encoders, round-trips) |
| Fuzz | `cargo fuzz` | Parsers and anything fed untrusted bytes |
| Snapshot | `insta` | Large outputs (CLI output, generated code, serialised data) |
| UB check | `cargo +nightly miri test` | Any crate containing `unsafe` |

- Use `cargo nextest run` for faster, clearer runs; it also isolates each test in its own process.
- Name tests after behaviour: `parses_empty_input_as_none`, not `test1`.
- Tests are deterministic. No real network, no wall-clock time, no shared temp paths. Use `tempfile` and inject clocks and RNG seeds.
- Test the error paths, not only the happy path.
- Run `cargo test --all-features` and `cargo test --no-default-features` in CI if the crate has features.
- Measure coverage with `cargo llvm-cov` to find untested code, but don't chase a percentage.

## 9. Dependencies & supply chain

Every dependency is code you ship and maintain. Add one only when it saves real work and is well maintained.

- Before adding a crate, check its downloads, last release, open issues, maintainers and `unsafe` usage. Prefer the ecosystem's standard picks (`serde`, `tokio`, `clap`, `tracing`, `thiserror`, `anyhow`).
- Add with `cargo add`, which writes a sensible version requirement.
- Disable default features you don't use: `default-features = false, features = ["..."]`. This cuts compile time and attack surface.
- Run `cargo deny check` in CI: it covers advisories (RustSec), licences, banned crates and duplicate versions. `cargo audit` is the lighter alternative.
- Find unused dependencies with `cargo machete` (fast) or `cargo udeps` (nightly, more precise).
- Update deliberately: `cargo update` on a schedule, or let Dependabot / Renovate open PRs, and read the changelog for major bumps.
- Never use `*` or git dependencies in a published crate. Pin git dependencies to a `rev` everywhere else.
- Features must be additive: enabling a feature must never break code that compiled without it.

## 10. Performance

Measure before and after every optimisation. A change that isn't faster in a benchmark gets reverted.

- Benchmark with `criterion` (or `divan`) in `benches/`. Never time a debug build.
- Profile with `cargo flamegraph`, `perf` or `samply` to find the real hot spot before touching code.
- The usual wins, in order: a better algorithm → fewer allocations (`Vec::with_capacity`, reuse buffers, `&str` over `String`) → fewer clones → better data layout.
- Avoid allocating in hot loops. Watch for hidden allocations: `format!`, `to_string()`, `collect()` into a fresh `Vec`.
- Use `#[inline]` only on small functions called across crates, and only when a benchmark shows it helps.
- Release profile for shipped binaries:

```toml
[profile.release]
lto = "thin"          # "fat" for max speed, slower builds
codegen-units = 1
panic = "abort"       # smaller and faster, if you don't catch panics
strip = true

[profile.dev.package."*"]
opt-level = 2         # optimised deps, fast-compiling own code
```

- Slow builds hurt too: use `cargo check` while coding, try the `mold` linker, and watch `cargo build --timings`.

## 11. Documentation & comments

Doc comments explain **what** and **how to use**. Code comments explain **why**. Nobody needs a comment explaining what the code literally does. Comments is short, do not explain everything.

- Every public item gets a `///` doc comment. Its first line is a one-sentence summary.
- Use the standard sections where they apply: `# Examples`, `# Errors`, `# Panics`, `# Safety`.
- Examples in docs are real, compiling code (doc tests). Use `?` in them, not `unwrap()`.
- Each crate has a `//!` header in `lib.rs`: what it does, a minimal example, and its feature flags.
- Check docs with `cargo doc --no-deps --open`, and make CI fail on broken intra-doc links (`RUSTDOCFLAGS="-D warnings"`).
- Mark planned work with `// TODO(name or #issue): ...`. A bare `TODO` with no owner or issue rots.
- Keep a `README.md` (what, how to build, how to run) and a `CHANGELOG.md` ("Keep a Changelog" format) at the repo root.

## 12. Git, CI & the pre-commit checklist

CI runs exactly the commands you run locally. If CI is red, nothing merges.

**Workflow**

- Small PRs with one purpose each. Refactors and behaviour changes go in separate commits.
- Commit messages in imperative mood (`Add retry to fetcher`), or Conventional Commits (`feat:`, `fix:`) if you generate changelogs.
- `main` is protected: merge through a PR with green CI only.
- Tag releases with SemVer (`v1.4.0`). In a library, any breaking change to the public API means a major bump; check it with `cargo semver-checks`.

**Minimal CI pipeline**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features   # or: cargo test
cargo test --doc --workspace
cargo deny check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Cache `~/.cargo` and `target/` (for example with `Swatinem/rust-cache`) to keep CI fast.

**Before every commit**

- [ ] `cargo fmt` done
- [ ] `cargo clippy --all-targets` shows no warnings
- [ ] `cargo test` passes, and a new test covers the change
- [ ] No new `unwrap()`, `todo!()`, `dbg!()` or `println!` left in library code
- [ ] Every new `unsafe` block has a `// SAFETY:` comment
- [ ] New public items have doc comments
- [ ] New dependencies are justified and pass `cargo deny`
- [ ] Commit message says *what* changed and *why*
