name: Bug report
description: Report a reproducible defect
labels: [bug]
body:
  - type: input
    id: version
    attributes:
      label: Version
      description: Crate versions (e.g. rustweb-core 0.1.0) or commit hash
    validations:
      required: true
  - type: dropdown
    id: target
    attributes:
      label: Target
      options:
        - host (native tests / SSR)
        - wasm32-unknown-unknown (browser)
    validations:
      required: true
  - type: textarea
    id: repro
    attributes:
      label: Reproduction
      description: Minimal code plus commands to reproduce
      placeholder: |
        ```rust
        // minimal view/diff/router case
        ```
        ```sh
        cargo test -p rustweb-tests --test …
        ```
    validations:
      required: true
  - type: textarea
    id: expected
    attributes:
      label: Expected vs actual
    validations:
      required: true
  - type: textarea
    id: logs
    attributes:
      label: Logs
      description: Panic payload, HydrationError, or console output
