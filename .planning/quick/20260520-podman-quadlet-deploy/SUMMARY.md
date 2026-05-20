---
type: quick
status: complete
completed: 2026-05-20
slug: podman-quadlet-deploy
---

# Quick Task Summary: Podman Quadlet Deploy

Moved the production runtime deployment from compose/podman-compose to Ansible-managed Podman quadlets.

## Accomplishments

- Replaced the compose runtime artifact with systemd quadlet templates for the dedicated Podman network, Next.js app container, and Caddy container.
- Moved VM host state into the Ansible role: runtime packages, service masking, swap, firewalld, protected env/secrets/wallet paths, image pulls, and service enablement.
- Removed Terraform cloud-init user data and the bootstrap script from the compute module so instance configuration is no longer baked into ignored metadata.
- Added a manual `recreate_runtime_instance` deploy workflow input that taints the runtime instance before `terraform apply`.
- Converted GHCR cleanup from Node to Python and moved it into a shared scheduled/manual image cleanup workflow.
- Added runtime image cleanup that preserves the active image, protected tags, `latest`, and newest retained app/tools images on the VM.
- Updated deployment/config docs and smoke helper scripts for the quadlet runtime shape.

## Validation

- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/data-smoke.yml deploy/ansible/playbooks/system-cleanup.yml` -> passed
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/` -> passed
- `python3 -m py_compile scripts/cleanup-ghcr-images.py` -> passed
- `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff` -> passed
- `git diff --check` -> passed

## Validation Note

`terraform validate` was not completed locally. A clean `TF_DATA_DIR=/tmp/autographs-tfdata terraform -chdir=infra/terraform init -backend=false` could download providers with network escalation, but `terraform validate` failed to load provider schemas in this sandboxed execution context.
