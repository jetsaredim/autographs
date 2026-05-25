---
status: complete
date: 2026-05-25
slug: reconcile-docs-and-workflow-guardrails
key-files:
  modified:
    - .planning/STATE.md
    - .planning/codebase/ARCHITECTURE.md
    - .planning/codebase/CONCERNS.md
    - .planning/codebase/CONVENTIONS.md
    - .planning/codebase/INTEGRATIONS.md
    - .planning/codebase/STRUCTURE.md
  external:
    - /home/jgreenwa/.codex/get-shit-done/references/context-budget.md
    - /home/jgreenwa/.codex/get-shit-done/references/agent-contracts.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/plan-phase.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/review.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/code-review.md
    - /home/jgreenwa/.codex/get-shit-done/workflows/ship.md
    - /home/jgreenwa/.codex/agents/gsd-code-reviewer.md
---

# Summary

Refreshed stale codebase-map docs so they describe the implemented Phase 1-3 app, OCI/Oracle/Object Storage integrations, GitHub delivery spine, and Phase 4 admin boundary.

Added global GSD guardrails for token stewardship and PR review visibility:

- `context-budget.md` now explicitly asks workflows to choose the cheapest safe path, trim prompts, avoid duplicate work, and warn before heavy fan-out.
- `agent-contracts.md` adds a token stewardship contract for concise structured agent output.
- `plan-phase.md`, `review.md`, `code-review.md`, `ship.md`, and `gsd-code-reviewer.md` now carry targeted concise-context or PR-comment requirements.
- `code-review.md` now requires posting actionable findings to an associated GitHub PR, with a PR-level fallback when inline mapping is unavailable.
- `ship.md` now requires automated PR review findings to be posted to the PR before the workflow is considered complete.

## Verification

- Repo docs were reviewed for removal of planning-only claims.
- Previous app verification in this session was green: test, lint, typecheck, and build all passed.
