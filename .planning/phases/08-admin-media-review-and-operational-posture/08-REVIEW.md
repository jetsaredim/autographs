---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-12T00:48:37Z
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
  - controller/db/schema.sql
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
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
convergence:
  status: clear
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-CONVERGENCE.md
  plan_evidence: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-263-CANDIDATE-OWNERSHIP-PLAN-REVIEW.md
---

# Phase 08: PR #263 Terminal Ownership Deep Code Review

**Reviewed:** 2026-10-12T00:48:37Z
**Reviewed commit:** `9439b248d82255db77cd59cfb9a6b29ca6213f3d`
**Implementation commit:** `c3566008f196aba8a761045e441446483447bd15`
**Base:** `main` (`f751364c0856bc82ce229e247341a4c2935f4a53` merge base)
**Depth:** deep
**Files Reviewed:** 20
**Status:** clean
**Convergence:** clear

## Narrative Findings (AI reviewer)

### Summary

The complete PR diff was reviewed against `main`, with iteration-10 CR-02
(`live_smoke_cleanup_discovery`) traced through the iteration-11 implementation
as a terminal ownership contract. The review inspected initial candidate
production, durable registration and commit, object creation, replacement
candidate production, atomic inventory refresh, normal and fallback cleanup,
fresh-process recovery, Oracle warning loading, retry-route actionability,
object verification, row destruction, reporting, privacy, and test fidelity.

No actionable Critical, Warning, or Info findings remain. The implementation
meets the approved revised plan without adding a new capability or expanding
Phase 8 scope. Credentialed Oracle/OCI execution was unavailable, so live
provider behavior remains supported by source inspection, compiled ignored
smoke coverage, and deterministic production-helper/repository/route tests
rather than `live_cleanup_verified` evidence.

All reviewed files meet quality standards. No issues found.

### Stable finding lineage

| Stable finding | Finding family | Historical lineage | Iteration-12 disposition |
| --- | --- | --- | --- |
| CR-01 | `exclusive_conflict_recovery_surface` | `sibling_path_miss` | Closed. The status and only retry action remain outside the hidden/inert item form; browser-faithful tests prove perceivability, ancestor inertness, retry, authority replacement, navigation, and stale-result rejection. |
| CR-02 | `live_smoke_cleanup_discovery` | `incomplete_fix` | Closed. A is durably registered and committed before object creation; fallible refresh cannot consume installed candidates; private item-ID discovery remains inclusive; object absence precedes row deletion; final guard destruction leaves enough Oracle authority for a fresh cleanup process. |
| WR-01 | `live_smoke_cleanup_evidence_privacy` | `sibling_path_miss` | Closed. Normal, manual, fallback, timeout, provider-failure, verification, registration-failure, and runbook paths expose item/step/count/fingerprint evidence without raw keys or provider-controlled error text. |

Previously closed mounted-output authority, normalized perspective projection,
mutation/navigation serialization, pointer-capture isolation, immutable media
revision compare-and-set, replacement-authoritative ambiguous-delete recovery,
publisher promotion safety, comparison-map migration, and public-artifact
privacy remain closed in the current source and full regression suite.

### Terminal ownership invariant inventory

| Boundary | Producers | Consumers / mutation boundary | Failure behavior verified |
| --- | --- | --- | --- |
| Initial A authority | UUID-derived A key; disposable Oracle item | `register_smoke_cleanup_authority`, Oracle commit, then `media.write` | Insert or commit failure aborts before object creation; errors retain only item/step context. The cleanup event has an item FK but intentionally does not require a pre-existing image row. |
| Route-generated B authority | Replacement write and metadata CAS; ambiguous old-object deletion; cleanup-event persistence | Current `autograph_images` row or cleanup-event target; smoke inventory | Every response/unwind state that leaves B authoritative keeps it discoverable before destructive item-row cleanup. Replacement regressions retain the readable committed object on ambiguous delete. |
| Atomic candidate refresh | Installed local set plus image and cleanup-event queries | `refresh_cleanup_candidates`, `discover_cleanup_object_keys`, normal cleanup, `Drop` | Query, row-read, decode, and partial-discovery failure cannot mutate the installed set; retry installs a deduplicated union only on success. |
| Terminal recovery | Durable A authority; current B image/event rows | Fallback `Drop`; later manual item-ID cleanup | Repeated discovery failure retains Oracle rows. After guard destruction, a fresh cleanup with no local candidates rediscovers A/B, deletes and verifies both, then removes rows. |
| Actionability boundary | `delete`, `replace`, reserved `livePersistenceSmokeGuard`, unknown future operations | Memory warning loader, Oracle warning loader, item/status responses, cleanup-retry route | One shared allowlist admits exactly `delete` and `replace`; the route rejects an injected reserved warning before object deletion, metadata removal, or event resolution. Ordinary delete/replace retry tests remain green. |
| Destructive ordering | Complete deduplicated candidate inventory | OCI DELETE, 404/read absence proof, item cascade/commit, zero-row verification | Every candidate is verified absent before the first destructive Oracle row operation; any inventory/delete/verification failure retains discovery rows. |
| Recovery evidence | Registration, inventory, delete, read, runtime/thread, Oracle cleanup | Console/CI transcript and operator runbook | Output uses safe item IDs, step labels, counts, categories, and domain-separated fingerprints. Raw candidates, checksums, provider strings, namespace, and bucket data are not forwarded. |

### Failure matrix

| Scenario | Required outcome | Reviewed outcome |
| --- | --- | --- |
| Authority insert or commit fails | No A object is written | Satisfied: registration is a checked precondition before `media.write`. |
| A write/image-row insertion fails after authority commit | A remains discoverable and cleanup precedes row deletion | Satisfied: the authority event already owns A and the guard inventories it. |
| Discovery fails before or after partial row reads | Installed candidates remain unchanged; no object/row destruction begins | Satisfied: discovery receives a clone and assignment occurs only after `Ok`. |
| First discovery fails and fallback retry succeeds | Preserve A, merge B/event targets, deduplicate, verify all absent, then delete rows | Satisfied by the production helper tests plus cleanup ordering fixture. |
| Discovery fails again and the original guard is destroyed | Durable rows retain every actionable identity for a fresh item-ID cleanup | Satisfied: A was committed before creation; B remains in current image/event metadata; fresh cleanup coverage starts without local guard state. |
| Reserved authority reaches normal admin reads | It remains private and non-actionable | Satisfied in Memory and Oracle loaders; item/status behavior omits it. |
| A repository bypasses loader filtering | Route rejects before every media/catalog/event side effect | Satisfied by injected-repository route coverage. |
| Ordinary delete or replace warning is retried | Existing deletion/metadata distinction remains reachable | Satisfied by positive delete, replacement, idempotent, and second-replacement regressions. |
| Candidate is duplicate or already absent | Cleanup is idempotent and rows are deleted only after absence proof | Satisfied by `BTreeSet` inventory and exact provider-404 handling. |
| Delete, read verification, runtime, thread, or Oracle cleanup fails | Preserve discovery authority and emit opaque recovery evidence | Satisfied in source and deterministic tests; credentialed live execution was not available. |

### Test fidelity assessment

- The decisive atomic-refresh tests call the same
  `refresh_cleanup_candidates` helper used by `LivePersistenceSmokeCleanup`;
  they are not a parallel model of the ownership transfer.
- The terminal fresh-cleanup fixture intentionally begins with no local
  candidates and reconstructs the union from durable current-image and
  cleanup-event rows, matching the post-`Drop` recovery claim.
- The register-before-create fixture is supplemented by a source-order binding
  to the live smoke. Actual Oracle insert/commit and OCI write remain
  credential-gated, so this is deterministic/source evidence, not a live proof.
- The injected warning repository deliberately bypasses both loader filters,
  making the route-defense test a real side-effect reachability test rather
  than a filter-only assertion.
- The admin DOM harness models ancestor `hidden`/`inert` behavior and exercises
  the dedicated recovery sibling, so CR-01 closure does not rely on direct
  dispatch to a browser-inoperable descendant.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test media_cleanup` — passed, 26 tests.
- `cargo test --manifest-path controller/Cargo.toml --test logging_contract` — passed, 3 tests.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke` — passed, 4 executed and 1 credential-gated test ignored.
- `node --check controller/static-admin/admin.js` — passed.
- `node --check controller/tests/static_admin_behavior.mjs` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 2 credential-gated live tests ignored as designed and all executed tests passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check main...HEAD -- controller docs/static-runtime-runbook.md` — passed.

---

_Reviewed: 2026-10-12T00:48:37Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep_
_Iteration: 12_
