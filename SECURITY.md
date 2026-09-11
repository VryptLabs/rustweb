# Security policy

## Supported versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | Yes                |
| < 0.1   | No                 |

## Reporting a vulnerability

Open a private security advisory on GitHub or contact the maintainers listed in
`.github/CODEOWNERS`. Include the affected crate versions, a minimal
reproduction, and the impact assessment.

We aim to acknowledge reports within 3 business days and to ship a patched
release within 30 days for confirmed high-severity issues. Low-severity issues
are scheduled into the next minor release.

## Scope

In scope: panics reachable from untrusted input across a component boundary,
hydration mismatches caused by the framework (not by app-level `view`
nondeterminism), dependency advisories affecting the default build.
