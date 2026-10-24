---
phase: 08-admin-media-review-and-operational-posture
pr: 281
reviewed_head: 71f43f5b38d97077874d7e7edfdeb7f9feed8c42
status: ready_to_resume
source_review: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
revised_plan: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-REVISED-PLAN.md
checker_review: .planning/phases/08-admin-media-review-and-operational-posture/08-PR-281-WORKBENCH-PLAN-REVIEW.md
checker_status: passed_cycle_3
reassessment_iteration: 2
product_decision: single-operator-local-workbench
---

# PR #281 Workbench Assumption Reassessment — Iteration 2

## Product decision

This workbench is a disposable local tool for one operator, not a collaborative
editor or production persistence service. The operator rejected multi-writer
ordering machinery as disproportionate. This revision protects accidental
repeated Save activation, safe temporary-file creation, stale browser
completion, and capability-aware launch instructions. Concurrent external API
writers remain unsupported.

The previous design stopped because Tokio `io-util` changed the lockfile.
`tempfile 3.27.0` is already a dev dependency and locked. Promoting it to a
runtime dependency supplies unique same-directory temporary files and
persist-on-rename without custom naming or a new locked package.

## Finding disposition

| Finding | Lineage | Revised disposition |
| --- | --- | --- |
| WR-01 | `incomplete_fix` | Close the supported single-operator path with browser single-flight Save, stale-publication suppression, and `NamedTempFile` replacement. Accept unspecified last-writer behavior for unsupported concurrent external PUTs. |
| WR-06 | `sibling_path_miss` from CR-01 | Document the exact emitted capability URL and capability-preserving image deep links. |

## Shared invariants

1. Browser Save admits one request at a time; a repeated click dispatches no
   second PUT.
2. Save captures one adjustment snapshot. If edits/navigation make it stale,
   completion cannot overwrite or claim to represent the current draft.
3. Draft invalidation does not abort Save persistence. Pending ends only after
   the response/body or transport error settles.
4. Fixture writes use a uniquely created temporary file in the workspace,
   write and flush the complete JSON document, then persist it to the stable
   destination. The destination is never truncated in place.
5. Concurrent external PUT ordering is outside the supported contract. Unique
   temporary files prevent shared-temp collisions, but no winner guarantee is
   provided.
6. Operators use the exact process capability URL and retain `cap` in image
   deep links.

## Assumption decisions

### Accepted

| ID | Assumption | Consequence |
| --- | --- | --- |
| A-01 | There is one human operator and one browser workflow. | Solve accidental duplicate activation, not collaborative editing. |
| A-02 | Fixtures are tiny and infrequent. | Synchronous `std::io::Write` through `NamedTempFile` is acceptable. |
| A-03 | `NamedTempFile::new_in(workspace)` owns temp naming and cleanup. | No custom suffix, retry, or cleanup protocol. |
| A-04 | `NamedTempFile::persist(destination)` is the publication boundary. | Flush first, surface errors, and drop the returned temp on persist failure. |
| A-05 | Save does not use `ownedApi` abort authority. | Await a dedicated persistence request; publish only for the current owner. |
| A-06 | Concurrent direct API callers are unsupported. | No ordinal, barrier, mutex-order contract, or deterministic winner test. |
| A-07 | The emitted capability is sensitive. | Retain it in deep links and never record a live token publicly. |

### Rejected

| ID | Rejected assumption | Reason |
| --- | --- | --- |
| R-01 | This tool needs production multi-writer ordering. | It has one operator and no collaborative-editing requirement. |
| R-02 | A shared `<fixture>.json.tmp` is acceptable. | Accidental overlap can collide; `tempfile` solves this. |
| R-03 | Fetch abort rolls back server persistence. | It does not; Save stays pending independently of draft ownership. |
| R-04 | Hand-built temp naming is preferable. | The existing library already owns collision safety and cleanup. |
| R-05 | The bare loopback URL is usable. | Private APIs correctly reject it without the capability. |

## Productization blocker

Before this workbench or its fixture API can support multiple users, multiple
browser sessions, background writers, or production persistence, the product
must define and implement a multi-writer contract. At minimum that requires:

- durable record versions or ETags and explicit conflict behavior;
- authorization and actor/audit identity;
- serialized or compare-and-swap commits with a defined winner;
- conflict/retry UX that never silently overwrites another editor; and
- deterministic concurrent-write, recovery, and durability tests.

Until those requirements are designed and verified, multi-user productization
is blocked; the single-operator limitation must remain explicit.

## Failure matrix

| Path | Required result |
| --- | --- |
| Double-click Save | One persistence call; pending until settlement. |
| Edit/navigate during Save | Snapshot may persist; no stale UI publication. |
| Save again after settlement | New explicit snapshot persists. |
| Temp create/write/flush/persist error | Error surfaced; temp cleaned by RAII; no in-place destination truncation. |
| Concurrent external PUTs | Independent temp files; final ordering explicitly unsupported. |
| Emitted/deep-link URL | API works when `cap` is retained. |
| Bare/missing-cap URL | Private API remains `403`. |

## Resume criteria

Independent review must accept the single-operator limitation, confirm the
productization blocker, verify bounded dependency promotion and the browser test
seam, and cover capability deep links. Only then may status become
`ready_to_resume`.

