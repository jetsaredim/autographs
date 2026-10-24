---
phase: 08-admin-media-review-and-operational-posture
pr: 281
reviewed: 2026-10-24T15:20:57Z
reviewed_head: f82229c3d765f8d1064190e2ed13ed381bd466c9
depth: deep
review_round: 3
files_reviewed: 16
files_reviewed_list:
  - .planning/spikes/011-local-image-workbench/README.md
  - .planning/spikes/011-local-image-workbench/workbench/Cargo.toml
  - .planning/spikes/011-local-image-workbench/workbench/Cargo.lock
  - .planning/spikes/011-local-image-workbench/workbench/src/main.rs
  - .planning/spikes/011-local-image-workbench/workbench/static/app.css
  - .planning/spikes/011-local-image-workbench/workbench/static/app.js
  - .planning/spikes/011-local-image-workbench/workbench/static/app.test.cjs
  - .planning/spikes/011-local-image-workbench/workbench/static/index.html
  - controller/image-processing/src/image_adjustments.rs
  - controller/image-processing/src/derivatives.rs
  - controller/image-processing/src/lib.rs
  - controller/tests/image_adjustments.rs
  - controller/src/lib.rs
  - controller/src/routes.rs
  - controller/src/catalog.rs
  - controller/src/oracle_catalog.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
convergence:
  status: ready_to_resume
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-ASSUMPTIONS.md
  plan_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-PLAN-REVIEW.md
---

# Phase 08: Local Image Workbench Code Review

**Reviewed:** 2026-10-24T15:20:57Z
**Reviewed commit:** `f82229c3d765f8d1064190e2ed13ed381bd466c9`
**Depth:** deep
**Status:** clean
**PR:** #281

## Summary

No actionable Critical, Warning, or Info findings remain in the final round-3
scope. The implementation closes WR-01 under the user-approved single-operator
local-workbench contract and closes WR-06's capability-aware operator path.
Concurrent external writers, multiple browser sessions, and multi-user use are
explicitly unsupported; this review did not reimpose a multi-writer ordering
contract.

Fixture publication now creates a unique `NamedTempFile` in the workspace,
writes and flushes the complete serialized document, and publishes it with a
same-directory `persist`. No fixed temporary path remains. The locked
`tempfile 3.27.0` implementation returns the still-owned `NamedTempFile` in
`PersistError`; the handler explicitly drops it, and `TempPath::drop` removes
the file. Earlier write/flush errors also clean up through RAII. `Cargo.lock` is
byte-identical to round 2, so promoting `tempfile` from dev-only to runtime did
not introduce lockfile churn.

The production browser path uses the same DOM-independent Save coordinator as
the deterministic tests. It marks pending before the first await, rejects a
duplicate activation, captures one image/owner/adjustment snapshot, persists
through non-aborting `api`, gates all success/error publication on current
ownership, and clears pending only after the request and response body settle.
The Save control remains disabled for that interval. Navigation or editing can
make the result stale without aborting the accepted write; stale completion
cannot change the adjustment, fixture marker, status, or event log.

The runbook now makes the exact process-emitted capability URL the supported
entry point, documents capability-preserving image deep links and the expected
403 for missing capability, states the one-operator/session boundary, and
records the accepted multi-user productization blocker: versions or ETags,
actor/audit identity, serialized or compare-and-swap commits, conflict/retry
UX, and deterministic concurrency/recovery/durability tests.

## Narrative Findings (AI reviewer)

No actionable findings.

## Finding Lineage

| Finding | Family | Round-3 result | Evidence |
|---|---|---|---|
| CR-01 | `loopback_browser_origin_isolation` | Resolved | Host, capability, Origin, and API no-store boundaries remain intact; authorized and missing-cap smoke paths behaved as documented. |
| WR-01 | `workbench_async_authority` | Resolved within approved supported scope | Unique same-workspace temporary publication plus production-used single-flight, unabortable, stale-safe browser coordination; focused Rust and Node tests pass. |
| WR-02 | `framing_transform_round_trip` | Resolved | Constrained transforms suppress crop-driving actions and complete framing draft state remains round-trippable. |
| WR-03 | `ordered_convex_quadrilateral_validation` | Resolved | Shared validation remains used by production adjustment and manual-framing paths; focused shared/controller tests pass. |
| WR-04 | `boundary_confidence_preservation` | Resolved | Three-sided gradient evidence remains tentative and non-actionable while preserving conservative veto behavior. |
| WR-05 | `stable_fixture_source_identity` | Resolved | Stable path key plus source digest identity and fail-closed reload behavior remain covered. |
| WR-06 | `operator_capability_launch_contract` | Resolved | Exact emitted URL, deep-link retention, negative authorization, sensitive-token handling, and smoke steps are documented and exercised. |

## Supported-Scope Audit

- **Producer:** Save activation captures the current operation owner, image id,
  and canonical adjustment exactly once.
- **Browser mutation boundary:** one coordinator instance synchronously owns the
  pending flag and rejects accidental duplicate activation.
- **Server mutation boundary:** one complete JSON document is written to a
  unique workspace-local temporary file, flushed, then atomically replaced
  through `persist` on the supported filesystem.
- **Cleanup/recovery:** pre-persist failures drop the `NamedTempFile`; persist
  failures explicitly drop the returned file; successful replacement leaves
  only the stable fixture.
- **Consumers/reporting:** fixture list/load and stable source-identity checks
  remain unchanged; only current-owner success updates adjustment, option
  marker, status, and event log.
- **Capability contract:** printed URL, browser extraction/header injection,
  source query parameter, history deep link, middleware, README, and smoke
  instructions agree.
- **Shared production paths:** controller persistence and the extracted
  image-processing consumers were unchanged by the final fix and their focused
  regression tests remain green.

## Verification

- `cargo test --release --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml` — passed, 12 tests.
- `cargo clippy --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml --all-targets -- -D warnings` — passed.
- `cargo fmt --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml -- --check` — passed.
- `node --check .planning/spikes/011-local-image-workbench/workbench/static/app.js` — passed.
- `node --test .planning/spikes/011-local-image-workbench/workbench/static/app.test.cjs` — passed, 12 tests.
- `cargo test --manifest-path controller/image-processing/Cargo.toml` — passed, 2 tests plus doc tests.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments` — passed, 23 tests; only the previously documented unrelated `oracle_wallet_dir` dead-code warning was emitted.
- `git diff --check 71f43f5b38d97077874d7e7edfdeb7f9feed8c42..HEAD` — passed.
- `Cargo.lock` has no diff from `71f43f5b38d97077874d7e7edfdeb7f9feed8c42`.
- Disposable capability-aware smoke — emitted capability URL and deep link returned `200`; authorized list, fixture save, and reload returned `200`; the same private API without capability returned `403`; reload returned the saved adjustment; the workspace contained one stable fixture and no temporary artifact.
- Locked `tempfile 3.27.0` source was inspected directly for `PersistError` ownership and `TempPath` drop cleanup semantics.
- No source file was modified and no commit was created.

## Convergence Verdict

Round 3 is clean. No in-scope Critical or Warning remains, no finding recurred,
and no sibling path, regression, or test-weakness expansion was found. The
mandatory third-review checkpoint therefore authorizes convergence as
`ready_to_resume` using the assumptions and passed cycle-3 plan review recorded
in frontmatter.

---

_Reviewed: 2026-10-24T15:20:57Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep_
