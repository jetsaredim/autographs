# Controller Engineering Standards

This document defines the maintained quality contract for the Rust controller
and its production configuration. Rules are assigned to the narrowest tool that
understands them; repository policy is not implemented by parsing Rust with
regular expressions.

## Tool Ownership

| Concern | Owner | Enforcement |
|---------|-------|-------------|
| Rust formatting | `rustfmt` | `cargo fmt --check` blocks CI |
| Rust correctness and ordinary linting | Rust compiler and Clippy | production features and all targets; warnings denied |
| Production placeholder macros | Clippy manifest lints | `dbg!`, `todo!`, and `unimplemented!` denied |
| Test execution and line coverage | `cargo-llvm-cov` | production-feature tests run once with a measured coverage floor |
| Rust architecture patterns | ast-grep | declarative, fixture-tested AST rules block CI |
| Runtime and persistence behavior | Rust tests | characterize typed configuration, Oracle binds, redaction, and repository behavior |
| Committed secret values | Gitleaks | dedicated CI secret scan |
| Deployment contracts | Ansible syntax, lint, and contract tests | validate rendered runtime behavior and secret delivery |
| VM ownership and cruft | redacted inventory collector | operator evidence; never parses Rust or collects secret values |

## Configuration Ownership

- `ControllerConfig` is the production process-environment owner. Oracle, OCI,
  authentication, publishing, and path settings should move into typed
  sub-configurations during Cleanup C2.
- Routes and persistence/media constructors receive typed configuration; new
  direct `std::env` or `env` reads outside `controller/src/config.rs` are
  blocked by `no-distributed-env-read`.
- Existing rule-specific suppressions are an explicit C2 debt baseline. CI
  rejects stale suppressions, suppress-all comments, and unsuppressed new
  reads.
- One-shot live-smoke gates remain owned by their test or binary. They are not
  production controller configuration.
- Every durable variable must have an owner, classification, default behavior,
  consumers, and retirement condition in the configuration contract.

## Secrets and Runtime Files

- Passwords, bearer tokens, private keys, password hashes, and wallet contents
  must not persist in controller env files after the OCI Vault cutover.
- Gitleaks detects committed secret material; it does not prove that a runtime
  template has the right secret-delivery architecture.
- Cleanup C2 must define one rendered non-secret runtime configuration and
  Ansible contract tests. C3/C4 must prove instance-principal Vault retrieval,
  tmpfs wallet materialization, restart/denial behavior, and removal of durable
  secret files.
- Secret OCIDs are configuration, not secret contents. Files containing
  sensitive coordinates remain mode `0600`.
- Runtime OCI user API keys are forbidden. Deployment automation keeps its
  separate credential path until a workload-identity migration is designed.
- Inventory tools may emit names, paths, owners, permissions, hashes, and
  resource metadata, but never values.

## Rust Structure

- Keep formatting, production-feature tests, measured coverage, and Clippy as
  mandatory gates.
- Enable low-noise Clippy lints individually. Do not enable the complete
  pedantic or restriction groups.
- Split large modules along existing domain responsibilities when touched. A
  line threshold is a review signal, not an automatic rewrite mandate.
- Introduce typed errors where callers classify, redact, retry, or map failures.
  Do not create a single project-wide error enum merely to remove
  `Result<T, String>`.
- Add abstractions only when they remove demonstrated duplication or encode a
  contract that tests or static analysis can verify.

## Oracle SQL

- Inline a complete one-off statement beside its query or execute call.
- Use a module-level `*_SQL` constant when the statement is reused, composed
  from a shared projection, or directly contract tested.
- Positional binds are appropriate when every value appears once and arguments
  follow placeholder occurrence order. Named binds are appropriate when values
  repeat or statement composition makes occurrence order unclear.
- Do not mix named and positional binds within one statement.
- Numeric placeholders use `:1..:N` in occurrence order. Repeated, skipped, or
  reordered placeholders require a focused repository regression test or a
  named-bind rewrite.
- This SQL rule is domain behavior, not generic Rust syntax. C1 intentionally
  does not replace the historical regex prototype with another source-string
  parser. A future compile-time SQL wrapper is justified only if touched Oracle
  work shows that focused tests are insufficient.

## Coverage Contract

The production-feature suite measured `62.30%` line coverage on 2026-08-22.
CI uses a `62%` floor to catch regressions while leaving rounding tolerance.
The lower production-feature percentage is expected because live Oracle and OCI
operations compile into the report but require separately gated VM smokes.

Coverage percentage is a regression signal, not proof of behavior. Changes to
authentication, privacy, Oracle persistence, publication, and cleanup still
require focused assertions and live smokes where applicable.

## Local Quality Commands

Use the tool versions pinned in `.github/workflows/ci.yml`:

```bash
ast-grep test --skip-snapshot-tests
ast-grep scan --error=unused-suppression --error=no-suppress-all controller/src
cargo fmt --manifest-path controller/Cargo.toml --check
cargo llvm-cov --manifest-path controller/Cargo.toml \
  --features production-persistence --summary-only --fail-under-lines 62
cargo clippy --manifest-path controller/Cargo.toml \
  --all-targets --features production-persistence -- -D warnings
```

The ecosystem inventory scripts remain appropriate for operational state: they
inspect repository declarations, filesystem metadata, Podman, and systemd.
They are not permanent Rust linting tools.
