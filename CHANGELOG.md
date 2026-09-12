# Changelog

All notable changes to `rustweb` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[SemVer](https://semver.org/). See `VERSIONING.md` for the full policy.

## [Unreleased]

### Added
- Criterion native benchmark suite (`tests/benches/render.rs`) over 10–5000-row
  keyed lists for `diff`, `apply`, and `render_to_string`; CI `bench` job saves a
  baseline and uploads the HTML report.
- Real-browser benchmark harness (`crates/dom/src/bench_wasm.rs`) using
  `wasm-bindgen-test` + `performance.now()`; CI `wasm-bench` job runs it in
  headless Chrome via `wasm-pack test --headless --chrome --release`.

### Changed
- `docs/src/performance.md` and `docs/src/release-v1.md` document the benchmark
  harnesses; the real-browser benchmark exit criterion now has a CI-enforced
  measurement path.

## [1.0.0-rc.1] - 2026-09-11

Release candidate: the public API is frozen for 1.0.0. The only remaining
gates before the final `v1.0.0` tag are non-codeable — production burn-in and
real-browser benchmarks (see `docs/src/release-v1.md`).

### Added
- `rustweb-testing::a11y`: offline static accessibility audit returning
  severity-ranked, path-located violations with fix hints (rules:
  `interactive-name`, `form-label`, `image-alt`, `heading-order`,
  `list-structure`, `landmark-main`). Enforced through `cargo test` on every
  CI job.
- `examples/todo-app`: `example_is_a11y_clean` gates the shipped example to
  zero blocking a11y violations.

### Changed
- `docs/src/a11y.md` and `docs/src/release-v1.md` updated: fuzz is validated at
  10M iterations and the a11y static audit is now a code-complete, PR-enforced
  exit criterion; the only remaining v1.0 gates are real-browser benchmarks and
  production burn-in.

## [0.2.0] - 2026-09-11

### Added
- `crates/core`: `ComponentError::Panicked`, `RenderError::Panicked`,
  `run_guarded` and `run_render_guarded` panic boundary (crates/core/src/runtime.rs).
- `crates/core`: `UnnecessaryRenderDetector` profiler and `RenderSample` payload
  for measuring view/patch time and skipping redundant renders
  (crates/core/src/perf.rs).
- `crates/core`: `Scheduler` with per-tick budget to keep `Msg` ping-pong loops
  recoverable (crates/core/src/scheduler.rs).
- `crates/dom`: `Renderer::update` that diffs, applies patches, and
  debug-asserts the committed tree equals the desired tree
  (crates/dom/src/renderer.rs).
- `crates/dom`: `EventDelegator::root_listener_count` — 100 bindings share 1
  root click listener (crates/dom/src/events.rs).
- `crates/macro`: `html!` rejects ARIA typos at compile time with Levenshtein
  suggestions (crates/macro/src/lib.rs).
- `crates/router`: `LazyRoute` factories run exactly once and are unit-tested
  (`lazy_loads_once`).
- `tests/fuzz_diff.rs`: deterministic fuzz harness (seed + iters) asserting
  apply-success rate and SSR match rate over 100k+ iterations.
- `tests/ssr_golden.rs`: golden-format tests freezing the SSR marker contract
  (`data-rwh`, `data-rwe-*`, `data-rwh-checksum`, `<!--rwc:-->`, `<!---->`).
- `examples/todo-app`: full runtime loop demo (`create → view → Msg::Add/Toggle
  → update → diff → apply → SSR → verify_hydration`) driven by `Renderer`.
- `.github/workflows`: `release.yml` (tag-triggered publish in dependency
  order), `audit.yml` (cargo-audit + cargo-deny + weekly schedule), `docs.yml`
  (mdBook + GitHub Pages), `dependabot.yml` (weekly cargo + actions).
- `SECURITY.md`, `deny.toml`, `.github/CODEOWNERS`, issue/PR templates.

### Changed
- `crates/dom/src/diff.rs`: remove paths now use OLD-tree navigation while
  create/move/set paths use NEW-tree navigation. This eliminates cross-parent
  index drift when both structural and content patches coexist.
- `crates/dom/src/diff.rs`: mixed keyed/unkeyed child lists fall through to
  positional diff (documented limitation, no cross-parent structural ambiguity).
- Attribute diffing switched to ordered comparison; `set_attr_at` now
  remove-then-push so SSR attribute order stays deterministic and matches the
  post-patch tree.
- Root `Cargo.toml`: `thiserror = "2"` (`2.0.20`), `rustweb-macro`: `syn = "3"`
  (`3.0.5`). CI actions upgraded to `checkout@v7`,
  `upload-pages-artifact@v5`, `deploy-pages@v5`, `action-gh-release@v3`.
- All Markdown rewritten to professional English; comments stripped repo-wide.
- `docs/book.toml` language `id` → `en`.

### Deprecated
- None.

### Removed
- None.

### Fixed
- GitHub Pages docs workflow: 403 on first upload because Pages was not
  enabled; switched mdBook installer to `taiki-e/install-action@mdbook`
  (peaceiris action pinned to deprecated Node 20).
- `cargo fmt --all -- --check` failures after comment stripping.
- 16 clippy warnings that had been silenced by `-D warnings`; resolved in
  source rather than with `#[allow]` (one exception:
  `#[allow(clippy::derivable_impls)]` on `Default for Cmd` to avoid
  introducing a spurious `C: Default` bound).

### Migration (v0.1 → v0.2)
- No source-level breaking changes for user code. Public types, `html!`
  expansion, and SSR marker format are additive.
- If you pinned `rustweb-cli build --features ssr`, feature names are
  unchanged.
- If you rely on `apply_patches_to_tree` for cross-parent mixed structural
  patches, prefer calling `Renderer::update` (it owns the diff+apply pairing
  and debug-asserts committed==next).
