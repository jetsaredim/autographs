# IaC operational review: open items

Status: owner disposition pending for P8-POSTURE-006 and Plan 08-08 Task 5.
This is the active review queue. The [full evidence register](iac-operational-review-findings.md)
preserves the source inventory, resolved decisions, production plans, tests,
and PR audit trail. Resolved items are outside this review queue.

## Decisions for owner review

| Item | Proposed disposition | What to decide | Evidence |
|---|---|---|---|
| Local image cleanup dry-run | Retain the manual preview, which defaults to `true`. Scheduled local Podman cleanup uses `false` and can prune selected old images; GHCR deletion has a separate explicit operation. | Accept this preview-versus-scheduled-cleanup contract, or name the behavior to change. | [Producer-to-action trace](iac-operational-review-findings.md#autographs-system-cleanup-dry-run) and [cleanup workflow/role entries](iac-operational-review-findings.md#github-workflows-and-local-actions). |
| Terraform source inventory | Accept the linked resource, data lookup, module, provider, state, and variable dispositions as documented, or name an address/input that needs a different disposition. | Review the proposed retain/remove decisions and named limitations. The reviewed IAC findings above are closed. | [Terraform inventory](iac-operational-review-findings.md#terraform-resources-lookups-modules-and-providers), [switch census](iac-operational-review-findings.md#switch-census-and-direct-boundaries), and [production state/plan evidence](iac-operational-review-findings.md#production-state-and-plan-evidence). |
| Ansible and workflow source inventory | Accept the linked playbook, role, handler, workflow, and local-action dispositions, or name an entry that needs work. | Review proposed retain/remove decisions and any named proof limitations. The freeze/unfreeze and patching safety flows are accepted. | [Ansible playbooks](iac-operational-review-findings.md#ansible-playbooks), [roles and handlers](iac-operational-review-findings.md#ansible-roles-task-files-and-handlers), [workflows and actions](iac-operational-review-findings.md#github-workflows-and-local-actions). |

## Evidence to finish after disposition

| Item | Current state | Next step |
|---|---|---|
| Three Ansible fixture playbooks | `publish-freeze-deploy-test.yml`, `publish-freeze-render-test.yml`, and `security-report-render-test.yml` have syntax/lint evidence in this register but no recorded behavior run. The production-shaped Python fixture did run the real preflight, role reaudit, and both unfreeze modes against Caddy on 2026-11-22. | Run the three fixtures where practical, or record why the existing related tests and live proofs suffice. [Per-entry evidence](iac-operational-review-findings.md#ansible-playbooks). |
| Ordinary production release | [PR #375](https://github.com/jetsaredim/autographs/pull/375) contains the closeout changes; [Release PR #370](https://github.com/jetsaredim/autographs/pull/370) remains open. | After PR #375 review/merge, review the refreshed Release PR plan, merge through the normal release path, and record saved-plan apply, deploy health, and publication result. [Execution paths](iac-operational-review-findings.md#production-execution-paths-traced-so-far). |
| Final Phase 8 closeout | `AGENTS.md` and `SECURITY.md` exist; private vulnerability reporting is enabled. | Recheck current-state guidance and the reporting link, then write `08-08-SUMMARY.md` and update Phase 8 state/requirement status. [Checkpoint](../.planning/phases/08-admin-media-review-and-operational-posture/08-08-CHECKPOINT.md). |

