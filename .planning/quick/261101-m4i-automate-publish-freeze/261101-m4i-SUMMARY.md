---
quick: 261101-m4i
status: complete
implementation_commit: 6fddb5c
---

# Automatic production publish freeze

- The preflight now backs up the live Caddyfile, inserts the publish denial routes in place, validates and reloads Caddy, and proves both routes return `503` with `no-store` while the public manifest remains available before changing Caddy service ordering or stopping the controller.
- The installer accepts an already frozen retry and rejects partial routes or an unknown proxy shape. The existing release audit and readiness-backed unfreeze remain in force.
- The real Caddy/Ansible fixture starts unfrozen and verifies the installed gate, unchanged Caddyfile inode, backup, retry paths, and unfreeze. Focused partial-route tests also pass.
- The `v0.5.0` tag still contains the earlier preflight and requires its documented one-time manual gate before retry. Later release tags containing this commit use the automatic preflight.

## Verification

- `ansible-playbook --syntax-check deploy/ansible/playbooks/publish-freeze-preflight.yml`
- `ansible-lint deploy/ansible/` — 0 failures, 0 warnings across 85 files.
- `python3 -m unittest deploy.ansible.tests.test_publish_freeze_deploy.PublishFreezeDeploymentTests.test_in_place_gate_is_idempotent_and_rejects_partial_routes`
- `python3 -m unittest deploy.ansible.tests.test_publish_freeze_deploy.PublishFreezeDeploymentTests.test_actual_production_workflow_requires_preflight_and_explicit_unfreeze deploy.ansible.tests.test_publish_freeze_deploy.PublishFreezeDeploymentTests.test_deploy_and_rollback_source_order_guard`
- `python3 -m unittest deploy.ansible.tests.test_publish_freeze_deploy.PublishFreezeDeploymentTests.test_actual_unfreeze_all_marked_against_real_caddy` — passed.
- `git diff --check`
