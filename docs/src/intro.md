# Introduction

`rustweb` is a Virtual DOM-based Rust frontend library targeting WebAssembly,
built for production use. Closest reference: Yew.

Principles:

1. **Correctness first**: diff/patch and SSR/hydration share a single traversal definition,
   so a mismatch is a detectable bug, not fate.
2. **No panics across components**: every lifecycle stage returns `Result` and runs behind `catch_unwind`.
3. **Measured performance**: event delegation, LIS-based keyed diffing, CI-enforced budgets.
4. **Actionable developer experience**: macro errors point at the problem with a hint, props are strongly typed.
