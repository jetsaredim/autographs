---
status: in-progress
issue: 170
created: 2026-07-27
task: issue-170-oracle-basiclite
---

# Quick Task: Evaluate Oracle Instant Client Basic Lite for controller image size reduction

## Goal

Replace the controller runtime and smoke image Oracle Instant Client package with Basic Lite if local checks and live smoke validation show it is compatible with the app's Oracle, Object Storage, publishing, and non-English metadata paths.

## Plan

1. Swap `oracle-instantclient-basic` to `oracle-instantclient-basiclite` in the controller runtime Dockerfile and live smoke Dockerfiles, preserving existing package pinning where present.
2. Update repository contract tests and operator documentation so the intended Basic Lite runtime dependency, image-size validation, live smoke requirements, and app-relevant limitations are explicit.
3. Build the controller and smoke images, compare image size against the issue's recorded baseline, and run normal local controller checks.
4. Run or document the existing live persistence/static publish smoke gate against production-like Oracle/Object Storage credentials, including confirmation that non-English catalog metadata survives the real controller persistence path.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- Build `controller/Dockerfile`, `controller/Dockerfile.smoke`, and `controller/Dockerfile.static-smoke`.
- Run the live smoke commands from `docs/static-runtime-runbook.md` when credentials/runtime access are available.
