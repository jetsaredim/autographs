---
quick_id: 261105-v5y
status: complete
completed: 2026-11-05
---

# Elevated Git remote access guidance

Added a manual `AGENTS.md` section requiring Git commands that contact a remote to run with elevated privileges outside the AI sandbox before the first attempt. Local Git commands remain available inside the sandbox. The rule preserves the existing branch, pull request, and connectivity guardrails.

## Verification

- The corrected guidance was checked against the user's clarification.
- `git diff --check` and a focused `AGENTS.md` diff passed after the correction.

## Scope

No application or UI files changed. The Phase 8 execution task remains separate.
