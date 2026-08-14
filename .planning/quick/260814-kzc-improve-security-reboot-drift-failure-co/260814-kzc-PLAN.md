# Quick Task 260814-kzc: Improve Security Reboot Drift Failure Comments

## Goal

Make failed `approved-production-reboot` attempts explain actionable validation failures in the GitHub issue and refresh stale scanner issue metadata when the fresh pre-reboot OpenSCAP scan has drifted from the approved advisory set.

## Tasks

1. Persist reboot validation failure context before failing closed.
2. Teach failed-request cleanup to refresh drifted scanner issues from the persisted current scan payload and include persisted context in the issue comment.
3. Add fixture coverage and docs for the improved failure comment path.
