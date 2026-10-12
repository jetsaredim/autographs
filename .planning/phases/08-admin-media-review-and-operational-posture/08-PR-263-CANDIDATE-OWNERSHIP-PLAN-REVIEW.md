---
phase: 08-admin-media-review-and-operational-posture
pr: 263
reviewed_commit: e114fb3
verdict: changes_required
blockers: 1
warnings: 0
info: 0
---

# PR 263 Candidate Ownership Plan Review

## ISSUES FOUND

**Plan:** `08-PR-263-CANDIDATE-OWNERSHIP-REVISED-PLAN.md`
**Reviewed commit:** `e114fb3`
**Verdict:** **CHANGES REQUIRED — 1 Blocker, 0 Warning, 0 Info**

## Blocker

### 1. Local-only candidates must remain recoverable after the final `Drop` discovery failure

**Dimension:** Key links / test fidelity

**Required property:** Every locally known candidate remains actionable until
its object is verified absent, including when the last inventory attempt occurs
inside `Drop`.

**Evidence:** The plan makes `refresh_inventory` transactional and tests
persistent failure while the state object still exists. However, the current
`Drop` path returns immediately when inventory fails and then destroys the
guard. A retained local-only A candidate therefore disappears at guard
destruction. The emitted item/step evidence is intentionally fingerprint-only
and cannot reconstruct A when it has no Oracle image or cleanup-event row—the
exact motivating assumption recorded in `08-REVIEW.md`.

**Source boundary:** `controller/tests/live_persistence_smoke.rs:1419-1428`

**Why this remains open:** An implementation can remove `mem::take`, pass the
proposed helper tests, and still leave `Drop` as “failed refresh → report →
return.” That closes ownership during the call but not through the guard's
terminal lifecycle. A subsequent item-ID cleanup can discover B, remove B and
the Oracle rows, and still orphan local-only A.

**Required plan change:** Define and test a terminal ownership mechanism. On a
final discovery failure, either delete and verify all retained local candidates
while preserving Oracle rows, or durably preserve their private identities
before `Drop` destroys the guard. Raw identities must remain out of output.

## Coverage audit

| Boundary | Assessment |
| --- | --- |
| Initial A registration | Covered by transactional pre-call candidate ownership |
| Post-replacement refresh | Covered for failure followed by successful retry |
| Partial query/row/decode failure | Covered; temporary union is not installed |
| Normal `cleanup_and_verify` | Covered; union remains installed through delete and verification failures |
| Fallback `Drop` with successful retry | Covered; A/B/event union precedes row deletion |
| Fallback `Drop` with repeated discovery failure | **Not covered; guard destruction loses local-only identities** |
| Manual cleanup | Existing private environment input and Oracle rows remain available; no new privacy regression planned |
| Destructive SQL ordering | Covered; remains after all candidate absence proofs |
| Duplicate/already-absent candidates | Covered through deduplication and exact 404 behavior |
| CR-01 recovery surface | Remains closed; plan does not modify its files or authority contract |
| WR-01 evidence privacy | Remains closed; raw identities and provider text stay out of output |
| Verification commands | Runnable and targeted, but cannot compensate for the missing terminal-lifecycle assertion |

## Recommendation

Revise the plan to define and test ownership at guard destruction, then
re-review. Do not change convergence status to `ready_to_resume` yet.

No source files were reviewed as changed, and no repository state was modified
by the independent plan reviewer.

---

## ISSUES FOUND — Second Review

**Plan:** `08-PR-263-CANDIDATE-OWNERSHIP-REVISED-PLAN.md`
**Reviewed commit:** `bd6deac`
**Verdict:** **CHANGES REQUIRED — 1 Blocker, 0 Warning, 0 Info**

Register-before-create resolves terminal ownership if its durable row remains
private and intact. The selected row status introduces a separate destructive
consumer, however.

### Blocker: Durable smoke authority must not become an actionable application cleanup warning

**Dimension:** Cross-path contract

**Required property:** A recovery-authority row protecting an active smoke
object must remain private, non-actionable metadata until cleanup verifies the
object absent and removes the item rows.

**Evidence:** The revised plan uses `deleteFailed`. The Oracle catalog currently
loads every `deleteFailed` event as an admin cleanup warning. The admin retry
route selects such a warning by image ID, deletes its target, removes active
image metadata for any operation other than `replace`, and marks the row
`retrySucceeded`. That contradicts the plan's requirement to keep the authority
row intact until final cleanup.

**Source evidence:**

- `controller/src/oracle_catalog.rs`: `load_cleanup_warnings` selects every row
  where `status = 'deleteFailed'`.
- `controller/src/routes.rs`: `retry_image_cleanup` acts on the selected warning,
  deletes its target, conditionally removes metadata, and resolves the row.
- `controller/db/schema.sql`: no distinct non-actionable authority state exists.

**Required plan change:** Define a non-actionable authority representation or
exclude the smoke authority operation from all warning/retry consumers. Add
the required catalog/route scope and executable tests proving the row remains
discoverable by item-ID cleanup but absent from admin warnings and unreachable
through cleanup retry.

### Second-review coverage audit

| Area | Result |
| --- | --- |
| Register before initial A creation | Covered; commit failure prevents object creation |
| Final `Drop` destruction | Covered if the durable row remains intact |
| Fresh item-ID cleanup | Covered: A comes from the authority row; B comes from current-image or cleanup-event metadata |
| Atomic in-memory refresh | Covered by the shared production transition and failure matrix |
| Delete/verification failure | Candidate identities and Oracle rows remain |
| Warning/retry reachability | **Not covered; `deleteFailed` makes the authority row actionable** |
| Privacy | Covered; raw keys remain private data and output remains fingerprint-only |
| CR-01 and WR-01 | Remain closed |

Do not change convergence status to `ready_to_resume` yet.

---

# PR 263 Candidate Ownership Plan Review — Iteration 3

**Reviewed plan:** `08-PR-263-CANDIDATE-OWNERSHIP-REVISED-PLAN.md`
**Reviewed commit:** `d60ce68`
**Verdict:** **APPROVED — 0 Blocker, 0 Warning, 0 Info**

## Verification Passed

The revised plan closes both previous review blockers:

1. Register-before-create gives local A durable private ownership before its
   object can exist, so final guard destruction cannot erase its only
   actionable identity.
2. Reserved operation `livePersistenceSmokeGuard` is excluded from ordinary
   cleanup-warning and retry consumers while remaining visible to private
   item-ID inventory.

The plan is implementation-ready for one bounded coder pass after this review
artifact and its PR comment are recorded in the convergence ledger.

## Terminal ownership and representation

- The disposable item is committed before its authority event, satisfying the
  existing item foreign key.
- A's exact target key is inserted and committed before `media.write`;
  registration or commit failure prevents object creation.
- The authority row survives discovery, deletion, and verification failures
  and is removed by cascade only after all candidates are verified absent.
- Repeated inventory failure followed by guard destruction leaves A
  discoverable through its durable event. Committed B remains discoverable
  through current-image or cleanup-event metadata.
- `livePersistenceSmokeGuard` fits the existing operation column and
  `deleteFailed` remains valid under the status constraint; operation plus
  status, not status alone, controls admin actionability.

## Action unreachability and regression safety

- One shared predicate permits only `delete` and `replace`.
- Memory and Oracle warning loaders apply it, so item/status responses omit
  smoke authority.
- The retry route validates it independently before media deletion, metadata
  mutation, or event resolution.
- An injected repository bypasses loader filtering to execute the route's
  defense. Ordinary `delete` and `replace` warnings retain positive regression
  coverage and their current metadata-removal distinction.
- Unknown or reserved operations fail closed before side effects.

## Discovery, testing, and privacy

- Item-ID cleanup intentionally reads every cleanup-event target without an
  operation/status filter, so private recovery remains broader than admin
  actionability.
- Candidate inventory is atomic; partial discovery cannot install a partial
  union; every absence proof precedes destructive SQL.
- Register-before-create, registration failure, repeated failure plus guard
  destruction, fresh cleanup, both warning loaders, injected-route defense,
  and allowed-operation regressions have executable coverage.
- Oracle source/query contracts supplement rather than replace executable
  evidence.
- Raw keys remain private inputs. Browser and recovery evidence remains
  fingerprint/item/step based, preserving WR-01. The plan does not touch the
  static-admin recovery surface, preserving CR-01.

## Convergence compliance

CR-02 lineage remains `incomplete_fix`; both prior plan-review findings are
incorporated; the consumer/failure matrices are executable. A subsequent deep
source review must still return zero actionable findings before convergence
closes.

No files, commits, pushes, or PR comments were created by the independent plan
reviewer.
