---
quick_id: 260820-uqf
verified: 2026-08-21T02:13:13Z
status: human_needed
score: 3/3 documentation must-haves verified
---

# Quick Task 260820-uqf Verification

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | The smoke environment example includes both required passwords. | VERIFIED | `docs/static-runtime-runbook.md` includes explicit database and wallet password placeholders beside the Oracle wallet settings. |
| 2 | Spike 002 cannot be mistaken for a current runnable procedure. | VERIFIED | Its execution section is labeled historical/non-runnable and directs current operators to `Dockerfile.smoke`, `live-persistence`, and the active runbook. |
| 3 | Operators have a safe pre-merge migrated-controller gate. | VERIFIED | The runbook builds the exact candidate, starts it alongside production with copied wallet/secrets, targets it from the established static-publish smoke, defines success, and uses a strict subshell plus exit/signal trap for mandatory cleanup on every outcome. The extracted Bash block passes `bash -n`. |

## Human Verification Required

Run the candidate-controller static-publish smoke on the production VM and post the exact commit/result to PR #211. The PR remains blocked from merge until that succeeds and the reviewer agent confirms the finding is resolved.
