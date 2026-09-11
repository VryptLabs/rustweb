# Testing

- Unit: `TestHarness<C>::new(props).create/view/update` with no browser.
- Snapshots: `assert_snapshot(name, &vnode, expected_sexpr)`; bless with
  `UPDATE_SNAPSHOTS=1`.
- Properties: `tests/patch_properties.rs` asserts `apply(diff(old,new)) == new`.
- Hydration: `tests/hydration_roundtrip.rs`.
- Macros: `tests/macro_smoke.rs` (including typed props).
- Browser: `tests/headless.rs` (`#[ignore]`, runs with `HEADLESS=1` plus a WebDriver).
- Benchmark regression: `tests/bench_regression.rs` (time budgets, fails on regression).
