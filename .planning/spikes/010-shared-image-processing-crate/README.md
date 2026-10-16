---
spike: 010
idea: shared-image-processing
name: shared-image-processing-crate
type: standard
validates: "Given the existing adjustment and derivative code, when it is extracted into a project-local crate, then the controller compiles and passes unchanged behavior contracts through stable re-exports while a future workbench can depend on the same crate."
verdict: VALIDATED
related: [008, 009a, 009b]
tags: [rust, image-processing, crate, architecture, regression]
---

# Spike 010: Shared Image-Processing Crate

## What This Validates

Given the deployed controller's image adjustment, detector, transformation,
and derivative behavior, extracting those capabilities into one project-local
Rust crate must preserve the existing controller API, serialized adjustment
contract, generated derivatives, and regression suite while making the same
implementation available to a future local workbench.

## Research

The existing `image_adjustments` and `derivatives` modules form one capability
and reference one another. Extracting only one would preserve an unnecessary
controller dependency cycle. A crate beneath `controller/` stays inside the
existing Docker build context and avoids introducing a repository-wide Cargo
workspace solely for one reusable boundary.

| Approach | Pros | Cons | Status |
|---|---|---|---|
| Root Cargo workspace with `crates/` | Conventional multi-crate layout | Broad Docker, lockfile, and CI restructuring | Deferred |
| Crate under `controller/` with compatibility re-exports | Small build change; stable controller paths; future tools can use a path dependency | Crate is nested under the controller tree | Chosen |
| Copy modules into the workbench | Fast initially | Guaranteed behavioral drift and duplicated fixes | Rejected |

## How to Run

```bash
cargo test --manifest-path controller/image-processing/Cargo.toml
cargo test --manifest-path controller/Cargo.toml --features production-persistence
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/image-processing/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path controller/Cargo.toml --all-targets --features production-persistence -- -D warnings
docker build -f controller/Dockerfile -t autographs-controller:spike-010 .
```

## What to Expect

The controller keeps its existing `autographs_controller::image_adjustments`
and `autographs_controller::derivatives` paths through re-exports. All existing
image, route, catalog, Oracle, publisher, and static-admin tests pass without
changing their expected behavior.

## Investigation Trail

- Established a clean pre-refactor baseline: 190 tests passed, two explicit
  live tests remained ignored, and no test failed.
- Confirmed `image_adjustments.rs` and `derivatives.rs` are mutually coupled and
  should move as one capability.
- Confirmed other proposed crate boundaries lack a current second consumer;
  they remain modules rather than speculative crates.
- Moved the two source modules byte-for-byte into
  `controller/image-processing` and retained the established controller module
  paths with public compatibility re-exports.
- The first production Docker build exposed that `cargo-chef cook` resolved the
  path dependency before the nested crate existed in its builder stage. Moving
  that copy ahead of `cook` fixed the build; the repeated build completed and
  produced `autographs-controller:spike-010`.
- Excluded the nested crate's generated `target` directory from Git and Docker
  contexts. The Docker build context dropped from roughly 463 MB to 7 KB.
- Passed the shared crate's strict Clippy check, the exact CI production-feature
  Clippy command, formatting, runtime-artifact validation, and a complete
  production-feature controller test run: 251 passed, two intentionally ignored
  live tests, and no failures. The shared crate's own unit test also passed.

## Results

**VALIDATED.** The controller now consumes the same project-local crate that the
future fixture-driven workbench can use by path dependency. The existing public
controller module paths, adjustment JSON/cache contract, detector behavior,
derivative generation, routes, persistence adapters, and tests remain intact.

The extracted crate shipped in controller v0.4.2. Production deployment,
authenticated editor use, and an incremental publish all completed successfully,
so the deployed-baseline gate for Spike 011 is satisfied.
