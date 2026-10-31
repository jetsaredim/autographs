---
quick_id: 261030-lyp
status: complete
implementation_commit: 20b77cc
---

# Quick Task 261030-lyp Summary

Restored `.github/workflows/ci.yml`'s Ansible job to syntax checking and linting. Removed the automatic functional validation step, including its Docker check, publish freeze/deploy integration run, and security/runtime scenario runs. Renamed the visible job to `Ansible syntax and lint` and removed unnecessary `-vvv` syntax output. All 27 syntax targets, full `ansible-lint deploy/ansible/` scope, functional test playbooks, fixtures, and the existing `pull_request` and `workflow_dispatch` triggers remain.

## Verification

- Parsed the workflow and confirmed the Ansible job has exactly checkout, tooling installation, collection installation, syntax check, and lint steps.
- Ran the syntax command extracted from the workflow: passed for all 27 playbooks. The initial sandbox attempt could not write under `~/.ansible/tmp`; the required rerun outside the sandbox passed.
- Ran `ansible-lint deploy/ansible/`: passed with zero failures and warnings across 84 files.
- `git diff --check`: passed.
- `actionlint` was not installed locally; the PR's existing Workflow checks job will run it.

The deeper playbooks remain runnable manually. This change does not assess whether each deep test should be retained, shortened, or removed from the repository.
