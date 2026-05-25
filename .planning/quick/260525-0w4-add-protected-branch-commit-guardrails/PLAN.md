---
status: complete
date: 2026-05-25
slug: add-protected-branch-commit-guardrails
---

# Add Protected Branch Commit Guardrails

## Objective

Prevent future GSD or agent-driven commits from landing directly on `main` or `master`.

## Scope

- Add project-facing branch guardrails to `AGENTS.md`.
- Configure this project to use phase and quick-task branches.
- Patch global GSD workflow/reference/commit guardrails so commit-capable workflows stop on protected branches.

## Verification

- Confirm current work is on a non-protected branch before committing.
- Confirm project config no longer uses branch strategy `none`.
- Confirm `git diff --check` passes.
