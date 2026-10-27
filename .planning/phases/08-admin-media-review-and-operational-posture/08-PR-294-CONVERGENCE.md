---
phase: 08-admin-media-review-and-operational-posture
pr: 294
trigger_review: 08-07-PR-294-REVIEW.md
review_round: 4
status: ready_to_resume
assumption_revision_evidence: 08-PR-294-ASSUMPTIONS.md
implementation_plan_evidence: 08-PR-294-CONTRACT-REVISION.md
implementation_plan_review_evidence: 08-PR-294-CONTRACT-PLAN-REVIEW.md#round-4-first-cutover-revision--verification-passed
next_review_round: 5
---

# PR 294 convergence reassessment

Round 1's old-URL lifetime and cached-dimension findings were fixed. Round 2 found `CR-02`, a candidate visibility regression from the old-URL fix, and `WR-02`, a missed emergency-takedown consumer in release pruning. Their lineage, shared invariants, consumer inventory, assumption audit, and failure matrix are recorded in `08-07-PR-294-REVIEW.md#convergence-reassessment` and expanded in the revised assumptions artifact above.

The revised plan makes `current` promotion the public media commit boundary, adds durable exact-path tombstones for emergency denial, tests the boundary with a controlled publisher callback and real Caddy HTTP fixture, and specifies a publish freeze and readiness checks for the non-atomic Caddy/controller rollout. The independent plan review first found one blocker and two warnings, then recorded their resolution in its `Revision 2 — VERIFICATION PASSED` section. The evidence paths in frontmatter were checked before this status changed to `ready_to_resume`.

Coder work may resume against the revised plan. Review round 3 is the mandatory convergence checkpoint: if any in-scope Critical or Warning finding remains, stop point fixes and reassess assumptions and implementation plan again before further source changes.

## Round 3 mandatory checkpoint and renewed resume evidence

Review round 3 found `CR-03`, a deployment consumer of the candidate visibility invariant. The production GitHub Actions deployment omitted `autographs_deploy_publish_frozen`, so the Ansible role's false default removed an existing gate while the old publisher could still run. The finding and complete lineage, consumer inventory, assumption audit, and four-row failure matrix are recorded in `08-07-PR-294-REVIEW.md#round-3-convergence-reassessment` and in the revised `08-PR-294-ASSUMPTIONS.md`.

The revised `08-PR-294-CONTRACT-REVISION.md` section 4 specifies the actual workflow invocation, a verified old-controller process-exit barrier before new Caddy routing, quarantine of shared files lacking committed provenance, a frozen deploy through both restarts and failure paths, and a separate readiness-backed unfreeze. The independent review first found the unknown queued-request gap, then approved the process-exit and provenance revision in `08-PR-294-CONTRACT-PLAN-REVIEW.md#round-3-reassessment-revision--verification-passed`. These three evidence paths were reviewed before restoring `ready_to_resume` above. Coder work may resume only against this coherent deployment contract; the implementation and live rollout remain unverified.

## Round 4 renewed reassessment and resume evidence

Round 4 found `CR-04` (a later-deploy consumer of the committed URL lifetime invariant) and `WR-03` (a deployment-audit consumer of the emergency marker invariant). The review report records the lineage, complete consumer inventory, assumption audit, failure matrix, and resume criteria in `08-07-PR-294-REVIEW.md#round-4-convergence-reassessment`.

The revised `08-PR-294-ASSUMPTIONS.md` distinguishes first-cutover ambiguity from later committed history and permits an intentionally absent active file under a validated exact marker. `08-PR-294-CONTRACT-REVISION.md` section 5 defines recoverable pointer intent/history, a non-mutating initial inventory with a conservative retention wait when pruned-release evidence is unavailable, history-backed later audits, marker-aware validation, and rollback/pruning consumers. The independent plan review first found an unproven pruned-release path, then approved the revised first-cutover stop and wait in `08-PR-294-CONTRACT-PLAN-REVIEW.md#round-4-first-cutover-revision--verification-passed`. Those three evidence paths were checked before `ready_to_resume` was restored. Coder work may now follow section 5 as one publication-history contract change; deployment and CDN proof remain separate Wave 8 gates.
