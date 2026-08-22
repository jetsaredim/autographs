---
quick_id: 260822-osh
status: complete
created: 2026-08-22
---

# Replace Prototype Quality Parsing With Native Tooling

## Objective

Replace the proposed regex-based Python quality checker with maintained,
language-aware quality gates while preserving the operational inventory tools
that inspect filesystem, Podman, systemd, and deployment state rather than Rust
syntax.

## Tasks

### 1. Establish Rust-native and structural rules

- Configure selected Rust/Clippy lints in `controller/Cargo.toml` for production
  placeholder macros.
- Add an ast-grep project configuration, declarative Rust rules, and fixture
  tests for distributed runtime environment reads.
- Baseline current non-config environment reads with narrow rule-specific
  suppressions that identify C2 typed-configuration debt; reject unused or
  suppress-all directives.
- Do not reproduce SQL parsing or secret-template name matching with regexes.

### 2. Add measured coverage without duplicate test execution

- Install pinned `cargo-llvm-cov` and ast-grep releases in PR CI.
- Replace the plain default-feature `cargo test` step with `cargo llvm-cov`,
  using the measured current line-coverage result as a regression floor.
- Run ast-grep rule fixtures and the repository scan in CI.
- Retain production-persistence compilation and Clippy coverage.

### 3. Promote the enforceable standard and verify

- Promote the spike style guide into maintained documentation, distinguishing
  compiler, Clippy, coverage, AST, contract-test, secret-scan, and operational
  inventory ownership.
- Mark the prototype quality checker as research-only and update the cleanup
  plan so C1 points at the maintained gates.
- Run rule fixtures, the repository scan, formatting, tests/coverage,
  production checks, Clippy, workflow checks, and repository hygiene.
- Commit atomically, open a ready PR, record counter-review findings on the PR,
  address every finding, and merge only after all checks are green.

## Success Criteria

- CI no longer depends on a Python regex parser to understand Rust.
- Every structural rule has valid and invalid fixtures.
- Existing direct env-read debt is explicit and cannot grow silently.
- Coverage is measured in the same invocation that runs default-feature tests.
- The permanent style guide explains which maintained tool owns each rule.
