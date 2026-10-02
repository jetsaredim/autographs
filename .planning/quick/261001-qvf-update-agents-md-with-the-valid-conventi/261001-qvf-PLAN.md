---
quick_id: 261001-qvf
status: planned
mode: quick
description: Update AGENTS.md with the repository's valid Conventional Commit type identifiers
estimate:
  tokens: 1200
  raw_tokens: 1200
  tasks: 1
  confidence: low
must_haves:
  truths:
    - AGENTS.md states that the valid Conventional Commit type identifiers are exactly feat, fix, perf, revert, docs, and chore.
    - AGENTS.md states that every other Conventional Commit type identifier is invalid for this repository.
    - The new rule is outside all GSD-managed/generated blocks, which remain unchanged.
  artifacts:
    - AGENTS.md
  key_links:
    - The manual Conventional Commit guidance sits after the GSD workflow block and before the existing Git Commit Branch Guardrails section.
---

# Quick Task 261001-qvf Plan

<objective>
Document the repository's exact Conventional Commit type allowlist in AGENTS.md without altering generated project guidance.

Purpose: Give contributors and agents an unambiguous commit-type contract that matches the repository's release validation policy.
Output: A manual AGENTS.md section defining the complete valid set and rejecting every other type identifier.
</objective>

<tasks>

<task type="auto">
  <name>Task 1: Add the exact Conventional Commit type allowlist</name>
  <files>AGENTS.md</files>
  <action>Add a manual `## Conventional Commit Type Identifiers` section immediately after the GSD workflow end marker and before `## Git Commit Branch Guardrails`, keeping it outside every GSD-managed/generated block. State that the valid Conventional Commit type identifiers are exactly `feat`, `fix`, `perf`, `revert`, `docs`, and `chore`, and state explicitly that all other type identifiers are invalid for this repository. Do not modify any existing GSD-managed/generated block or unrelated AGENTS.md guidance.</action>
  <verify>
    <automated>python3 -c 'from pathlib import Path; import re; text = Path("AGENTS.md").read_text(); heading = "## Conventional Commit Type Identifiers"; assert text.count(heading) == 1; section = text.split(heading, 1)[1].split("\n## ", 1)[0]; assert re.findall(r"`([a-z]+)`", section) == ["feat", "fix", "perf", "revert", "docs", "chore"]; assert "All other Conventional Commit type identifiers are invalid for this repository." in section; assert text.index("&lt;!-- GSD:workflow-end --&gt;") &lt; text.index(heading) &lt; text.index("## Git Commit Branch Guardrails")' &amp;&amp; git diff --check -- AGENTS.md</automated>
  </verify>
  <done>AGENTS.md contains one manual commit-type section naming exactly the six valid identifiers and declaring every other identifier invalid, with generated blocks and unrelated guidance untouched.</done>
</task>

</tasks>

## Source Coverage Audit

| Source | Item | Task | Status | Notes |
|--------|------|------|--------|-------|
| GOAL | Document the exact repository Conventional Commit type allowlist | 1 | COVERED | The single task adds and verifies the complete rule. |
| REQ | No roadmap requirement IDs apply to this quick task | — | N/A | Scope comes from the quick-task description. |
| RESEARCH | No research artifact applies | — | N/A | Level 0 internal documentation change. |
| CONTEXT | No numbered CONTEXT.md decisions apply | — | N/A | The orchestrator constraints are fully represented in Task 1. |

<verification>
Run the Task 1 automated command from the repository root and inspect `git diff -- AGENTS.md` to confirm that only the intended manual section changed.
</verification>

<success_criteria>
- The valid type identifiers are exactly `feat`, `fix`, `perf`, `revert`, `docs`, and `chore`.
- Other type identifiers are explicitly invalid for this repository.
- No GSD-managed/generated block or unrelated file is modified.
</success_criteria>

<output>
Create `.planning/quick/261001-qvf-update-agents-md-with-the-valid-conventi/261001-qvf-SUMMARY.md` when done.
</output>
