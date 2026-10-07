---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-07T00:00:00Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 8
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-07T00:00:00Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 8 — approved Ambiguous Delete and Exclusive Recovery Addendum

**Summary:**

- Findings in scope: 2
- Fixed: 2
- Skipped: 0
- Implementation commit: `2378cb5`
- PR fix comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-6037912329

## Fixed Issues

### CR-01: Ambiguous old-object deletion can roll metadata back to a missing object

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/routes.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/live_persistence_smoke.rs`, `docs/static-runtime-runbook.md`
**Commit:** `2378cb5`
**Applied fix:** Once replacement metadata commits and an old-object delete is attempted, the replacement remains authoritative for every returned delete error. Persisted cleanup warnings return normal success; warning-persistence failure returns UUID-correlated, redacted `imageRecoveryRequired` evidence with `old_delete_outcome_ambiguous`, without rollback or candidate deletion. Deterministic retain-then-error and delete-then-error adapters prove the catalog always references a readable replacement. The double-opt-in live smoke exercises real Oracle/OCI revision A→B replacement and stale source/draft/assist/Save/Reset conflicts without production fault injection, then verifies cleanup.

### CR-02: Conflict recovery exposes an actionable blank editor and accepts same-view stale results

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `2378cb5`
**Applied fix:** Current conflicts now enter a dedicated inert editor surface rather than rendering a blank editor. Recovery owns an abort controller and a tuple of recovery generation, captured editor-authority generation, item, navigation context, request owner, status, and surface state. All normal reconciliation/render, navigation, logout, and new-review paths advance editor authority and invalidate the owner; Save and dirty handlers reject invocation while inert. Deferred-response tests prove input/Save blocking and late direct-render, navigation, and new-review results are inert, while retry succeeds only through the full predicate.

## Invariant and Consumer Audit

| Invariant | Producers and mutation boundaries | Consumers and failure paths | Evidence |
|---|---|---|---|
| Ambiguous deletion never reverses authority | Replacement metadata CAS commits before old delete; delete errors have unknown provider outcome | Warning success/failure, cleanup retry, HTTP recovery evidence | Retain-then-Err and delete-then-Err tests with readable replacement and exact file counts |
| Recovery assignment is exclusive | Recovery generation, editor generation, item, navigation, owner, status, inert surface | Refresh success/retry, render/reconcile, Save/input, navigation/logout/new review | Deferred DOM cases and static source contract |
| Live validation is private and disposable | Two explicit opt-ins, draft-only A/B fixture, local checksums | Stale source/draft/assist/Save/Reset and current Reset | Feature compile plus exact runbook cleanup/abort contract |

## Verification

Verification ran in `/tmp/autographs-pr263-review`, the existing PR worktree on `gsd/phase-08-admin-media-review-and-operational-posture`.

- `node --test controller/tests/static_admin_behavior.mjs` — passed.
- Focused `admin_workflow`, `media_cleanup`, and `static_admin` suites — passed; 75 tests.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run` — passed.
- Focused live cleanup classification test — passed; only provider-confirmed 404 is accepted, followed by normal absence verification.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence` — passed; full production-feature suite.
- Exact CI `cargo clippy --manifest-path controller/Cargo.toml --all-targets --features production-persistence -- -D warnings` — passed.
- CI coverage command was not runnable locally because `cargo-llvm-cov` is not installed; Cargo returned `no such command: llvm-cov`.
- `cargo fmt --all -- --check` and `git diff --check` — passed.

---

_Fixed: 2026-10-07T00:00:00Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 8_
