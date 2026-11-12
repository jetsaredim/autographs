# Terraform and Ansible operational review plan

Status: proposed for Phase 8 execution before 08-08 closeout.

The Phase 8 posture pass found avoidable configuration switches, but reviewing
only booleans would miss larger problems in the deployment contract. Review the
Terraform and Ansible implementation as one operator workflow before selecting
changes. The output is a findings register with evidence and a small set of
scoped implementation PRs. A switch stays only when its current value is proven
by a supported operation, safety invariant, or test; its name alone is not a
reason to retain it.

## Current surface map

The source inventory starts with these ownership boundaries. This map identifies
review targets; it does not assign final dispositions or claim that production
state matches the repository defaults.

| Boundary | Current implementation | Review focus |
| --- | --- | --- |
| OCI tenancy | `infra/terraform/tenancy/` and its IAM and state-bucket modules | Separate bootstrap permissions, state bucket ownership, runtime secret policies, and operator access. |
| Production infrastructure | Main `infra/terraform/` root with network, compute, and data-services modules | Resource addresses, conditional creation, Vault readiness and rotation, image selection, outputs, and dependencies. |
| DNS and edge | Porkbun DNS record, Cloudflare zone, records, TLS setting, and cache rules in the main root | Ownership during and after cutover, proxy persistence, and ordinary deploy mutation guard. |
| Ordinary delivery | `.github/workflows/deploy.yml`, `deploy/ansible/playbooks/deploy.yml`, freeze preflight, unfreeze, and rollback | Saved plan/apply, image replacement, freeze lifecycle, health checks, retry, and failure reporting. |
| Maintenance | Security scan, patch, reboot, image cleanup, and system cleanup workflows and playbooks | Approval, drift, package and kernel checks, cleanup, and reboot recovery. |
| Validation and operator contract | CI, Ansible fixtures, configuration examples, and runbooks | Actual supported states, test gaps, stale variables, and commands an operator can follow. |

Initial code inspection shows the four Terraform `create_*` inputs alter resource
addresses or lookups, and `create_autonomous_database` also gates Vault data and
secret rotation. The Cloudflare mode controls zone and record resources. The
deploy workflow applies Terraform, then runs freeze preflight, deploy, unfreeze,
and rollback playbooks. These are coupled paths that need a shared review. The
production state addresses, actual values, and plan effects remain to be
captured before choosing changes.

## Inventory and trace

Inventory the entire `infra/terraform/` root and modules, `deploy/ansible/`
playbooks and roles, their GitHub Actions callers, configuration examples,
runbooks, and validation fixtures. For each resource, module, provider, data
lookup, playbook, role, handler, and operator entry point, record:

| Evidence | Questions to answer |
| --- | --- |
| Purpose and owner | What production outcome does this element provide, and which workflow owns it? |
| Inputs and state | What are its default and actual production values, state address, secret source, and supported alternatives? |
| Producers and consumers | Which workflow, module, task, output, handler, test, or runbook sets or reads it? |
| Execution path | What happens on initial provisioning, ordinary deploy, Cloudflare DNS apply, freeze/unfreeze, rollback, security scan, patch, and reboot? |
| Failure and recovery | What happens on partial apply, drift, retry, interrupted deployment, and operator rollback? |
| Validation | Which test or saved production plan proves the claimed behavior? What remains untested? |
| Disposition | Keep, simplify, remove, consolidate, or investigate, with a reason and the effect of the change. |

Trace each Boolean and mode value through every direct producer, consumer,
mutation boundary, reporting/action path, and relevant test. Include values
used only in fixtures: decide whether they represent a supported operational
state, a necessary test seam, or dead configuration. Record both true and false
behavior where each is reachable. Do not classify a value as a safety control
without identifying the unsafe action it prevents and a test that proves it.

## Review dimensions

1. **Terraform ownership and state:** Check resources and data lookups against
   the live state and intended end state. Review `count`/`for_each` addressing,
   imports, moved/removed blocks, provider boundaries, lifecycle rules,
   dependency edges, outputs, and drift behavior. A cleanup must not silently
   recreate, replace, destroy, or stop managing a production resource.
2. **Deployment flow:** Follow the saved-plan/apply boundary, controller image
   selection, publish freeze, service replacement, health checks, unfreeze, and
   rollback. Look for duplicate checks, unreachable branches, hidden coupling,
   stale variable names, and ways a partial failure leaves the system frozen or
   reports success incorrectly.
3. **Ansible behavior:** Review role and playbook entry points, defaults,
   overrides, task conditions, handlers, idempotence, check-mode behavior,
   privilege use, and failure cleanup. Verify that a task has one clear owner
   and that facts passed across playbooks have the same meaning.
4. **Security and operations:** Trace OCI/Cloudflare credentials, IAM scope,
   private values, DNS mutation gates, security patch approval/drift checks,
   and the operator commands that recover from failed runs. Check whether
   production and test paths enforce the same invariants.
5. **Validation and documentation:** Identify contracts tested only by source
   string checks, missing failure cases, stale examples, and claims that differ
   from current code. Prefer behavior tests and a sanitized production plan for
   stateful changes.

## Known switches to investigate

These are inventory seeds, not predetermined keep/remove decisions:

| Surface | Values | Question |
| --- | --- | --- |
| OCI resource existence | `create_network`, `create_runtime_instance`, `create_autonomous_database`, `create_media_bucket` | Are false values supported in the current single-site production workflow? What happens to state addresses, Vault readiness, password rotation, and imports if a switch is removed? |
| OCI policy/topology | `autonomous_database_is_free_tier`, `autonomous_database_is_mtls_connection_required`, `assign_public_ip` | Are these actual operator choices or fixed project policy? Would fixing a value change the production plan? |
| Cloudflare | `cloudflare_mode` and DNS apply controls | Does each mode represent a supported transition, and can an ordinary deploy change DNS or proxy status? |
| Deploy cutover | `autographs_deploy_explicit_cutover`, `autographs_deploy_publish_frozen`, `autographs_deploy_allow_unfreeze` | Which states are reached during deploy, rollback, and retry? Could one explicit operation state replace coupled flags without weakening the freeze gate? |
| Security patching | `security_patching_reboot_validate_dnf_noop` and related role conditions | Is this an essential runtime guard, a fixture seam, or redundant with another proof? |

Review other switches discovered by the inventory with the same standard. A
retained switch needs its supported values, owner, failure behavior, and proof
recorded. A removed switch needs every caller, example, and test updated.

## Execution order and gates

1. Produce the full inventory and findings register before selecting code
   changes. Rank findings by operational risk and value, not by how easy a
   variable is to delete. Review the inventory with the owner.
2. Group related contract changes into focused PRs. Review one shared
   invariant across Terraform, Ansible, workflow callers, tests, and runbooks
   before editing siblings. Keep the publish performance fix in a separate
   Wave 8 PR; it must also be complete before closeout.
3. For Terraform changes, capture production state addresses and a sanitized
   saved plan using the current GitHub variables. Require no unapproved data
   lookup change, creation, update, replacement, or destruction. Pay special
   attention to ADB password retrieval/readiness/rotation and the persistent
   Cloudflare proxy mode. A syntax-only validation is insufficient.
4. For Ansible changes, exercise freeze, full deploy, unfreeze, rollback,
   security scan/patch/reboot, and failure cleanup, including the paths that
   currently set opposite values for the same flag. Run syntax/lint and the
   relevant behavior fixtures.
5. Record findings, proposed changes, plan evidence, fixes, and clean review
   on each PR. Update the Phase 8 posture register with a disposition for every
   finding. Complete the review and approved fixes before 08-08 closeout.

Keep behavior with a demonstrated operational use. Simplify or remove behavior
whose value cannot be shown by the inventory and tests.
