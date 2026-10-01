---
phase: 08-admin-media-review-and-operational-posture
pr: 261
finding: WR-05
status: ready_to_resume
trigger_review: 08-PR-261-REVIEW-ITER3.md
assumption_revision_evidence: 08-PR-261-WR-05-REVISED-PLAN.md#contract-decision
implementation_plan_review_evidence: 08-PR-261-WR-05-PLAN-REVIEW-ITER2.md
---

# PR 261 Review/Fix Convergence Reassessment

## Trigger

Round three retained Warning `WR-05`: runtime preflight and migration state detection prove that all supported edit-event literals are present, but do not prove that unsupported event types are rejected. Per the repository convergence guard, no further point-fix work may begin until the shared assumption and implementation plan are revised and reviewed.

## Finding Lineage

| Finding | Classification | Current state |
|---|---|---|
| CR-01 | Independent round-one blocker | Core event admission fixed; WR-03 and then WR-05 exposed incomplete validation assumptions. |
| CR-02 | Independent round-one blocker | Resolved across all seven item-returning mutation paths. |
| WR-01 | Independent round-one test weakness | Oracle schema/live coverage added; WR-04 cleanup weakness resolved. |
| WR-02 | CR-01 fix regression | Interruption-safe staged constraint replacement implemented. |
| WR-03 | Incomplete CR-01 fix | All nine supported values checked, but exact rejection contract remained unproven. |
| WR-04 | WR-01 test weakness | Explicit verified cleanup now governs success; Drop reports stable recovery evidence. |
| WR-05 | Incomplete WR-03 fix | Open: literal-presence checks accept widened predicates such as `OR 1=1`. |

## Shared Invariant

The enabled Oracle edit-event constraint must admit every value in `EditEventKind::ALL` and reject every unsupported value. Runtime readiness and migration convergence must establish both halves of this exact allowlist because history decoding rejects unknown values.

## Consumer Inventory

- Producers: all nine `EditEventKind` variants and every mutation calling `insert_edit_event`.
- Persistence boundary: fresh schema DDL, Phase 8 migration canonical/temporary constraint states, and Oracle preflight.
- Consumers: history decoding through `EditEventKind::from_str`, pending-change queries, publish-event snapshots, and operator history responses.
- Mutation boundaries: create, update, attach, set-primary, remove, replace, and image-adjustment update.
- Tests: schema source contracts, runtime preflight condition tests, migration state fixtures, credentialed live Oracle/OCI smoke, and cleanup verification.

## Assumption Audit

- Rejected assumption: presence of every quoted supported literal is equivalent to an exact allowlist.
- Revised assumption: Oracle constraint text is untrusted schema input and must match a deliberately restricted predicate grammar whose parsed literal set equals `EditEventKind::ALL` exactly.
- Migration-specific revision: a script-generated constraint may be recognized only by strict normalized equality with the script's canonical predicate. Manually formatted but semantically valid predicates may be conservatively rebuilt; false rejection is safe, false acceptance is not.
- Credential boundary: non-live tests can prove parser/state-contract behavior, but credentialed execution remains separate evidence and must be reported honestly.

## Failure Matrix

| Constraint state | Required result |
|---|---|
| Exact nine-value allowlist | Runtime accepts; migration converges or leaves an equivalent script-generated canonical constraint. |
| Missing supported literal | Runtime rejects; migration repairs while retaining an enabled constraint. |
| Extra literal | Runtime rejects; migration repairs. |
| All nine plus `OR 1=1` or another trailing predicate | Runtime rejects; migration must not classify it as current. |
| Widened temporary constraint | Migration must not rename it as canonical. |
| Formatting/case/outer-parenthesis variation within the restricted exact grammar | Runtime parser may accept; migration may conservatively rebuild. |

## Revised Plan Requirements

See `08-PR-261-WR-05-REVISED-PLAN.md`. The initial plan review identified unsafe whole-predicate lowercasing in `08-PR-261-WR-05-PLAN-REVIEW.md`; the plan was revised to preserve string-literal bytes and independently approved in `08-PR-261-WR-05-PLAN-REVIEW-ITER2.md`. Both required evidence references are recorded in frontmatter, so the reassessment is ready for one coherent coder resumption.

## Resume Criteria

1. The revised assumption is recorded in this artifact and the revised plan.
2. An independent reviewer approves the revised plan's exact grammar, migration recognizer, state transitions, and tests.
3. The reviewer evidence path and revised-plan evidence path are recorded in frontmatter.
4. Only then may status change from `reassessment_required` to `ready_to_resume` and one coherent WR-05 contract change be assigned to a coder.
5. The next source review must preserve WR-05 lineage and must be clean before the loop closes.
