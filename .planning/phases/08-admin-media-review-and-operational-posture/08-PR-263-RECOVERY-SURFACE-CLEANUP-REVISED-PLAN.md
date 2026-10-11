---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: proposed
trigger_review: 08-REVIEW.md
findings:
  - CR-01
  - CR-02
  - WR-01
review_required: true
---

# PR 263 Recovery Surface and Cleanup Authority Revised Plan

## Objective

Close the resumed deep-review findings as one recovery-authority contract
change: keep exclusive conflict recovery usable, make live-smoke cleanup
discover every private object before deleting its metadata, and extend the
fingerprint-only evidence boundary through cleanup tooling and operator docs.

This plan does not reopen closed media-revision, gesture, publisher,
comparison-map, or public-artifact work. It must be implemented in one coder
pass only after independent plan approval.

## Locked invariants

1. Recovery status and its bound retry remain outside every hidden or inert
   editor ancestor; ordinary editor actions remain inactive until recovery
   settles or is invalidated.
2. Object identity discovery completes before any Oracle row that can reveal a
   current or cleanup-candidate key is deleted.
3. Cleanup output and errors never expose raw object keys, raw media checksums,
   credentials, namespace/bucket values, or unsanitized provider responses.
4. Existing generation, request-owner, editor-authority, media-revision, CAS,
   and ambiguous-delete behavior remains unchanged unless this plan explicitly
   strengthens it.

## Task 1: Separate recovery presentation from editor ownership

**Files:**

- `controller/static-admin/index.html`
- `controller/static-admin/admin.css`
- `controller/static-admin/admin.js`
- `controller/tests/static_admin.rs`
- `controller/tests/static_admin_behavior.mjs`

**Implementation:**

- Add one dedicated recovery status/action container in `#add-item-view`
  outside `#item-form`. Give it status semantics and a stable selector; keep it
  hidden when no recovery owns the editor.
- Centralize recovery presentation so pending, retryable, missing, auth, and
  successful-refresh transitions cannot write their only actionable copy into
  `#image-message`. `enterConflictRecoverySurface`,
  `renderConflictRecoveryRetry`, `recoverConflictedItem`,
  `leaveConflictRecoverySurface`, and invalidation paths must retire or update
  the same container.
- Preserve the existing complete acceptance predicate: recovery generation,
  editor-authority generation, item ID, navigation revision, request owner,
  status, view, and exclusive editor state. Moving presentation must not make
  ordinary form, image, publish, or programmatic handlers actionable.
- Make the recovery container visible while `#item-form` is hidden/inert; make
  only the current retry actionable in retryable state. Authority change,
  navigation, logout, direct render/reconcile, or newer review must hide the
  old container and make late callbacks inert.

**Tests:**

- Extend static markup contracts to prove the recovery container is outside
  the item form and carries the required status/action semantics.
- Upgrade the executable DOM fixture to model the relevant parent/child tree,
  ancestor `hidden`, and inert descendant user-event suppression. A user-event
  helper must refuse clicks through an inert ancestor; explicit programmatic
  dispatch remains separately named.
- Reproduce pending and retryable recovery with the form hidden/inert and
  assert the message remains perceivable and retry is user-dispatchable.
- Defer retry completion across navigation, direct render, logout, and a new
  review; assert the retired surface and result are inert. Preserve current
  200/404/401/403 and successful reopen cases.

**Acceptance criteria:**

- CR-01 fails against the old topology and passes with the dedicated surface.
- No ordinary editor consumer becomes actionable during recovery.
- Existing recovery generation and authority tests remain green.

## Task 2: Make cleanup discovery precede destructive row cleanup

**Files:**

- `controller/tests/live_persistence_smoke.rs`
- `controller/tests/media_cleanup.rs`
- `docs/static-runtime-runbook.md`

**Implementation:**

- Give `LivePersistenceSmokeCleanup` an inventory step keyed by the disposable
  item ID. Before response assertions and again immediately before cleanup,
  query Oracle for current `autograph_images.object_key` values and pending
  `autograph_cleanup_events.target_object_key` values. Merge those with locally
  known A/B candidates and deduplicate them in memory.
- Preserve response bodies/statuses for assertions, but register discovered
  identities before panicking or returning on an unexpected response.
- Use the same discovery-first ordering in explicit cleanup and fallback
  cleanup: inventory keys, delete and verify every object (404 accepted under
  the existing exact classification), then delete Oracle rows, commit, and
  verify item/image/edit/cleanup-event counts are zero.
- If discovery or object deletion cannot be proven, retain Oracle discovery
  rows and emit redacted recovery-required evidence. Never delete rows merely
  to make the smoke appear clean.
- Keep cleanup idempotent for duplicate keys, already absent objects, repeated
  invocations, and partial prior cleanup.

**Tests:**

- Add a deterministic local fixture for “B metadata committed, route returns
  recovery-required before the smoke registers B.” Assert unwind inventory
  discovers B, deletes/verifies A and B as applicable, and only then removes
  all rows.
- Cover current-image keys, cleanup-event targets, local candidates,
  duplicates, provider 404, discovery failure, delete failure, verification
  failure, and retry after partial cleanup.
- Keep credential-gated Oracle/OCI execution opt-in; compile it in CI and do
  not add production fault injection.

**Acceptance criteria:**

- CR-02 is closed without requiring a raw key from an HTTP response or log.
- Every path inventories before row deletion and preserves discovery metadata
  on unverified cleanup.
- Normal and fallback cleanup prove object absence and zero rows.

## Task 3: Enforce fingerprint-only cleanup evidence

**Files:**

- `controller/tests/live_persistence_smoke.rs`
- `controller/tests/logging_contract.rs`
- `docs/static-runtime-runbook.md`

**Implementation:**

- Introduce one domain-separated cleanup fingerprint helper compatible with
  the application recovery-evidence contract. Raw keys remain input/data only.
- Replace key-bearing progress, timeout, panic, absence-verification, and
  forwarded-provider messages with safe operation/status text plus item ID,
  count, recovery ID where available, and fingerprint.
- Sanitize provider failures at the cleanup boundary; do not assume provider
  text is safe to forward.
- Update interrupted-cleanup instructions to prefer item-ID discovery. If an
  operator must supply candidate keys privately, state that tooling consumes
  but never echoes them and show only fingerprint-based expected evidence.

**Tests:**

- Capture cleanup output for success, already-absent, timeout, provider error,
  verification error, and recovery-required paths.
- Seed distinctive raw keys, checksums, namespace, and bucket markers; assert
  none appear while stable domain-separated fingerprints and safe correlation
  fields do.
- Extend documentation/static contracts so examples and expected output do not
  regress to raw private identifiers.

**Acceptance criteria:**

- WR-01 is closed across normal, timeout, panic, provider, and runbook paths.
- Tests fail if any seeded private identifier reaches captured output.

## Verification

Run from the repository root:

```bash
node --check controller/static-admin/admin.js
node --check controller/tests/static_admin_behavior.mjs
node controller/tests/static_admin_behavior.mjs
cargo fmt --manifest-path controller/Cargo.toml --all -- --check
cargo test --manifest-path controller/Cargo.toml --test static_admin
cargo test --manifest-path controller/Cargo.toml --test media_cleanup
cargo test --manifest-path controller/Cargo.toml --test logging_contract
cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run
cargo test --manifest-path controller/Cargo.toml --all-targets --all-features
cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings
git diff --check
```

The credential-gated live smoke is not required locally without credentials.
After implementation is pushed, all PR checks must pass. A subsequent deep
source review must preserve lineage and report zero Critical, Warning, and Info
findings before convergence closes.

## Plan approval gate

An independent plan reviewer must verify:

- every direct producer, consumer, mutation boundary, reporting/action path,
  and relevant test for all three invariants is covered;
- discovery and deletion ordering is implementable in both explicit and Drop
  cleanup without losing recovery metadata;
- error redaction covers provider-controlled text as well as keys formatted by
  the smoke itself;
- the plan does not weaken closed Phase 8 authority/privacy contracts.

Approval requires zero Critical, Warning, and Info findings. Record the review
artifact and PR comment in `08-PR-263-CONVERGENCE.md` before changing its status
to `ready_to_resume`.
