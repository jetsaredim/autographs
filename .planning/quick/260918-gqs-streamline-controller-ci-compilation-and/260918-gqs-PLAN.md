---
quick_id: 260918-gqs
status: complete
description: Streamline controller CI compilation and Docker build caching
must_haves:
  truths:
    - Controller coverage, Clippy, and release-image validation remain mandatory for relevant changes.
    - Rust dependencies are reusable across source-only Docker builds.
    - Non-image changes skip the runtime image build without blocking required checks.
  artifacts:
    - controller/Dockerfile
    - .github/workflows/ci.yml
    - scripts/release.py
  key_links:
    - CI impact classification reuses the release controller-image boundary.
    - The Docker bake GHA cache exports the cargo-chef dependency layer.
---

# Quick Task 260918-gqs Plan

## Task 1: Remove duplicate controller compilation

- **Files:** `scripts/validate-runtime.sh`, `controller/tests/runtime_kernel_persistence.rs`
- **Action:** Keep the shell preflight for required runtime artifacts but rely on
  the full coverage suite to execute the Rust integration test once.
- **Verify:** Run the runtime contract test and production-feature suite.
- **Done:** The contract remains enforced without a separate pre-coverage Cargo build.

## Task 2: Add reusable Docker dependency layers

- **Files:** `controller/Dockerfile`, `controller/tests/caddy_static_routes.rs`
- **Action:** Add pinned cargo-chef planner/cook stages while preserving explicit
  compile-time asset copies and the Oracle Linux runtime image.
- **Verify:** Run Dockerfile contracts, Hadolint, and a full BuildKit image build.
- **Done:** Source-only changes reuse the dependency layer and still produce the
  same controller runtime image contract.

## Task 3: Skip irrelevant image builds

- **Files:** `scripts/release.py`, `scripts/test_release.py`, `.github/workflows/ci.yml`
- **Action:** Add tested arbitrary-ref impact classification and condition the
  image job at job level, defaulting workflow dispatches to a full build.
- **Verify:** Run automation tests and actionlint, then exercise controller,
  infrastructure-only, and workflow-change classifications.
- **Done:** Relevant image inputs build; unrelated changes produce a successful
  skipped image check.
