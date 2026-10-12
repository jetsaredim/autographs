---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: 73cb7694b6e540f24d4fe87b5d67a3d70356272b
reviewed: 2026-10-07
review_comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-6037438527
verdict: approved
blockers: 0
warnings: 0
info: 0
github_controller_check: https://github.com/jetsaredim/autographs/actions/runs/37617365860/job/112778844905
github_clippy_status: diagnosed_pending_fix_and_rerun
---

# PR 263 Ambiguous Delete and Exclusive Recovery Plan Review

## Verdict

**APPROVED — zero blockers, zero warnings, and zero advisories.**

The `Ambiguous Delete and Exclusive Recovery Addendum` closes both Critical findings from the media-recovery source review without weakening the previously approved media-revision, CAS, privacy, or response contracts. It is implementation-ready as one coherent coder pass after this artifact and PR comment are recorded in the convergence evidence.

The current GitHub Controller checks job is red, but it is no longer unexplained: CI reports a stable-toolchain deprecation at `controller/tests/media_cleanup.rs:1344` (`AtomicUsize::fetch_update` must become `try_update`). This is an execution gate, not a plan-design finding. The addendum explicitly requires the exact CI Clippy command and all exact CI gates to pass before implementation closes, so the diagnostic must be fixed and a subsequent GitHub run must be green.

## Finding Lineage

| Prior finding | Addendum evidence | Verdict |
|---|---|---|
| Recovery source CR-01: ambiguous old-object DELETE could roll metadata back to missing bytes and delete the readable replacement | Once replacement metadata commits, every old-object DELETE error is outcome-ambiguous. The addendum forbids restoration and replacement deletion after any attempted old-object DELETE error; replacement remains authoritative whether the provider applied the delete or retained the old object. | Closed. |
| Recovery source CR-02: late refresh could overwrite same-view editor work | Recovery owns a dedicated inert surface and captures both recovery and editor-authority generations. Every ordinary input, Save, item assignment, render, reconciliation, saved-response application, navigation, logout, or new review is blocked or establishes newer editor authority before proceeding. | Closed. |
| Media failure-recovery plan review Iterations 1-3 | Typed CAS outcomes, proof-gated deletion, exact redacted bodies, UUID-correlated private diagnostics, status-bearing source acquisition, and generation-safe 200/404/500/network/auth recovery states remain explicitly in force. | Preserved. |
| Gesture/output/mutation/publisher/migration/privacy lineage | The addendum changes only ambiguous deletion and editor ownership; the completion criteria retain the full lineage-preserving deep review and exact regression gates. | Preserved. |

## Goal-Backward Verification

### Replacement-authoritative ambiguous DELETE

| Required property | Evidence | Verdict |
|---|---|---|
| DELETE returning `Err` never proves the original still exists | The addendum treats both applied-then-error and retained-then-error as the same ambiguous outcome. | Covered. |
| Known-readable replacement remains catalog authority on every ambiguous outcome | No rollback occurs after attempted old-object deletion; neither candidate is deleted when warning persistence also fails. | Covered. |
| Warning persistence success remains an ordinary successful replacement with cleanup evidence | Replacement response remains successful and carries the persisted cleanup warning while the old key is only a cleanup candidate. | Covered. |
| Warning persistence failure is safe and diagnosable | Replacement remains authoritative, every possibly existing object is preserved, and the route returns the established redacted `imageRecoveryRequired` response with UUID-correlated private `old_delete_outcome_ambiguous` evidence. | Covered. |
| Earlier rollback semantics cannot re-enter the unsafe path | The addendum explicitly supersedes the old existence assumption and forbids the rollback branch after an old-object DELETE attempt. Any retained rollback applies only before deletion begins under separately proven object assumptions. | Covered. |

### Exclusive recovery ownership

| Required property | Evidence | Verdict |
|---|---|---|
| Pending/retryable recovery exposes no editable blank form or stale item actions | Conflict recovery clears item/review authority and renders a dedicated inert surface. Only the bound retry action may be active in retryable state. | Covered. |
| Same-view activity cannot be mistaken for unchanged authority | A separate monotonic editor generation changes on normal assignment, render, reconciliation, saved response, navigation, logout, or new review. Form and programmatic handlers reject invocation while recovery owns the surface. | Covered. |
| Recovery acceptance has a complete predicate | Recovery generation, captured editor generation, item ID, navigation context, request ownership, and dedicated recovery state must all match. | Covered. |
| Every producer/consumer boundary has deterministic evidence | Tests defer recovery across user input, programmatic Save, direct render, saved-response application, navigation, logout, and new review; each is blocked or invalidates the late result. Retry uses the same predicate. | Covered. |

### Deterministic and live validation

| Required property | Evidence | Verdict |
|---|---|---|
| Provider ambiguity is deterministic in CI | Test adapters cover both delete-then-`Err` and retain-then-`Err`, with warning persistence success/failure, catalog key, replacement readability, object preservation, response/evidence redaction, and no replacement deletion. | Covered. |
| Production validation uses disposable private data only | Live mode requires two explicit opt-ins, creates a uniquely marked draft, never publishes it, and uses small known images with locally recorded checksums. | Covered. |
| Production is not used for fault injection | Ambiguous deletes, database-warning failures, transport faults, and partial outages are expressly confined to local/CI adapters. | Covered. |
| Live validation proves the real authority boundary | The sequence obtains revision A, replaces with B, asserts stale-A source/draft/assist/Save/Reset conflicts, reloads, proves B succeeds, audits redaction, and verifies Oracle/Object Storage cleanup. | Covered. |
| Interrupted cleanup is operationally recoverable | The runbook must include exact opt-ins, prerequisites, evidence, abort conditions, verified cleanup, and manual recovery steps without logging credentials, raw object keys, or checksums. | Covered. |

## GitHub Clippy Gate

The proposed gate is sufficient and currently active:

- CI run `37617365860`, Controller checks job `112778844905`, failed only at the exact Clippy step after formatting, runtime validation, and production-feature coverage passed.
- The captured diagnostic is `std::sync::atomic::Atomic::<usize>::fetch_update` deprecated at `controller/tests/media_cleanup.rs:1344`; CI recommends `try_update`.
- The plan requires the exact CI command: `cargo clippy --manifest-path controller/Cargo.toml --all-targets --features production-persistence -- -D warnings`.
- The plan also requires the exact production-feature coverage command and focused/full regressions.
- Local success does not close the gate. The diagnostic must be fixed and a new GitHub Controller checks run must pass before implementation completion.

## Implementability and Boundedness

- Server work is localized to the replacement cleanup branch and deterministic media/repository fixtures; it removes an unsafe rollback decision rather than adding a second recovery mechanism.
- Client work centralizes ownership in one editor generation and one dedicated recovery surface, extending the already approved recovery record rather than distributing new booleans across handlers.
- The required user and programmatic action inventory is explicit enough to prevent sibling-path omissions.
- Live validation extends the existing credential-gated Oracle/OCI smoke and its cleanup/runbook pattern; it does not add production fault injection or publishing.
- The Clippy diagnostic is in `media_cleanup.rs`, a file necessarily touched by the new deterministic delete fixtures, so its repair fits the coherent coder pass and remains independently enforced by CI.

## Approval and Resume Recommendation

Record this artifact and its PR comment as `implementation_plan_review_evidence` and `implementation_plan_review_comment` in `08-PR-263-CONVERGENCE.md`, then move convergence from `reassessment_required` to `ready_to_resume`. One coherent coder pass may proceed.

Implementation must not close until deterministic tests, live-smoke compile contracts, exact CI Clippy/coverage gates, cleanup/privacy checks, and the lineage-preserving deep source review all pass with zero Critical, Warning, and Info findings.

No implementation code or tests were changed or executed during this plan-only review.
