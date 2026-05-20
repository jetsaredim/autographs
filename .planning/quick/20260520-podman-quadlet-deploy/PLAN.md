---
type: quick
status: complete
created: 2026-05-20
slug: podman-quadlet-deploy
---

# Quick Task: Podman Quadlet Deploy

Move the production runtime deployment from compose/podman-compose to systemd-managed Podman quadlets while keeping both the Next.js app and Caddy as containers on a dedicated Podman network.

## Scope

- Replace Ansible compose deployment with quadlet templates.
- Keep Caddy containerized and managed by Podman/systemd.
- Keep a dedicated Podman network for app-to-Caddy traffic.
- Move host setup currently baked into bootstrap bash into idempotent Ansible tasks.
- Update deployment docs and config language away from compose/podman-compose for the current production path.
- Move GHCR image cleanup to a scheduled/manual Python workflow.
- Add a post-health-check deploy cleanup for unused local Podman images.

## Verification

- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/data-smoke.yml deploy/ansible/playbooks/system-cleanup.yml`
- `python3 -m py_compile scripts/cleanup-ghcr-images.py`
- `git diff --check`
