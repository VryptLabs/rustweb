name: Feature request
description: Propose an addition or API change
labels: [enhancement]
body:
  - type: textarea
    id: problem
    attributes:
      label: Problem
      description: What production use case is blocked today?
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposal
      description: API sketch, semantics, and affected crates
    validations:
      required: true
  - type: dropdown
    id: compat
    attributes:
      label: Compatibility impact
      options:
        - Additive (minor)
        - Breaking (requires MIGRATION.md entry)
        - Unsure
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives considered
