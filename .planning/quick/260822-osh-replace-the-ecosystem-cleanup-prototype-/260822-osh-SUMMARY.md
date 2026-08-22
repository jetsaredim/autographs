---
quick_id: 260822-osh
status: complete
completed: 2026-08-22
---

# Native Controller Quality Gates

## Outcome

Replaced the spike's proposed regex-based Python Rust checker with maintained,
language-aware gates. Operational inventory collectors remain Python tools
because they inspect repository declarations, filesystem metadata, Podman, and
systemd rather than attempting to parse Rust.

## Changes

- Added focused Clippy denials for `dbg!`, `todo!`, and `unimplemented!`.
- Added a fixture-tested ast-grep rule that prevents new distributed process
  environment reads and records existing typed-configuration debt with narrow,
  rule-specific suppressions.
- Changed CI to run production-feature tests once through `cargo-llvm-cov`,
  enforcing the measured `62%` line-coverage floor, and expanded Clippy to the
  production feature set.
- Published the maintained controller engineering standard and marked the
  spike's Python checker as historical research evidence.

## Verification

- ast-grep rule tests: 1 rule and 22 fixtures passed.
- ast-grep repository scan passed with unused and suppress-all directives
  rejected.
- Production-feature Rust suite passed at `62.30%` line coverage.
- `cargo fmt --check` and production-feature `cargo clippy -D warnings` passed.
- Repository automation suite passed all 48 tests.
- Repository hygiene, Terraform version alignment, YAML parsing, and
  `git diff --check` passed.

## Decisions

- Keep domain behavior such as Oracle bind correctness in focused Rust tests;
  do not replace the prototype with another source-string parser.
- Treat coverage as a regression signal, not proof of live Oracle or OCI
  behavior. Credentialed production paths retain their VM smoke gates.
- Defer the actual typed configuration consolidation to Cleanup C2; C1 freezes
  the current direct-read baseline so it cannot silently grow.
