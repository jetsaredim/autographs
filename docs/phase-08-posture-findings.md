# Phase 8 Posture Findings Register

This register records the pre-media repository posture pass for Phase 8. It keeps the cleanup scope separate from the upcoming CDN and admin media implementation work, and it gives downstream executors concrete evidence that operational posture was reviewed before image preview or adjustment files changed.

| Finding ID | Surface | Finding | Severity | Disposition | Fix Evidence | Verification |
|------------|---------|---------|----------|-------------|--------------|--------------|
| P8-POSTURE-001 | Source organization | Current source layout still matches the Rust/static split, but future media work needs an explicit pre-media boundary so cleanup does not drift into image-editor implementation files. | Medium | Tracked in Phase 8 plan | This register records the boundary before Plan 08-03 begins. | `git diff --name-only` must not include media implementation files during this plan. |
| P8-POSTURE-002 | Docs/runbooks | Operator docs and current-state references need to keep the retired Next.js runtime framed as historical evidence, not active runtime architecture. | High | Guarded by CI | Plan 08-02 adds a repository hygiene scan for denied retired-runtime current-state claims. | `python3 scripts/validate_repo_hygiene.py` |
| P8-POSTURE-003 | Workflows | Pull request validation needs a lightweight guardrail that fails stale codebase-map/current-state claims before they merge. | High | Guarded by CI | The CI workflow-checks job will run the repository hygiene validator. | `.github/workflows/ci.yml` contains `Validate repository hygiene`. |
| P8-POSTURE-004 | Deployment/process scripts | Production security patching is a live-VM maintenance surface and should remain reviewed with deploy/runtime changes rather than ordinary docs cleanup. | High | Fixed | Codebase maps now describe Phase 8 as production security patching repair plus operational posture work. | Map grep for `production security patching repair`. |
| P8-POSTURE-005 | Configuration names | Some service/config names are intentionally verbose or legacy-shaped after the static runtime and admin rename work. | Low | Accepted with rationale | The naming cleanup is tracked as a future refinement because changing deployed variable names can create operational churn. | `.planning/STATE.md` pending todo remains the follow-up owner. |
| P8-POSTURE-006 | Terraform create/enable variables | Terraform still carries create/enable-style variables for resources that are effectively end-state managed in production. | Medium | Accepted with rationale | The variables are retained until a dedicated IaC cleanup can account for import/state behavior without surprising operators. | Phase 8 state keeps the Terraform boolean review as a future refinement. |
| P8-POSTURE-007 | Stale codebase maps | Codebase maps can misroute future executors if they describe Phase 8 as taxonomy media cues or AI-assisted ingest. | High | Fixed | Architecture, concerns, stack, testing, and AGENTS current-state sections now point Phase 8 at admin media review and operational posture. | `rg -n "Phase 8.*admin media|Phase 8.*operational posture|production security patching repair" .planning/codebase/*.md AGENTS.md` |
| P8-POSTURE-008 | Validation gaps | Existing CI validates runtime, workflows, Terraform, Ansible, and scripts, but did not yet validate stale current-state claims. | Medium | Guarded by CI | Plan 08-02 adds standard-library Python tests and the repository hygiene validator. | `python3 -m unittest scripts/test_validate_repo_hygiene.py` |
| P8-POSTURE-009 | Public edge/cache hygiene | CDN/cache behavior is Phase 8 work, but enablement must wait until adjusted-media cache behavior can be verified after the media adjustment pipeline exists. | High | Tracked in Phase 8 plan | The posture pass records CDN/cache as a pre-media contract and post-media verification concern, not a Task 1 implementation. | Phase 8 downstream plans retain CDN/cache contract and post-media verification gates. |

## IaC boolean reassessment during Wave 8

The pre-media pass recorded P8-POSTURE-006 but did not remove the Terraform
create/enable variables or audit Ansible mode booleans. The Cloudflare
implementation initially repeated that pattern with separate enabled and proxy
flags. Wave 8 replaced those flags with one validated `cloudflare_mode` value:
`off`, `dns_only`, or `proxied`.

| Surface | Current switches | Assessment |
|---------|------------------|------------|
| Runtime Terraform resource existence | `create_network`, `create_runtime_instance`, `create_autonomous_database`, `create_media_bucket` | These control whether stateful OCI resources exist. `create_autonomous_database` also gates Vault data lookups, readiness checks, and password rotation. Removing them requires a reviewed production state plan and full consumer audit for unintended updates as well as replacement or destruction. P8-POSTURE-006 remains a deferred IaC cleanup. |
| Runtime Terraform configuration | `assign_public_ip`, Autonomous Database Free tier and mTLS fields | Public IP changes topology; Free tier and mTLS are currently true in production and should be reviewed as fixed policy rather than casually toggleable options. Preserve current values until a production plan proves a refactor has no replacement. |
| Ansible deployment | `autographs_deploy_publish_frozen`, `autographs_deploy_allow_unfreeze`, `autographs_deploy_explicit_cutover` | Full deploy asserts frozen and explicit cutover before replacing runtime services. The unfreeze playbook sets the explicit override to permit freeze route removal; explicit cutover suppresses restart handlers. Review the freeze/unfreeze pair and cutover contract before simplifying them. |
| Ansible security patching | `security_patching_reboot_validate_dnf_noop` | This controls the reboot follow-up safety probe and is overridden in validation fixtures. Keep it until the safety behavior has an equivalent test seam. |

This reassessment distinguishes migration-state combinations from safety and
resource-existence controls. It does not mark the broader Terraform/Ansible
boolean cleanup complete. The full Terraform/Ansible inventory, review, and
change gates are in [the IaC operational review plan](iac-operational-review-plan.md).

## Pre-media evidence

Plan 08-02 changed only posture documentation, planning/codebase maps, AGENTS current-state guidance, CI workflow wiring, and the repository hygiene scripts/tests. It did not modify `controller/static-admin/`, `controller/src/image_adjustments.rs`, `controller/src/derivatives.rs`, `controller/src/publisher.rs`, or image-adjustment migration files.

Verification commands for this pre-media boundary:

- `python3 -m unittest scripts/test_validate_repo_hygiene.py`
- `python3 scripts/validate_repo_hygiene.py`
- `git diff --name-only 97a8fdd..HEAD`

## Pre-media PR evidence

The pre-media PR landed before downstream CDN or admin media work began.

- PR: [#207](https://github.com/jetsaredim/autographs/pull/207)
- Merge commit: `5b0d8ebb69f5342b84ce75dfae8eb56ade6523e5`
- Merged at: `2026-08-17T01:57:29Z`
- PR head: `03354ee9931e897cd368552d9a58d28182ed6ceb`
- CI: green on PR #207 before merge.
- Review evidence: automated review findings were posted to PR #207, fixed in `03354ee`, and followed by a clean review confirmation comment on the PR.

Plan 08-03 is unblocked only after this pre-media PR evidence is recorded in this register and in `.planning/phases/08-admin-media-review-and-operational-posture/08-02-SUMMARY.md`.
