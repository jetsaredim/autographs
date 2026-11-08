# Terraform and Ansible boolean cleanup plan

Status: proposed for review during Phase 8, before the Cloudflare DNS apply or Phase 8 closeout.

The Phase 8 posture pass recorded Terraform create/enable toggles as
P8-POSTURE-006 but deferred their removal. It did not inventory Ansible mode
booleans. This plan closes that review gap without treating every boolean as
unnecessary. Changes to the production Terraform root require a saved plan
against its current OCI state before merge or apply.

## Decisions to review

| Configuration | Proposed disposition | Reason and required evidence |
|---------------|----------------------|------------------------------|
| `autonomous_database_is_free_tier` | Remove as an input; set `is_free_tier = true` in the data-services module. | Always Free is a project constraint. Confirm the production plan changes no ADB attribute or resource. |
| `autonomous_database_is_mtls_connection_required` | Remove as an input; set `is_mtls_connection_required = true` in the data-services module. | Wallet mTLS is the production connection contract. Confirm the production plan changes no ADB attribute or resource, and update CI/deploy variable wiring and runbooks. |
| `create_network`, `create_runtime_instance`, `create_autonomous_database`, `create_media_bucket` | Inspect state and bootstrap consumers before deciding which to remove. Do not simply delete a `count` expression. | These inputs control stateful OCI resource addresses and first-run readiness. Removing `count` can change `[0]` addresses and may require `moved` blocks; a false value can instead plan destruction. Record each current production value, state address, import path, and effect of both values. |
| `assign_public_ip` | Keep unless the public runtime topology is made fixed. | This changes the runtime VNIC contract rather than selecting a migration stage. Confirm all consumers before choosing a constant. |
| `autographs_deploy_explicit_cutover` | Remove only if full deploy remains an explicit cutover and the handler path is retired coherently. | Full deploy asserts `true`; handlers are suppressed when true. Check every notify/handler path, freeze preflight, unfreeze, rollback, and deploy tests before removing it. |
| `autographs_deploy_publish_frozen` | Keep as an operation state until the freeze/unfreeze contract is redesigned. | Full deploy requires true; `publish-unfreeze.yml` deliberately sets false. Both states are used. |
| `security_patching_reboot_validate_dnf_noop` | Keep until a replacement test seam is designed. | The reboot safety probe uses it, and validation fixtures intentionally exercise both values. |

## Review and execution order

1. Capture the current production Terraform state addresses and a saved plan
   with all current GitHub variables. Record whether each candidate change is
   no-op, update, replacement, or destroy. Never publish secret plan values.
2. Review the two ADB policy constants first. Implement them in a focused IaC
   PR, update GitHub variable documentation, and require a production-state
   plan showing no ADB update, replacement, or destroy before merge.
3. Review the four create flags one resource at a time. If removing a flag,
   preserve the resource's state address with an explicit `moved` block when
   necessary, verify bootstrap/import guidance, and require a no-destroy plan.
   Keep any flag whose false state remains a supported operator workflow.
4. Review Ansible cutover booleans separately from Terraform state changes.
   Exercise freeze, full deploy, unfreeze, rollback, and security reboot
   validation tests. Remove only an unreachable or redundant switch, together
   with its consumers and documentation.
5. Post the plan, review findings, no-destroy evidence, and clean review
   confirmation on the implementation PR. Update P8-POSTURE-006 with the final
   disposition for each variable before Phase 8 closeout.

## Acceptance criteria

- No production Terraform plan proposes unintended OCI or Cloudflare resource
  creation, replacement, or destruction from the cleanup.
- Every removed input has no remaining workflow, example, module, documentation,
  or Ansible consumer.
- Terraform `fmt` and `validate`, CI plan, Ansible syntax/lint, and the relevant
  deploy, unfreeze, and security patching tests pass.
- The implementation PR records the reviewed state addresses and a sanitized
  plan summary before merge. A syntax-only validation is insufficient for
  stateful flag removal.
