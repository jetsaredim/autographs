---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-24T14:43:11Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 2
findings_in_scope: 2
finding_families: 2
fixed: 2
skipped: 0
reassessment_required: false
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-24T14:43:11Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Approved assumptions:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-ASSUMPTIONS.md`
**Reviewed plan:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-REVISED-PLAN.md`
**Plan review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-PLAN-REVIEW.md` (`passed`, cycle 3)
**Iteration:** 2

## Summary

- Findings in scope: 2
- Finding families: 2
- Fixed: 2
- Skipped: 0
- Reassessment required: no
- Source commits: `16241f6`, `f82229c`

The cycle-3 plan closes the two round-2 findings under the user-approved single-operator local-workbench contract. Browser Save is single-flight and stale-safe, fixture publication no longer shares a fixed temporary path, and the operator runbook now preserves the capability through launch and deep links. Concurrent external API writers and multi-user use remain explicitly unsupported; no mutex, ordinal, barrier, custom temp naming, or winner-order contract was added.

## Fixed Issues

### WR-01: Client ownership does not protect the fixture mutation itself

**Finding family:** `workbench_async_authority`

**Lineage:** `incomplete_fix`

**Files modified:** `.planning/spikes/011-local-image-workbench/workbench/Cargo.toml`, `.planning/spikes/011-local-image-workbench/workbench/src/main.rs`, `.planning/spikes/011-local-image-workbench/workbench/static/app.js`, `.planning/spikes/011-local-image-workbench/workbench/static/app.test.cjs`

**Commit:** `16241f6`

**Applied fix:** Promoted the already-locked `tempfile 3.27.0` dependency from dev-only to runtime use without changing `Cargo.lock`. Fixture save now validates and serializes the complete document, creates a unique `NamedTempFile` inside the workspace, writes and flushes the complete JSON through `std::io::Write`, and publishes through same-directory `persist`. Pre-persist failures rely on RAII cleanup; persist failure explicitly drops the returned temporary file before surfacing the underlying API error. A focused Rust test replaces an existing fixture, parses and validates the stable document, verifies the replacement value, and proves that only the stable fixture remains in the workspace.

The browser now uses one exported DOM-independent Save coordinator in production and tests. It captures the owner and canonical snapshot once, marks pending synchronously, rejects duplicate activation, persists through `api` rather than abort-controlled `ownedApi`, publishes only for the current owner, reports only current-owner errors, and clears pending after the request and response body settle. Pending disables Save; stale success cannot change adjustment, option marker, success status, or event log. There is no implicit queue or autosave, and a later explicit Save captures a new snapshot.

**Supported limitation:** one local operator and one browser session. Unique temporary files remove the shared-temp collision, but concurrent direct API requests have no defined winner and remain unsupported. Multi-user productization requires the separately documented version/conflict contract.

**PR comment:** https://github.com/jetsaredim/autographs/pull/281#issuecomment-6385523151

### WR-06: The capability fix left the documented launch path unusable

**Finding family:** `operator_capability_launch_contract`

**Lineage:** `sibling_path_miss` from CR-01

**Files modified:** `.planning/spikes/011-local-image-workbench/README.md`

**Commit:** `f82229c`

**Applied fix:** The runbook now directs the operator to copy the exact process-emitted `?cap=...` URL, explains that the bare shell cannot authorize private APIs and missing-cap API requests receive `403 Forbidden`, and treats the live token as sensitive. Image deep links retain both `cap` and `image`; bare `?image=N` is explicitly insufficient. The README records the one-operator/session boundary, unsupported external writers, browser single-flight behavior, and the multi-user productization blocker: durable versions or ETags, actor/audit identity, serialized or compare-and-swap commits, conflict/retry UX, and deterministic concurrency/recovery/durability tests.

**PR comment:** https://github.com/jetsaredim/autographs/pull/281#issuecomment-6385547117

## Verification

Verification ran in the main checkout because `.planning/config.json` sets `workflow.use_worktrees` to `false`.

### RED-first evidence

- `node --test static/app.test.cjs` initially passed 9 existing tests and failed the 3 new Save tests because `createSaveCoordinator` was absent and production Save still used `ownedApi`.
- The new Rust replacement/output cleanup test passed against the prior successful-save path; it became focused acceptance coverage for `NamedTempFile` replacement rather than a behaviorally RED test.

### Focused and full evidence

- Workbench `cargo test --release` — passed, 12 tests.
- Workbench strict `cargo clippy --all-targets -- -D warnings` — passed.
- Workbench `cargo fmt -- --check` — passed.
- `node --check static/app.js` — passed.
- `node --test static/app.test.cjs` — passed, 12 tests, including duplicate suppression, synchronous pending, stale non-publication, current-owner errors, settlement cleanup, later explicit snapshot, and production non-aborting seam.
- `cargo test --manifest-path controller/image-processing/Cargo.toml` — passed, 2 unit tests plus doc tests.
- `cargo test --manifest-path controller/Cargo.toml --test image_adjustments` — passed, 23 tests.
- `cargo test --manifest-path controller/Cargo.toml --lib` — passed, 14 tests.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- Ordinary controller `cargo clippy --manifest-path controller/Cargo.toml --all-targets` — passed with the pre-existing `oracle_wallet_dir` dead-code warning. A stricter exploratory `-D warnings` invocation failed only on that same read-only, pre-existing field; production controller source was not changed.
- `git diff --check` — passed.
- `Cargo.lock` remained byte-identical with SHA-256 `106a06f5e4d4fec7d9d867d9a65bf529308f24f5d4ddc61db1b52e8c190ba611`.

### Capability-aware smoke

The current release binary was exercised with a disposable local source and external workspace. The exact emitted capability URL returned `200`; list, source, save, reload, and capability-preserving deep-link requests returned `200`; the same private API without the capability returned `403`; the source SHA-256 was unchanged; the reloaded adjustment matched the saved value; and the workspace contained exactly one stable fixture with no temporary artifact. No live token or private content was recorded.

Port 8788 was already held by another process in the execution environment, so the current binary used an ephemeral loopback port for the automated capability-flow smoke. The standard default-port behavior remains covered by `uses_the_single_documented_default_port` in the release workbench suite.

### Final invariant audit

The final diff was re-audited across Save activation, snapshot capture, pending lifecycle, abort ownership, adjustment/option/status/log publication, fixture serialization, temporary-file cleanup, stable read/reload/list consumers, capability generation and injection, history/deep-link retention, runbook instructions, negative authorization, and regression tests. No new in-scope Critical or Warning issue was found. Host, Origin, capability, and `Cache-Control: no-store` behavior were not weakened; production controller persistence and shared image-processing code remained unchanged.

## Cumulative PR Audit Trail

Iteration 1:

- CR-01 — `65cc8e9`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6372949681
- WR-01 initial ownership fix — `4fffcb6`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6372990061
- WR-02 — `2908cac`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6373011813
- WR-03 — `59179e8`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6373036422
- WR-04 — `65f3251`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6373085287
- WR-05 — `71f43f5`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6373109361

Iteration 2 closeout:

- WR-01 supported-scope completion — `16241f6`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6385523151
- WR-06 — `f82229c`: https://github.com/jetsaredim/autographs/pull/281#issuecomment-6385547117

Pull request: https://github.com/jetsaredim/autographs/pull/281

---

_Fixed: 2026-10-24T14:43:11Z_
_Fixer: Codex (gsd-code-fixer)_
_Iteration: 2_
