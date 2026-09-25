---
status: diagnosed
trigger: PR #253 Validate Ansible fails after the seventh review-fix round in the mixed configure/investigate post-result refresh fixture
created: 2026-09-25
updated: 2026-09-25T05:45:49-04:00
---

## Symptoms

- Expected behavior: `security-post-result-refresh-validate-test.yml` renders aggregate `investigate` guidance for a target group containing one UEK configuration-drift host and one unsafe-kernel host, offers no approval label, and passes the complete Ansible CI sequence.
- Actual behavior: the refreshed report omits `do not run deployment convergence for this target group`, so the mixed-host assertion fails even though aggregate action remains `investigate`.
- Error messages: GitHub Actions run 35729944810, job 106752727120 fails at `deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml:316` with `evaluated_to: false` for the expected target-group deployment-convergence warning.
- Timeline: introduced or exposed by the seventh review-fix round at PR head `173a385`; all other PR checks pass.
- Reproduction: run `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml` on branch `gsd/quick-enforce-a-uek-only-production-kernel-pos`.

## Current Focus

reasoning_checkpoint:
  hypothesis: "Confirmed: scanner hardening expanded the authoritative kernel-posture contract, but update/reboot snapshots and result reconstruction still carry the older subset, so the report defaults missing inventory completeness to unsafe and loses bounded recovery diagnostics."
  confirming_evidence:
    - "Commit fce6775 added security_patching_kernel_inventory_is_complete to classification, issue host selection, and the report recovery predicate, and updated direct render/classification fixtures, but did not update patch.yml, post_result.yml, security-reboot.yml, post_reboot_result.yml, or the post-result fixture."
    - "The unmodified validation playbook reproduces the line-316 failure while still proving aggregate actions are configure+investigate, aggregate action is investigate, and no approval label is offered."
    - "The rendered report classifies configure-host as Kernel recovery required with rpm/grubby/boot-entry diagnostics all unknown; therefore it never enters the RHCK configuration-drift branch containing the missing target-group warning."
    - "Running the same unmodified playbook with --extra-vars security_patching_kernel_inventory_is_complete=true passes all assertions and renders configure-host as UEK-only configuration drift while investigate-host remains recovery."
  falsification_test: "Supplying only security_patching_kernel_inventory_is_complete=true would not restore the warning if aggregate precedence, RHCK inventory, or template wording were the cause; instead the entire playbook passed, confirming the missing fact controls the failed branch."
  fix_rationale: "Define one explicit scanner posture snapshot and reconstruct it symmetrically in update and reboot result paths; completeness gates and remaining-host selection must use that same contract. Do not add only a fixture default."
  blind_spots: "Production update/reboot playbooks retain live scanner facts within one ansible-playbook process, which can mask reconstruction omissions; isolated result fixtures expose the declared snapshot contract. REVIEW-CONVERGENCE.md must decide whether to preserve individual facts or introduce a structured posture object before coder work."
hypothesis: confirmed — post-result kernel posture reconstruction is incomplete after scanner bootability hardening
test: reproduced unchanged failure, inspected the rendered artifact, traced producer/snapshot/reconstruction/template consumers, and reran with only the missing inventory fact supplied
expecting: confirmed — the safe RHCK-drift host renders configuration drift, the unsafe host renders recovery, aggregate investigate blocks group convergence, and the fixture passes when inventory completeness is present
next_action: create and review REVIEW-CONVERGENCE.md with the complete posture invariant and update/reboot consumer matrix before any coder work

## Evidence

- 2026-09-25: PR #253 head is `173a385`; eight CI jobs pass and Validate Ansible alone fails.
- 2026-09-25: PR #253 has no unresolved review threads, but the attempted final reviewer failed on usage limits and never produced a clean verdict.
- 2026-09-25: the repository has exceeded the global third-review convergence threshold, so another point fix is prohibited until finding lineage, shared invariants, consumers, assumptions, failure matrix, and revised-plan evidence are recorded.
- timestamp: 2026-09-25T05:31:00-04:00
  observation: `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ANSIBLE_LOCAL_TEMP=/tmp/autographs-ansible-local ANSIBLE_REMOTE_TEMP=/tmp/autographs-ansible-remote ansible-playbook deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml` reproduces exactly one failed assertion at line 316 after 86 successful tasks and three expected rescued fake-GitHub failures.
  implication: the failure is deterministic in local CI-equivalent execution and occurs after report rendering, not in target selection, action aggregation, or fake-endpoint rescue.
- timestamp: 2026-09-25T05:33:00-04:00
  observation: the failed run's `/tmp/autographs-security-post-result-mixed-kernel.md` has `next_action: investigate`, lists both hosts, and shows correct RHCK/UEK summary values, but renders both hosts under `Kernel recovery required`; `configure-host` has both UEK flags true while all newly added inventory/probe/boot-entry diagnostics are `unknown`.
  implication: aggregate investigate precedence and RHCK discovery are intact; the report's missing inventory fact alone prevents `configure-host` from reaching the configuration-drift branch.
- timestamp: 2026-09-25T05:37:00-04:00
  observation: rerunning the unchanged playbook with `--extra-vars security_patching_kernel_inventory_is_complete=true` passes all 87 tasks and all assertions. The report then renders `configure-host` under `UEK-only configuration drift` with the exact target-group convergence warning while `investigate-host` remains under recovery.
  implication: this controlled intervention confirms the report branch predicate, not the aggregate classifier or asserted wording, is causal.
- timestamp: 2026-09-25T05:40:00-04:00
  observation: `kernel_posture.yml` produces inventory completeness, bounded inventory/running/default probe facts, exact running/default boot-entry facts, and the derived UEK-valid booleans. `classify_findings.yml`, `create_issue.yml`, and `security-report.md.j2` consume the expanded contract directly.
  implication: fce6775 made inventory completeness authoritative for investigation and reporting, while 0612068 made exact boot entries authoritative for UEK validity and recovery diagnostics.
- timestamp: 2026-09-25T05:42:00-04:00
  observation: both preservation branches in `patch.yml` snapshot RHCK packages, kernel paths, two UEK-valid booleans, scan status, and action, but do not snapshot inventory completeness or any bounded probe/boot-entry facts. `post_result.yml` neither requires nor selects on inventory completeness and reconstructs the same older subset. No `security_patching_post_update_*` variables exist for the omitted posture facts.
  implication: the update result contract is asymmetric. Besides the visible false-recovery rendering, an inventory-only `investigate` state can be omitted from `security_patching_remaining_hosts` and incorrectly drive issue closure because selection does not include the authoritative inventory predicate.
- timestamp: 2026-09-25T05:44:00-04:00
  observation: `security-reboot.yml` and `post_reboot_result.yml` duplicate the same older posture subset and feed the same `security-report.md.j2`; no `security_patching_post_reboot_*` variables exist for inventory completeness or the new diagnostics.
  implication: the root cause is a shared update/reboot snapshot-reconstruction contract gap, although the current CI symptom is the isolated update-result fixture.

## Eliminated

- hypothesis: aggregate action precedence regressed and selected `configure` or offered an approval label
  evidence: the failing fixture passes assertions for remaining actions `configure` and `investigate`, aggregate action `investigate`, empty action/report labels, and absence of both approval labels.
- hypothesis: the expected warning was deleted or textually changed
  evidence: the exact phrase remains in `security-report.md.j2` inside the RHCK configuration-drift branch for aggregate actions other than `configure`.
- hypothesis: the fake GitHub endpoint prevents the refreshed report from being available
  evidence: rendering, stat, size validation, file read, result-comment rendering, and authoritative request construction all finish before the intentional PATCH failure; the rescue exposes the complete rendered body.
- hypothesis: RHCK configuration drift was not preserved for `configure-host`
  evidence: the failed report summary shows one RHCK drift package for `configure-host`, and the recovered run renders that package as `kernel-core` in the configuration-drift section.

## Consumer Inventory

| Stage | Files / consumers | Required posture contract | Confirmed state |
|---|---|---|---|
| Producer | `roles/security_patching/tasks/kernel_posture.yml` | RHCK inventory; inventory completeness and bounded probe result; running/default release and image; bounded running/default probes; exact running/default boot-entry results; derived running/default bootable-UEK flags | Complete producer after `0612068` and `fce6775` |
| Direct classification | `roles/security_patching/tasks/classify_findings.yml` | scan status, inventory completeness, both UEK-valid flags, RHCK packages, findings/DNF evidence; probe facts for reason text | Complete and fail-closed |
| Initial issue selection/render | `roles/security_patching/tasks/create_issue.yml`, `templates/security-report.md.j2` | inventory completeness participates in host inclusion and recovery routing; probe and boot-entry facts supply recovery diagnostics | Complete on the direct scanner path |
| Update snapshot | both preservation blocks in `roles/security_patching/tasks/patch.yml` | must preserve every fact needed to select, classify, and render the authoritative post-update state | Incomplete: no inventory completeness and no inventory/running/default probe or boot-entry snapshots |
| Update result selection and reconstruction | `roles/security_patching/tasks/post_result.yml` | completeness gate, remaining-host predicate, action aggregation, and scanner-fact mirror must share the snapshot contract | Incomplete: gate/selector/mirror use the pre-hardening subset; action aggregation itself is correct |
| Update report/comment | `templates/security-report.md.j2`, `templates/security-update-result.md.j2` | issue report needs the full reconstructed posture; result comment currently shows only RHCK count and UEK booleans | Issue report defaults missing inventory to unsafe and missing diagnostics to unknown; comment does not expose inventory completeness |
| Reboot snapshot/result | `playbooks/security-reboot.yml`, `roles/security_patching/tasks/post_reboot_result.yml`, `templates/security-report.md.j2` | symmetric post-reboot posture preservation, selection, and reconstruction | Same incomplete subset and latent report/false-close risk as update path |
| Validation consumers | `security-report-render-test.yml`, `security-finding-classification-validate-test.yml`, `security-kernel-posture-validate-test.yml`, `security-post-result-refresh-validate-test.yml`, `security-reboot-result-validate-test.yml`, `security-update-reconciliation-validate-test.yml` | direct fixtures and isolated result fixtures must model the same declared posture snapshot | Direct fixtures were updated by `fce6775`; isolated update/reboot result and reconciliation coverage were not expanded |

## Resolution

- root_cause: Commit `fce6775` expanded scanner classification and `security-report.md.j2` to require `security_patching_kernel_inventory_is_complete` (with new bounded probe/boot-entry diagnostics) but left update and reboot snapshot, completeness, remaining-host, and reconstruction paths on the older kernel-posture subset; the isolated post-result fixture therefore defaults the safe RHCK host's missing inventory fact to false, renders recovery instead of configuration drift, and omits the asserted mixed-group warning.
- fix: not applied — diagnose-only. Before coder work, create and review `REVIEW-CONVERGENCE.md`; the correction must make update and reboot posture snapshots/reconstruction symmetric with the scanner contract, include inventory-only unsafe states in remaining-host selection, preserve bounded diagnostics needed by the report, and update all isolated result/reconciliation fixtures together.
- verification: unchanged playbook reproduced the line-316 failure; supplying only `security_patching_kernel_inventory_is_complete=true` made the complete three-play fixture pass and restored the exact warning. Full verification after a future fix must cover safe RHCK drift, inventory-only probe failure, unsafe running/default/boot-entry states, mixed-host precedence, update result refresh, and reboot result refresh.
- files_changed: `.planning/debug/pr253-ansible-ci-regression.md` only; no source or test files changed.
