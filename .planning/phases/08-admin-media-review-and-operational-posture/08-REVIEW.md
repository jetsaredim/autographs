---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-11T02:08:55Z
depth: deep
files_reviewed: 20
files_reviewed_list:
  - controller/src/catalog.rs
  - controller/src/image_adjustments.rs
  - controller/src/media.rs
  - controller/src/oci_media.rs
  - controller/src/oracle_catalog.rs
  - controller/src/publisher.rs
  - controller/src/routes.rs
  - controller/static-admin/admin.css
  - controller/static-admin/admin.js
  - controller/static-admin/index.html
  - controller/tests/admin_workflow.rs
  - controller/tests/live_persistence_smoke.rs
  - controller/tests/logging_contract.rs
  - controller/tests/media_cleanup.rs
  - controller/tests/publisher.rs
  - controller/tests/static_admin.rs
  - controller/tests/static_admin_behavior.mjs
  - controller/tests/static_contract.rs
  - docs/static-runtime-runbook.md
  - .github/workflows/ci.yml
findings:
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
convergence:
  status: reassessment_required
  recurring_critical_warning: 3
  scope_expansion: false
  requires_reassessment: true
  assumptions_evidence: null
  plan_evidence: null
---

# Phase 08: PR #263 Deep Code Review — Exclusive Recovery and Live Smoke

**Reviewed:** 2026-10-11T02:08:55Z
**Reviewed commit:** `6bb44af1556efd2fdbe7424b4335aa7cb7856594`
**Base:** `main`
**Depth:** deep
**Files Reviewed:** 20
**Status:** issues_found
**Convergence:** reassessment_required

## Narrative Findings (AI reviewer)

### Summary

This independent review re-read the complete PR issue-comment history, confirmed that the inline review-comment history is empty, and re-verified the interrupted report already posted as PR comment `6038097203` against the current source. All three findings remain reproducible at the reviewed HEAD. No additional actionable finding survived the full producer/consumer, mutation-boundary, reporting-path, and test audit.

The application-side ambiguous-delete fix itself remains sound: once replacement metadata commits, the route keeps the known-readable replacement authoritative after every ambiguous old-object DELETE error, including cleanup-warning persistence failure. The current defects sit in sibling operational surfaces. The exclusive recovery message and retry are inside the form that recovery hides and marks inert; the production live-smoke cleanup guard cannot discover a committed B key before an unexpected post-commit error aborts the test; and the documented manual cleanup command emits the private key it is meant to keep out of evidence.

Because PR #263 is already beyond the mandatory third-review checkpoint and three in-scope Critical/Warning findings remain, AGENTS.md forbids another point-fix round. A durable reassessment and independently reviewed revised plan are required before coder work resumes.

### Stable finding lineage

| Stable finding | Finding family | Lineage classification | Current disposition |
| --- | --- | --- | --- |
| CR-01 | `exclusive_conflict_recovery_surface` | `sibling_path_miss` | The prior same-view recovery authority defect is fixed in state/handler logic, but its dedicated presentation is hidden and inert with the editor it disables. |
| CR-02 | `live_smoke_cleanup_discovery` | `incomplete_fix` | The production-safe live validation has normal-path cleanup, but its guard does not discover route-generated keys before status assertions and cannot clean a post-commit failure. |
| WR-01 | `live_smoke_cleanup_evidence_privacy` | `sibling_path_miss` | Route diagnostics are fingerprint-only, but the operator cleanup/reporting sibling path prints raw private identifiers. |

Previously reviewed media-revision CAS, source fetch-to-blob authority, gesture lifecycle, mounted-output authority, ambiguous DELETE behavior, publisher promotion, comparison-map migration, and public artifact privacy remain closed in the inspected implementation and regressions.

## Critical Issues

### CR-01: Recovery status and retry are descendants of the form that recovery hides and makes inert

**Classification:** BLOCKER
**Finding Family:** `exclusive_conflict_recovery_surface`
**Lineage:** `sibling_path_miss`
**File:** `controller/static-admin/admin.js:1710-1748`
**Related:** `controller/static-admin/index.html:158-316`, `controller/static-admin/admin.css:16-18`, `controller/tests/static_admin_behavior.mjs:1138-1254`
**Evidence Level:** `source_only`

**Shared invariant:** While conflict recovery owns the editor, ordinary item/image actions must remain inactive, but pending/retryable recovery status and the generation-bound retry action must remain visible and operable outside every hidden or inert ancestor.

**Consumers inspected:** The current-conflict entry path, `enterConflictRecoverySurface`, `recoverConflictedItem` success/404/401/403/500 branches, `renderConflictRecoveryRetry`, retry dispatch, editor-authority invalidation, direct editor render, navigation, logout, new-review invalidation, the `#item-form`/`#image-message` DOM topology, the global `[hidden]` CSS rule, static-admin source contracts, and the executable fake-DOM harness.

**Issue:** `enterConflictRecoverySurface()` sets `itemForm.inert = true` and `itemForm.hidden = true`, then writes the pending message to `imageMessage`. The markup places `#image-message` inside `#item-form`. If the refresh fails, `renderConflictRecoveryRetry()` appends the only retry button to that same hidden, inert descendant. The global `[hidden]` rule removes the form from layout, and inertness prevents descendant interaction even if another style exposed it. A real browser therefore shows neither recovery progress nor an operable retry. The harness directly dispatches to `imageMessage.children[1]` and models each referenced ID as an independent node, so it does not enforce ancestor hidden/inert semantics and produces a false green.

**Fix direction for the revised plan:** Establish a dedicated recovery status/action container outside `#item-form`, or inert/hide only an editor-content wrapper that excludes recovery presentation. Route pending, retryable, missing, auth, and successful recovery states through that surface; hide/retire it on every editor-authority change. Add browser-faithful evidence that the recovery message is perceivable and retry is user-dispatchable while all ordinary editor descendants are inactive.

### CR-02: A post-commit replacement error can leave live-smoke B outside automatic cleanup

**Classification:** BLOCKER
**Finding Family:** `live_smoke_cleanup_discovery`
**Lineage:** `incomplete_fix`
**File:** `controller/tests/live_persistence_smoke.rs:239-253`
**Related:** `controller/src/routes.rs:1573-1657`, `controller/tests/live_persistence_smoke.rs:97-103`, `controller/tests/live_persistence_smoke.rs:1297-1425`, `docs/static-runtime-runbook.md:650-674`
**Evidence Level:** `source_only` (credential-gated live execution was unavailable)

**Shared invariant:** Disposable production smoke cleanup must discover and retain every possibly existing private object identity before deleting the Oracle rows that provide that identity, regardless of the HTTP result returned after a storage/metadata mutation.

**Consumers inspected:** Replacement-object generation/write, memory and Oracle expected-key metadata mutation, ambiguous old-object deletion, cleanup-warning persistence success/failure, exact recovery-required response, the smoke's response assertion and repository reload, the cleanup guard's explicit and `Drop` paths, Oracle cascade deletion, object absence verification, manual cleanup mode, runbook abort procedure, deterministic media-cleanup tests, and the credential-gated live test/CI compile path.

**Issue:** The cleanup guard starts with A only. The replacement route generates B internally, commits B as the catalog authority, and can then return `imageRecoveryRequired` if old-object DELETE fails and cleanup-warning persistence also fails. The smoke asserts HTTP 200 before reloading Oracle and registering B. That assertion panics on the valid post-commit recovery response; `Drop` then deletes only its known A key and deletes the Oracle item. Cascading row removal destroys the remaining automatic lookup for B, leaving a private orphan. The redacted recovery response intentionally provides only a fingerprint, so it cannot reconstruct the key after metadata deletion.

**Fix direction for the revised plan:** Make cleanup discovery precede result assertions. The guard should own the disposable item ID, query all current image and cleanup-event targets before any row deletion, merge/deduplicate them with locally known candidates, delete/verify every discovered object, and only then delete Oracle rows. Preserve the response for later contract assertions. Add deterministic failure injection for “B committed, recovery-required returned” and prove unwind cleanup removes B and all rows without log-exposed keys.

## Warnings

### WR-01: The documented interrupted-cleanup command prints private object keys

**Classification:** WARNING
**Finding Family:** `live_smoke_cleanup_evidence_privacy`
**Lineage:** `sibling_path_miss`
**File:** `controller/tests/live_persistence_smoke.rs:1060-1068`
**Related:** `controller/tests/live_persistence_smoke.rs:990-1088`, `controller/tests/logging_contract.rs:62-86`, `docs/static-runtime-runbook.md:661-674`
**Evidence Level:** `source_only`

**Shared invariant:** Production-smoke and recovery transcripts may identify disposable item IDs and domain-separated key fingerprints, but must not emit raw object keys, media checksums, credentials, bucket/namespace data, or provider errors that reproduce those identifiers.

**Consumers inspected:** Cleanup environment parsing, cleanup progress output, timeout panic text, provider delete/read errors, object-absence verification, the operator runbook's expected-evidence and interrupted-recovery instructions, route logging contracts, and current live-smoke tests.

**Issue:** The runbook says expected evidence must not contain object keys, then directs the operator to the cleanup mode that prints `cleanup deleting OCI object key: {object_key}` and embeds the same raw key in its timeout panic. Other provider errors are forwarded into the cleanup transcript without a test proving they are identifier-free. A routine manual recovery capture can therefore violate the phase's explicit evidence-privacy contract.

**Fix direction for the revised plan:** Report counts, item IDs, and domain-separated fingerprints only. Remove raw keys from normal, timeout, panic, and forwarded provider-error output, and add log-capture tests for successful deletion, timeout, provider failure, and absence verification.

## Convergence Reassessment

### Finding lineage

The last approved ambiguous-delete/exclusive-recovery plan closed two application defects: it stopped rolling metadata back after outcome-ambiguous DELETE and introduced exclusive editor ownership. CR-01 is a sibling presentation omission in that exclusive surface, not a recurrence of the state-generation algorithm. CR-02 and WR-01 are incomplete/sibling operational-validation paths added to prove the same recovery contract in production. None expands Phase 8 scope; all three are within the approved recovery and live-smoke contract.

### Shared invariants

1. **Exclusive recovery must be both exclusive and usable.** Disabling ordinary editor authority cannot disable the only recovery status/action path.
2. **Cleanup discovery must precede destructive metadata cleanup.** Oracle rows are the authoritative discovery source for route-generated object identities and cannot be deleted until all possibly existing private objects are inventoried.
3. **Private recovery evidence remains opaque.** The fingerprint-only application contract must extend to smoke tools, failure messages, and operator transcripts.

### Complete consumer inventory

| Invariant | Direct producers | Consumers and mutation boundaries | Reporting/action paths | Relevant tests |
| --- | --- | --- | --- | --- |
| Exclusive usable recovery | 409s from review-open/source/draft/assist/Save/Reset; `handleMediaRevisionConflict`; `recoverConflictedItem` | `clearImageReviewState`, `invalidateEditorAuthority`, form inert/hidden state, item assignment/reconciliation, direct render, navigation, logout, new review, retry generation/abort owner | `#image-message`, retry button, global missing/auth messages, add-item view | `static_admin`, `static_admin_behavior.mjs`; current fake DOM omits ancestor semantics |
| Cleanup discovery before row deletion | replacement key generation/write; memory/Oracle metadata CAS; ambiguous old DELETE; cleanup-warning persistence | smoke status assertion/reload; guard object inventory; explicit cleanup; `Drop`; OCI delete/read; Oracle item delete/cascades; cleanup-event targets | live success/abort output; `LIVE_PERSISTENCE_RECOVERY_REQUIRED`; manual cleanup command/runbook | `media_cleanup` covers route outcomes; live smoke is compile-covered/credential-gated; no deterministic guard failure test |
| Opaque recovery evidence | route UUID/fingerprint diagnostics; smoke cleanup input; provider errors | normal cleanup output, timeout/panic, absence verification, operator copy/paste into PR/CI | runbook expected evidence and interrupted cleanup | `logging_contract` covers route source only; no cleanup transcript capture |

### Assumption audit

| Assumption | Result | Evidence |
| --- | --- | --- |
| Writing to `#image-message` remains visible after hiding the item form | False | `#image-message` is a descendant of the hidden form; `[hidden]` forces `display: none`. |
| A directly dispatched retry in the fake DOM proves browser operability | False | The harness does not model the real ancestor tree or inert descendant event suppression. |
| Replacement cannot create a new cleanup identity unless it returns 200 | False | B metadata commits before old-object cleanup and warning persistence; the latter can return a recovery-required 500. |
| Deleting the Oracle item is safe even when cleanup inventory is incomplete | False | Cascade deletion removes B's discoverable metadata while Object Storage is independent. |
| Supplying a raw key privately means cleanup may echo it | False | The runbook explicitly prohibits raw keys in smoke/recovery evidence and logs. |
| Green full-suite and CI results cover these boundaries | False | The live test is credential-gated; the fake DOM omits ancestor semantics; no cleanup log-capture test exists. |

### Failure matrix

| Scenario | Required outcome | Current outcome | Finding |
| --- | --- | --- | --- |
| Current media conflict; refresh pending | Visible non-editable recovery status; editor inactive | Entire form, including status, is hidden/inert | CR-01 |
| Current media conflict; refresh returns 500/network | Visible generation-bound retry; editor inactive | Retry is appended inside hidden/inert form | CR-01 |
| Recovery returns 200/404/auth | Fresh item, missing-item navigation, or logout under generation checks | State transitions are generation-safe; no defect found on these branches | Closed |
| Replacement commits B, then old DELETE and warning persistence fail | Smoke discovers B before asserting response; unwind deletes A/B then rows | Status assertion panics before B registration; `Drop` deletes A and rows only | CR-02 |
| Normal replacement returns 200 | Smoke registers B and verifies both objects/rows absent | Works on the inspected path | Closed |
| Manual cleanup succeeds | Transcript contains only safe counts/IDs/fingerprints | Raw object key is printed | WR-01 |
| Manual cleanup times out/provider fails | Redacted error with safe correlation | Timeout embeds key; provider text is forwarded without sanitization proof | WR-01 |

### Revised plan requirements

1. Define one recovery-surface DOM ownership contract that keeps status/retry outside every hidden/inert editor ancestor and retires it on every authority change.
2. Define one discovery-first smoke cleanup algorithm, including current image rows, cleanup-event targets, locally known candidates, deduplication, delete/404 handling, verification, and row-deletion ordering for explicit and unwind cleanup.
3. Define one cleanup evidence contract covering normal progress, timeout, provider failure, panic, and absence-verification messages with fingerprint-only correlation.
4. Add browser-faithful recovery presentation coverage, deterministic post-commit recovery cleanup coverage, and cleanup transcript capture tests.
5. Re-audit every direct producer, consumer, mutation boundary, reporting/action path, and test in the inventory above as one coherent contract change; do not split the work into three unrelated point fixes.

### Resume criteria

Coder work may resume only after:

1. A durable revised assumptions/reassessment artifact incorporates this lineage, consumer inventory, assumption audit, and failure matrix.
2. A revised implementation plan satisfies every requirement above.
3. An independent plan review approves the revised plan with zero Critical, Warning, and Info findings.
4. Both new evidence references are recorded in the convergence artifact before its status changes from `reassessment_required` to `ready_to_resume`; a status-only change is invalid.
5. The prior approved plan evidence is not reused as closure evidence because it predates these three current findings.

No further coder point-fix round is authorized by this report.

## Verification

- `gh pr view 263 --json ...` — confirmed open, ready-for-review PR #263, base `main`, reviewed HEAD `6bb44af1556efd2fdbe7424b4335aa7cb7856594`.
- Complete `issues/263/comments` history read; complete `pulls/263/comments` history read (zero inline comments).
- `gh pr checks 263` — all nine current checks pass.
- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed, with the CR-01 fidelity limitation above.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed: 230 non-ignored tests; 2 credential-gated live tests ignored.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check main...HEAD -- controller docs/static-runtime-runbook.md .github/workflows/ci.yml` — passed.
- The credential-gated production smoke was compile-tested only; it was not executed against live Oracle/OCI.

---

_Reviewed: 2026-10-11T02:08:55Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
