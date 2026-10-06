---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-06T20:22:47Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 7
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-06T20:22:47Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 7 — approved Media Failure Recovery Authority Addendum

**Summary:**

- Findings in scope: 2
- Fixed: 2
- Skipped: 0
- Implementation commit: `6cb2f5d`
- PR fix comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-6024771455

## Fixed Issues

### CR-01: Rollback failure deletes the object still referenced by catalog metadata

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/catalog.rs`, `controller/src/oracle_catalog.rs`, `controller/src/routes.rs`, `controller/tests/admin_workflow.rs`, `controller/tests/live_persistence_smoke.rs`, `controller/tests/logging_contract.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/publisher.rs`
**Commit:** `6cb2f5d`
**Applied fix:** Replacement and restoration are now typed expected-object-key compare-and-set transitions in memory and Oracle. Initial candidate cleanup is reload-gated, restoration reload verifies the exact original key, opaque revision, and adjustment before deletion, and repository failure, verification mismatch, CAS loss, or deletion failure preserves every possibly referenced object. Exact redacted 409, 404, recovery-required 500, and fully-recovered replacement-failed 500 bodies are distinct. Recovery-required responses carry a UUID correlated to a structured private event containing transition/outcome and SHA-256 key fingerprints without exposing raw storage identity.

### CR-02: Unscoped 409 handling can tear down a newer review and cannot recover the current one

**Status:** Fixed — requires human verification
**Files modified:** `controller/static-admin/admin.js`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`
**Commit:** `6cb2f5d`
**Applied fix:** Every conflict path now proves its operation-specific session, item/image/media, request/draft, assist, mutation, controller, blob, and node authority before any UI effect. Source-guide acquisition uses authenticated fetch-to-blob with abort/revocation ownership. A current conflict invalidates once, clears stale item/review actions, and refreshes through a dedicated recovery generation before presenting reopen guidance; 404, transient failure/retry, auth failure, navigation, and new-review races have explicit generation-safe state transitions. Immediate reopen uses the refreshed media revision, while stale conflicts and late recovery results are inert.

## Invariant and Consumer Audit

| Invariant | Producers and mutation boundaries | Consumers and failure paths | Evidence |
|---|---|---|---|
| Replacement is expected-key CAS | In-memory item lock and Oracle `object_key = :15` predicate | Initial replacement, rollback restoration, live-smoke/publisher callers | Memory conflict test, Oracle SQL contract test, all-feature compile/test |
| Destructive cleanup follows proof | Reloaded catalog snapshot and exact original key/revision/adjustment comparison | Initial candidate cleanup, rollback replacement cleanup, concurrent winner | Repository failure, concurrent winner, verification mismatch, delete-failure, and successful restoration tests |
| Recovery errors are typed and private | Stable response builders and UUID-correlated structured event | 409 conflict, 404 missing image, recovery 500, recovered failure 500 | Exact-body route tests and logging privacy/correlation contract |
| Conflict effects are origin-scoped | Draft controller/revisions, source controller/blob, assist generation, mutation token/submission tuple | Review open/source, draft, assist, Save, Reset, source/output callbacks | Executable DOM stale-conflict, source-conflict, refresh/retry, missing-item, and immediate-reopen cases |
| Recovery never reuses stale item authority | Conflict recovery generation plus navigation context | Success reconcile, 404 collection state, retry-only transient state, auth/logout, late result | Executable DOM recovery state table and central `reconcileAdminItemResponse` assignment audit |

## Verification

Verification ran in `/tmp/autographs-pr263-review`, the existing PR worktree on `gsd/phase-08-admin-media-review-and-operational-posture`.

- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- Focused `admin_workflow`, `media_cleanup`, `static_admin`, and `logging_contract` suites — passed; 78 tests.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features --quiet` — passed; 229 non-ignored tests, 2 credential-gated live tests ignored.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-06T20:22:47Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 7_
