---
quick_id: 260823-e2j
status: complete
completed: 2026-08-23
---

# Candidate Runtime Secret-Mount Removal

## Outcome

Closed PR #217's remaining controller credential-boundary gap. The documented
candidate-controller production gate no longer copies or mounts the legacy
runtime secrets directory and now states that OCI media access uses the VM
instance principal.

## Changes

- Removed the candidate secrets variable, copy, cleanup, and Podman mount from
  `docs/static-runtime-runbook.md`.
- Added a focused runtime-contract regression covering the managed controller
  quadlet, rendered application environment, legacy-key cleanup task, and
  candidate-gate documentation.
- Posted the actionable HIGH finding to PR #217 before applying the fix.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes`
  — 7 passed.
- `python3 scripts/validate_repo_hygiene.py`
- `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ANSIBLE_LOCAL_TEMP=/tmp/ansible-tmp ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml`
- `git diff --check`

## Commit

- `cd521aa` — Harden instance-principal runtime boundary
