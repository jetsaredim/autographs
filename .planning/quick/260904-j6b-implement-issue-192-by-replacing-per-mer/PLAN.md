# Release-Please Production Release Model

This is the orchestration index for quick task `260904-j6b`. Execute the three plans in order; each plan is independently verifiable and the later plans depend on contracts created by the earlier plans.

## Outcome

Ordinary merges accumulate in a release-please Release PR without tagging or deploying. Merging that validated Release PR creates a draft semantic GitHub Release and drives build/deployment in the same workflow run. Production uses semantic controller tags with digest verification, infrastructure-only releases explicitly reuse the active controller, retry reconciles partial draft state, and rollback changes only the controller while using current-main automation.

## Execution Order

| Wave | Plan | Objective |
|------|------|-----------|
| 1 | [01-PLAN.md](01-PLAN.md) | Establish release-please, release state, manifest, and reconciliation contracts with tests. |
| 2 | [02-PLAN.md](02-PLAN.md) | Replace per-merge deployment with release-gated automatic/retry/controller-rollback workflow paths and fixture-backed workflow tests. |
| 3 | [03-PLAN.md](03-PLAN.md) | Protect active/rollback images during cleanup and document release, retry, rollback, and retention operations. |

## Goal-Backward Must-Haves

- A normal merge can only run release preflight and create/update the ready Release PR; it cannot tag or enter production.
- The Release PR is created with `RELEASE_PLEASE_TOKEN`, so it receives normal PR CI, and the v5.0.0 action is pinned to `45996ed1f6d02564a971a2fa1b5860e934307cf7`.
- Any release-please failure stops; an unresolved draft stops later releases until manual retry reconciles it.
- Automatic releases classify the complete previous-tag-to-target-tag range, build a controller tag only for controller impact, and deploy by semantic tag after verifying its digest.
- Infrastructure-only releases reuse the active controller tag/digest without applying the new repository tag to old image bytes.
- Finalization is strictly deployment/health or repo-only validation, idempotent status commit, then Release publication; conflicting manifest assets fail closed.
- Retry can safely resume from any completed manifest/deployment/status/publish prefix.
- Rollback reads a published manifest but uses current-main workflow/Ansible code, performs no historical Terraform apply, changes only the controller mapping, and preserves deployed repository status.
- Local and remote cleanup protect the active and previous controller mappings; remote deletion is manual and inventory-first.
- The complete initial implementation is pushed to a ready PR before review agents run, and findings/fixes/clean confirmation remain on that PR.

## Decision Coverage

| Decisions | Plans | Coverage |
|-----------|-------|----------|
| D-01-D-03, D-15-D-16, D-18-D-21 | 01, 02 | Release PR lifecycle, token, pinned action, same-run gate, fail-closed preflight/action behavior |
| D-04-D-08, D-11-D-12, D-17, D-22, D-24 | 01, 02 | Range classification, tag/digest manifest, status, finalization, cumulative/fixture-backed validation |
| D-09-D-10, D-23 | 01, 02, 03 | Retry, production serialization, current-source controller-only rollback, operator procedure |
| D-13-D-14 | 03 | Active/rollback retention and explicit remote pruning |

No persisted `RESEARCH.md` exists for this quick task. The plans incorporate the official release-please manifest/action contracts and GitHub token-trigger behavior already researched during discussion. No context or issue requirement is unplanned.

## Delivery Sequence

1. Execute and commit Plans 01-03 on the task branch.
2. Run the complete local changed-surface verification from Plan 03.
3. Push the initial implementation and open a normal ready-for-review PR closing issue #192.
4. Wait for cumulative PR CI before starting the review/coder cycle.
5. Post every actionable review finding to the PR, respond to line-specific threads directly after fixes, and post the final clean confirmation to the PR. Do not add repository review-tracking documents.
6. After merge, verify the implementation merge only updates the Release PR. Merge the Release PR only after its CI succeeds, then validate the expected infrastructure-only first release and run an incremental publish.
