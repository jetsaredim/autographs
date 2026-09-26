---
phase: quick-260920-gz7
generated: 2026-09-25T06:00:00-04:00
trigger_iteration: 7
status: ready_to_resume
current_review: .planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW.md
prior_reviews:
  - .planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW-FIX.md
revised_plan_evidence: .planning/quick/260920-gz7-enforce-a-uek-only-production-kernel-pos/260920-gz7-REVIEW-CONVERGENCE.md
plan_review_evidence: https://github.com/jetsaredim/autographs/pull/253#issuecomment-5830481837
---

# PR 253 Review Convergence Checkpoint

## Trigger Signals

- Seven review/fix rounds occurred without a durable clean-review confirmation.
- The latest head passed all checks except `Validate Ansible`; the failure exposed a cross-path state-contract omission rather than an isolated assertion defect.
- The same omitted facts affect both update and reboot result reconstruction and can make an inventory-only unsafe host disappear from remaining-host selection.

## Finding Lineage

1. The initial implementation enforced UEK-only production posture and classified installed RHCK packages as configuration drift.
2. Later review rounds hardened bootability checks from filename heuristics to bounded package ownership and exact boot-entry probes.
3. Commit `fce6775` made kernel inventory completeness authoritative for classification and report routing.
4. Direct scanner and report fixtures were updated, but the update/reboot snapshot and reconstruction consumers retained the earlier, smaller posture subset.
5. The isolated post-update fixture therefore defaulted the missing completeness fact to unsafe, misreported a safe RHCK-drift host as kernel recovery, and omitted the mixed-group deployment warning.

The surviving defect is one contract gap propagated across duplicated consumers, not a new independent issue.

## Shared Invariants

- Every result path must fail closed when kernel inventory, running-kernel, default-kernel, ownership, or exact boot-entry evidence is incomplete.
- A host remains actionable while it has OpenSCAP findings, installed RHCK packages, incomplete kernel inventory, or an invalid running/default UEK posture.
- Update and reboot result rendering must receive the same kernel posture evidence used by direct scanner classification.
- RHCK configuration drift must remain distinguishable from unsafe kernel recovery so the issue gives the correct operator action.
- Mixed target groups containing any `investigate` host must block deployment convergence for the entire group.
- Missing snapshot fields must be treated as an incomplete result contract, not silently replaced with values that can close the issue.

## Consumer Inventory

| Stage | Producer/consumer | Required correction |
|---|---|---|
| Scanner producer | `roles/security_patching/tasks/kernel_posture.yml` | Remains the authoritative source of inventory, bounded probe, boot-entry, and derived UEK-valid facts. |
| Update snapshot | Both snapshot blocks in `roles/security_patching/tasks/patch.yml` | Preserve every authoritative posture fact required for selection and reporting. |
| Update result | `roles/security_patching/tasks/post_result.yml` | Validate the complete snapshot, retain inventory-only unsafe hosts, and reconstruct the full scanner-facing posture. |
| Reboot preflight snapshot | `roles/security_patching/tasks/validate_reboot_state.yml` | Preserve the same posture contract when reconciliation prevents reboot mutation. |
| Reboot post-mutation snapshot | `playbooks/security-reboot.yml` | Preserve the same posture contract after reboot. |
| Reboot result | `roles/security_patching/tasks/post_reboot_result.yml` | Apply the same completeness, retention, and reconstruction rules. |
| Report | `templates/security-report.md.j2` | Consume reconstructed facts without path-dependent behavior. |
| Result comments | `templates/security-update-result.md.j2`, `templates/security-reboot-result.md.j2` | Surface inventory completeness beside the derived UEK booleans so an investigation result is actionable. |
| Validation | Post-update refresh/status, reboot-result, and reconciliation validation playbooks | Exercise safe drift, inventory-only failure, missing-field refusal, unsafe boot posture, mixed-host precedence, and diagnostic preservation. |

## Canonical Snapshot Contract

Each `security_patching_post_update_*` and `security_patching_post_reboot_*` snapshot must carry the following scanner facts under the corresponding prefix, in addition to the existing finding/advisory/action fields:

- `installed_rhck_packages`
- `kernel_inventory_is_complete`
- `kernel_inventory_probe_facts`
- `running_kernel_release`
- `running_kernel_image`
- `default_kernel_image`
- `running_kernel_probe_facts`
- `default_kernel_probe_facts`
- `running_kernel_owner_probe_facts`
- `default_kernel_owner_probe_facts`
- `running_kernel_boot_entry`
- `default_kernel_boot_entry`
- `running_kernel_is_valid_uek`
- `default_kernel_is_valid_uek`

Result completeness gates must require every listed field plus `entries`, `advisory_ids`, `next_action`, and a `complete` scan status. Result reconstruction must mirror every listed field back to its unprefixed scanner name before rendering `security-report.md.j2`. No required field may be supplied through a permissive default at the completeness boundary.

The completeness boundary must also validate the shape of structured evidence, not merely that its top-level variable is defined:

- Each of `kernel_inventory_probe_facts`, `running_kernel_probe_facts`, `default_kernel_probe_facts`, `running_kernel_owner_probe_facts`, and `default_kernel_owner_probe_facts` must be a mapping containing `rc`, `stdout`, and `stderr`.
- Each of `running_kernel_boot_entry` and `default_kernel_boot_entry` must be a mapping containing `valid`, `expected_image`, `entry_kernel_images`, `expected_initramfs`, `initramfs_images`, and `probe`; `probe` must itself be a mapping containing `rc`, `stdout`, and `stderr`.
- A defined but malformed or partial structure is an incomplete snapshot and must stop result publication exactly like an absent field.

## Assumption Audit

- **Rejected:** live facts remaining in one Ansible process are sufficient. Isolated result roles and future orchestration boundaries depend on the declared snapshot contract.
- **Rejected:** adding a fixture default is safe. It hides the production false-close risk.
- **Rejected:** the two derived UEK booleans are a complete posture contract. Classification and recovery reporting also depend on inventory completeness and bounded diagnostic evidence.
- **Retained:** explicit prefixed facts are preferable for this repair. Introducing a new nested posture object now would require simultaneously migrating templates, fixtures, and hostvars lookups and would widen an already review-heavy PR. The repair will use one documented fact list mirrored symmetrically across update and reboot paths.
- **Retained:** absence of any required snapshot fact is a hard result-contract failure. This is safer than defaulting missing evidence.

## Failure Matrix

| State | Host retained? | Action/report behavior |
|---|---:|---|
| Complete inventory, valid running/default UEK, no RHCK, no findings | No | Issue can close. |
| Complete inventory, valid running/default UEK, RHCK installed | Yes | `configure`; report RHCK configuration drift. |
| Incomplete inventory, otherwise apparently valid UEK | Yes | `investigate`; report kernel recovery with inventory diagnostics. |
| Complete inventory, invalid running or default UEK/boot entry | Yes | `investigate`; report kernel recovery with exact probe/boot-entry diagnostics. |
| OpenSCAP findings remain, kernel posture safe | Yes | Preserve patch/reboot classification derived from findings. |
| Configure host mixed with any investigate host | Yes, both | Aggregate `investigate`; prohibit group deployment convergence. |
| Required post-update/post-reboot snapshot field absent | Stop result workflow | Name incomplete hosts; do not render or close from partial evidence. |

## Revised Plan Requirements

1. Extend both post-update snapshot sites, the reboot preflight snapshot in `validate_reboot_state.yml`, and the post-mutation snapshot in `security-reboot.yml` with every field in the canonical contract.
2. Extend update/reboot result completeness gates to require those fields.
3. Add inventory incompleteness to update/reboot remaining-host predicates.
4. Mirror the complete snapshots back to the scanner-facing fact names before rendering the shared report.
5. Update both result-comment templates to display kernel inventory completeness along with RHCK count and derived UEK validity.
6. Update isolated fixtures to provide the declared contract. Add inventory-only unsafe regression cases for both result paths plus absent-field and defined-but-malformed structured-evidence refusal cases in the post-result status and reboot-result validations.
7. Run the previously failing playbook, `security-post-result-status-validate-test.yml`, the reboot-result validation, the complete Ansible CI suite, syntax/lint validation, repository runtime validation, and existing Rust checks.
8. Push the fix, record the diagnosis and verification on PR #253, then run an independent reviewer against the exact pushed head. Every finding or clean result must be posted to the PR.

## Resume Criteria

Coder work may resume only after an independent plan review confirms that the revised plan covers all snapshot/reconstruction consumers and the failure matrix. At that point this artifact must be updated to `ready_to_resume` with durable revised-plan and plan-review evidence.
