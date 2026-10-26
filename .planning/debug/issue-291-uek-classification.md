---
status: resolved
trigger: "address issue 291"
created: 2026-10-25
updated: 2026-10-25
---

## Symptoms

Production scan and reboot preflight classified three UEK advisories as `investigate` despite complete OpenSCAP status and verified running/default UEK. A later manual advisory-scoped DNF probe returned rc 0 and `Nothing to do.`. The run did not publish the contemporaneous DNF output or host classification reason, so its exact branch cannot be recovered from the issue or action log.

## Current Focus

- hypothesis: The published OpenSCAP package list contains the Arm `kernel-uek64k*` family, which the shared reboot-family regex excludes.
- test: Add the observed 11 package names to the production-shaped fixture and allow the exact `kernel-uek64k` family in classifier and reboot preflight checks.
- expecting: The three-advisory no-op scan selects reboot; unrelated packages and ambiguous or failed DNF probes still block reboot.
- next_action: Review and merge PR #292 through the normal release path. Any reboot requires separate operator approval on the refreshed issue.

## Evidence

- Issue #268 reports three UEK advisories, complete scan, no RHCK drift, and verified UEK posture, but omits package-family decision and DNF outcome.
- Run 40224439797 reclassified the same advisory IDs to investigate; no reboot or cleanup occurred. Its successful DNF task does not reveal the command exit code or output.
- `classify_findings.yml` selects investigate for missing package metadata, disallowed package family with DNF no-op, or unrecognized DNF output after the verified kernel checks.
- Branch scanner run 40257319058 reported complete OpenSCAP, three unchanged advisory IDs, verified running/default UEK, zero RHCK drift, DNF rc 0 with `Nothing to do.`, and exactly 11 disallowed `kernel-uek64k*` package names. This identifies the selected branch: no-op DNF with packages outside the configured reboot-family regex.
- Corrected branch scanner run 40259034186 refreshed issue 268 with `next_action: reboot`, the same three advisory IDs, complete OpenSCAP status, verified running/default UEK, zero RHCK drift, DNF rc 0/no-op, and zero packages outside reboot families. The workflow succeeded without reboot or installonly cleanup.

## Diagnostic repair

The October 25 run did not record enough evidence to establish its exact branch retrospectively. The subsequent branch scan demonstrated the current cause: Oracle's advisory metadata includes `kernel-uek64k*` Arm UEK package names, while the shared reboot-family regex admitted only `kernel-uek*`. The branch scan's DNF no-op and verified kernel posture rule out the other classifier branches for that scan. The reporting fix now exposes safe branch inputs; the targeted correction admits the exact 64K UEK family while preserving independent DNF no-op and boot-target safety checks. PR review round one identified that redacted arbitrary DNF previews could leak credentials. Round two found the same issue in the failed reboot preflight context published by cleanup. Scanner, reboot result, and failed-preflight comments now expose only allowlisted signals, lengths, and hashes. The round-two finding is a sibling-path miss of the same public-output invariant.

Verification: finding classification, report rendering, reboot result, update reconciliation, reboot preflight, failed-preflight cleanup, and repository automation fixtures passed. Changed Ansible files passed `ansible-lint` at the production profile. PR CI passed, a post-reassessment reviewer found no actionable findings, and live branch scan 40259034186 confirmed production classifies the three advisories as `reboot` without mutation.
