---
status: complete
date: 2026-05-25
slug: add-protected-branch-commit-guardrails
key-files:
  modified:
    - AGENTS.md
    - .planning/config.json
  external:
    - /home/jgreenwa/.codex/get-shit-done/references/git-integration.md
    - /home/jgreenwa/.codex/get-shit-done/references/planning-config.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/quick.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/execute-phase.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/plan-phase.md
    - /home/jgreenwa/.npm/_npx/4db0de1f85c3165e/node_modules/get-shit-done-cc/sdk/dist/query/commit.js
---

# Summary

Added branch guardrails so agent and GSD work should not commit directly to `main` or `master`.

Project config now uses phase branches and quick-task branches:

- `git.branching_strategy`: `phase`
- `git.quick_branch_template`: `gsd/quick-{slug}`

The repo instructions now explicitly require creating or switching to a work branch before committing when work starts on `main` or `master`.
