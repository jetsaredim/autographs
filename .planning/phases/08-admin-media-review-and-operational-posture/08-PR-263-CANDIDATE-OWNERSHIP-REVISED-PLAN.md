---
phase: 08-admin-media-review-and-operational-posture
pr: 263
status: review_required
trigger_review: 08-REVIEW.md
trigger_review_commit: daef9efba4b96d02281b8ce1c4c0d01ea32367db
finding: CR-02
finding_family: live_smoke_cleanup_discovery
lineage: incomplete_fix
review_required: true
review_evidence: null
review_comment: null
---

# PR 263 Candidate Ownership Revised Plan

## Objective

Close the remaining CR-02 ownership gap without reopening the closed recovery
surface or evidence-privacy contracts. A failed Oracle inventory attempt must
be atomic with respect to the cleanup guard's locally known candidates: the
guard retains its complete pre-call set until a successfully discovered union
replaces it.

## Locked invariants

1. Locally known candidates are durable cleanup authority until each object is
   verified absent; fallible discovery never consumes or partially replaces
   them.
2. A successful inventory atomically installs the deduplicated union of the
   pre-call candidates, current image keys, and cleanup-event targets.
3. Object deletion and absence verification complete for the installed union
   before destructive Oracle row cleanup begins.
4. Any discovery, deletion, or verification failure retains Oracle discovery
   rows and the guard's full candidate set for retry or recovery.
5. Cleanup evidence remains fingerprint-only; CR-01 and WR-01 stay closed.

## Task 1: Make candidate refresh transactional

**Files:**

- `controller/tests/live_persistence_smoke.rs`
- `controller/tests/media_cleanup.rs`
- `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW-FIX.md`

**Implementation:**

- Remove destructive transfer of `self.object_keys` before discovery. Snapshot
  or immutably borrow the current candidates, build the discovered union in a
  temporary `BTreeSet`, and assign the resulting vector only after every query,
  row read, and decode succeeds.
- Extract the production candidate-state transition behind a small helper that
  accepts the current durable set and a fallible discovery producer. The live
  guard and its pure all-feature tests must call the same helper; do not validate
  retry semantics only through a parallel fixture with stronger ownership.
- Preserve candidates across the post-replacement refresh, explicit cleanup,
  normal guard cleanup, and fallback `Drop`. A retry may add candidates but may
  never forget a previously known identity.
- Keep the existing ordering: inventory, delete and verify every candidate,
  then delete/commit/verify Oracle rows. Do not alter opaque evidence helpers or
  provider error redaction.

**Executable failure matrix:**

| Scenario | Required assertion |
| --- | --- |
| First discovery fails before returning any rows | Existing A remains byte-for-byte in guard state; no delete or SQL cleanup occurs. |
| First discovery partially reads rows then fails | No partial set is installed; the complete pre-call set remains authoritative. |
| First discovery fails, fallback retry succeeds with B/event targets | Installed union contains A, B, and targets exactly once; all are deleted and verified before the first row deletion. |
| Repeated discovery failures | Local candidates and Oracle rows remain; recovery evidence contains only safe step/item data. |
| Successful retry after duplicate/already-absent candidates | Deduplicated union is idempotent and row cleanup occurs only after every absence proof. |
| Delete or verification fails after successful inventory | The full installed union and Oracle discovery rows remain available for retry. |

**Tests:**

- Add pure tests beside the production helper in
  `live_persistence_smoke.rs`; they run under `--all-features` without Oracle or
  OCI credentials and exercise failure-then-success and persistent failure on
  the actual ownership transition.
- Tighten the deterministic cleanup authority fixture so it models transactional
  candidate installation and asserts the production ordering/event trace.
- Retain source-contract checks only as secondary evidence; they must not be the
  sole proof that ownership survives a failed query.
- Re-run privacy tests with distinctive raw markers to prove the ownership fix
  does not regress WR-01.

## Acceptance criteria

- No `mem::take` or equivalent destructive move can empty durable candidates
  before a fallible discovery completes.
- A first-failure/second-success test fails against `d61a38e` semantics and
  passes using the same state-transition helper invoked by the live guard.
- Persistent failure proves both local candidates and Oracle rows remain
  recoverable.
- CR-01 and WR-01 remain closed, all targeted/full checks pass, and the next
  lineage-preserving deep source review reports zero actionable findings.

## Verification

```bash
cargo fmt --manifest-path controller/Cargo.toml --all -- --check
cargo test --manifest-path controller/Cargo.toml --test media_cleanup
cargo test --manifest-path controller/Cargo.toml --test logging_contract
cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke
cargo test --manifest-path controller/Cargo.toml --all-targets --all-features
cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings
node controller/tests/static_admin_behavior.mjs
git diff --check
```

The credentialed live smoke cases remain opt-in; pure ownership-transition
tests in the same target must run without credentials.

## Plan approval gate

An independent plan reviewer must verify the helper cannot consume durable
candidates on any error, the tests execute the production transition, every
explicit/normal/Drop consumer preserves the invariant, destructive SQL remains
after all absence proofs, and privacy/recovery contracts stay closed. Approval
requires zero Critical, Warning, and Info findings. Record the review artifact
and PR comment in the convergence ledger before changing status to
`ready_to_resume`.
