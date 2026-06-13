---
gsd_task_version: 1.0
task: remove-obsolete-tenancy-split-doc
status: complete
completed_at: "2026-06-13T01:55:20Z"
---

# Quick Task Summary: Remove Obsolete Tenancy Split Doc

## Completed

- Removed the historical Terraform tenancy split migration runbook from active docs.
- Removed the deployment runbook link that told operators to run the already-completed migration.
- Replaced the Terraform state doc's split-migration pointer with current two-root state guidance.
- Updated the Rust controller walkthrough so remaining Phase 5 work no longer lists completed 05-06 deployment wiring.

## Verification

- `rg -n "terraform-tenancy-split|Tenancy Split Migration" .` found no active filename or title references.
- `git diff --check` passed.
