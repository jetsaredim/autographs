# IaC operational review: open items

Status: owner disposition pending for P8-POSTURE-006 and Plan 08-08 Task 5.
This is the active review queue. The [full evidence register](iac-operational-review-findings.md)
preserves the source inventory, resolved decisions, production plans, tests,
and PR audit trail. In the Terraform and Ansible surveys and switch census,
look for **Dispositioned** to see the decisions already made, and
**Proposed · owner review pending** to find the entries awaiting acceptance.
The evidence HTML search can find either label.

## Decisions for owner review

| Item | Proposed disposition | What to decide | Evidence |
|---|---|---|---|
| Deploy API signing key | Keep optional Terraform registration of an externally generated public key. | Confirm the bootstrap procedure and whether to investigate the current production key's enrollment. The other Terraform resource, lookup, module, and provider rows have dispositions. | [Signing key](iac-operational-review-findings.md#deploy-api-signing-key) and [Terraform inventory](iac-operational-review-findings.md#terraform-resources-lookups-modules-and-providers). |
| Ansible and workflow source inventory | Accept the remaining proposed dispositions or name an entry that needs work. | Start with the Ansible rows marked **Proposed · owner review pending**, then review remaining workflow and action entries. The freeze/unfreeze and patching safety flows are accepted at the operating-flow level. | [Ansible playbooks](iac-operational-review-findings.md#ansible-playbooks), [roles and handlers](iac-operational-review-findings.md#ansible-roles-task-files-and-handlers), [workflows and actions](iac-operational-review-findings.md#github-workflows-and-local-actions). |

## Evidence to finish after disposition

| Item | Current state | Next step |
|---|---|---|
| Three Ansible fixture playbooks | `publish-freeze-deploy-test.yml`, `publish-freeze-render-test.yml`, and `security-report-render-test.yml` have syntax/lint evidence in this register but no recorded behavior run. The production-shaped Python fixture did run the real preflight, role reaudit, and both unfreeze modes against Caddy on 2026-11-22. | Run the three fixtures where practical, or record why the existing related tests and live proofs suffice. [Per-entry evidence](iac-operational-review-findings.md#ansible-playbooks). |
| Final Phase 8 closeout | `AGENTS.md` and `SECURITY.md` exist; private vulnerability reporting is enabled. | Recheck current-state guidance and the reporting link, then write `08-08-SUMMARY.md` and update Phase 8 state/requirement status. [Checkpoint](../.planning/phases/08-admin-media-review-and-operational-posture/08-08-CHECKPOINT.md). |

