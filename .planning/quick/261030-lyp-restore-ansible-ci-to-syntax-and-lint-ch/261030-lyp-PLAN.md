---
quick_id: 261030-lyp
description: Restore Ansible CI to syntax and lint checks for issue 298
status: planned
must_haves:
  truths:
    - Ordinary and Release PRs run Ansible syntax checking and linting.
    - The CI Ansible job no longer executes the functional publish or security validation playbooks.
    - Functional test playbooks and fixtures remain available for manual runs.
  artifacts:
    - .github/workflows/ci.yml
    - scripts/test_security_patching_create_issue_tasks.py
  key_links:
    - The existing pull_request and workflow_dispatch triggers continue to run the Ansible job.
    - All existing syntax targets and the full ansible-lint scope remain in CI.
---

# Quick Task 261030-lyp Plan

## Task 1: Restore the fast Ansible CI job

Rename the job to describe its syntax/lint scope. Remove the `Test Ansible validation playbooks` step, including its Docker check and functional publish/security invocations. Keep the existing checkout, Ansible installation, collection installation, syntax target list, and `ansible-lint deploy/ansible/`. Remove verbose syntax output that serves no validation purpose.

## Task 2: Verify the workflow

Run `actionlint` if available, inspect the job's step and command inventory, and check the diff. Update the repository automation test that expected the removed functional invocation so it checks the retained syntax coverage. Confirm the functional playbooks remain in the repository and manual dispatch still triggers the fast CI job. Open a separate ready-for-review PR for this implementation.
