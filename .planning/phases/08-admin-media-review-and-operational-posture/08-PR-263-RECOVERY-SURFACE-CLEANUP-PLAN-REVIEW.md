---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_plan: 08-PR-263-RECOVERY-SURFACE-CLEANUP-REVISED-PLAN.md
reviewed_commit: 2a258e2
reviewed: 2026-10-11
verdict: approved
blockers: 0
warnings: 0
info: 0
---

# PR 263 Recovery Surface and Cleanup Authority Plan Review

## Verification Passed

**Verdict:** **APPROVED — 0 Blocker, 0 Warning, 0 Info**

The revised plan closes CR-01, CR-02, and WR-01 as one coherent
recovery-authority contract change. It is implementation-ready for one bounded
coder pass after this review artifact and its PR comment are recorded in the
convergence evidence.

## Finding and Invariant Coverage

| Finding | Required invariant | Plan coverage | Verdict |
|---|---|---|---|
| CR-01 — `exclusive_conflict_recovery_surface` | Recovery remains visible and operable while ordinary editor authority is inactive | Task 1 places a dedicated recovery status/action container outside `#item-form`, centralizes every recovery transition on it, preserves the full acceptance predicate, and retires it on every authority change | Covered |
| CR-02 — `live_smoke_cleanup_discovery` | All possible private object identities are inventoried before destructive Oracle cleanup | Task 2 inventories current image keys, cleanup-event targets, and local candidates before assertions and before cleanup; explicit and `Drop` paths delete and verify objects before deleting rows | Covered |
| WR-01 — `live_smoke_cleanup_evidence_privacy` | Cleanup evidence contains only safe identifiers and domain-separated fingerprints | Task 3 replaces key-bearing and provider-controlled output across normal, timeout, panic, verification, and recovery-required paths and updates the operator runbook accordingly | Covered |

The plan correctly preserves the already-closed application-side
ambiguous-delete behavior: a committed readable replacement remains
authoritative, and this pass changes only recovery presentation, cleanup
discovery/order, and cleanup evidence.

## Producer, Consumer, and Boundary Coverage

The plan covers the complete relevant topology:

- Recovery producers and transitions: `handleMediaRevisionConflict`,
  `enterConflictRecoverySurface`, `recoverConflictedItem`,
  `renderConflictRecoveryRetry`, `leaveConflictRecoverySurface`, and
  invalidation.
- Recovery consumers: pending status, retry action, successful refresh,
  missing item, authentication failure, navigation, logout, direct
  render/reconciliation, and newer review ownership.
- Editor mutation boundaries: recovery generation, editor-authority generation,
  item ID, navigation revision, request owner, current view, recovery status,
  and exclusive form state remain in the acceptance predicate.
- Cleanup producers: locally known A/B candidates,
  `autograph_images.object_key`, and
  `autograph_cleanup_events.target_object_key`.
- Cleanup mutation boundaries: inventory, deduplication, object deletion,
  absence verification, Oracle row deletion, commit, and zero-row verification.
- Reporting/action paths: normal progress, already-absent objects, timeout,
  provider failure, verification failure, fallback-thread panic,
  recovery-required evidence, and interrupted-cleanup documentation.
- Tests: static markup contracts, browser-faithful DOM behavior, deterministic
  cleanup failure fixtures, privacy-bearing transcript fixtures,
  credential-gated live compilation, and full regression gates.

## Failure Matrix

| Scenario | Planned required outcome | Verdict |
|---|---|---|
| Recovery pending | Dedicated status remains perceivable outside the hidden/inert editor | Covered |
| Recovery retryable | Only the current generation-bound retry remains user-dispatchable | Covered |
| Recovery succeeds | Fresh item reopens under the complete acceptance predicate | Covered |
| Recovery becomes stale through navigation, logout, direct render, or newer review | Surface is retired and late callbacks cannot mutate editor state | Covered |
| B commits before a recovery-required response | Inventory discovers B from Oracle before response failure can unwind cleanup | Covered |
| Discovery fails | Discovery rows remain intact and redacted recovery-required evidence is emitted | Covered |
| Delete fails | Oracle discovery metadata remains; no destructive row cleanup proceeds | Covered |
| Absence verification fails | Rows remain available for recovery and failure evidence is redacted | Covered |
| Partial prior cleanup or repeated cleanup | Candidates are deduplicated; provider 404 is accepted only under the existing exact classification | Covered |
| Normal cleanup output | Safe counts, item ID, recovery ID where available, and fingerprints only | Covered |
| Timeout or provider failure | Provider-controlled text and private identifiers are not forwarded | Covered |
| Fallback panic | Safe recovery-required evidence is emitted without raw identities | Covered |

## Privacy Boundary

The plan expressly prohibits raw object keys, raw media checksums, credentials,
namespace or bucket values, and unsanitized provider responses from cleanup
evidence. It treats raw keys as internal data only, adds a domain-separated
fingerprint helper, requires distinctive secret-marker fixtures, and makes the
tests fail if any seeded private value reaches captured output.

The runbook change also closes the operator-action sibling path: item-ID
discovery is preferred, privately supplied candidate keys are consumed without
echoing, and expected evidence is fingerprint-based.

## Regression Safety

The change is correctly bounded away from closed Phase 8 contracts. The plan
preserves:

- recovery and editor-authority generations;
- request ownership and late-result rejection;
- media revision and typed conflict behavior;
- CAS and replacement-authoritative ambiguous-delete handling;
- gesture, mounted-output, resize-projection, and image-review mutation behavior;
- publisher promotion and migration contracts;
- public-artifact privacy and the ban on direct private Object Storage exposure.

Focused tests are supplemented by
`cargo test --all-targets --all-features` and all-feature Clippy, giving the
closed sibling contracts appropriate regression coverage.

## Verification Sufficiency

The command set is runnable from the repository root and covers:

- JavaScript syntax and executable DOM behavior;
- static admin contracts;
- cleanup and logging contracts;
- credential-gated live-smoke compilation;
- the complete all-target/all-feature Rust suite;
- all-feature Clippy with warnings denied;
- formatting and whitespace integrity;
- mandatory PR checks after push.

Live Oracle/OCI execution remains correctly opt-in and is not replaced by
unsafe production fault injection.

## Scope and Convergence Compliance

Three tightly connected tasks touch the exact presentation, cleanup, test, and
operator-documentation surfaces implicated by the stable findings. The work is
coherent and bounded for one coder pass.

The plan satisfies the `AGENTS.md` convergence guard:

1. Finding lineage is preserved.
2. Shared invariants and complete consumer inventories are incorporated.
3. The assumption audit and failure matrix are durable in the convergence
   artifact.
4. Approval requires zero Blocker, Warning, and Info findings.
5. This review artifact and its PR comment must be recorded before changing
   convergence to `ready_to_resume`.
6. A subsequent lineage-preserving deep source review must return zero
   actionable findings before closure.

No files were edited, no tests were executed, and no PR or repository state was
changed during this independent plan review.
