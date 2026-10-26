---
status: investigating
trigger: "address issue 291"
created: 2026-10-25
updated: 2026-10-25
---

## Symptoms

Production scan and reboot preflight classified three UEK advisories as `investigate` despite complete OpenSCAP status and verified running/default UEK. A later manual advisory-scoped DNF probe returned rc 0 and `Nothing to do.`. The run did not publish the contemporaneous DNF output or host classification reason, so its exact branch cannot be recovered from the issue or action log.

## Current Focus

- hypothesis: A DNF probe or package-family input differs between the production scan and the later manual probe; reporting hides the difference.
- test: Publish bounded per-host branch inputs and reproduce the three-advisory no-op plus ambiguous/failed probes in fixtures.
- expecting: The no-op fixture selects reboot; ambiguous/failed probes remain investigate without approval.
- next_action: After deployment, run a fresh production scan and compare the published host evidence with the exact three-advisory classifier branches.

## Evidence

- Issue #268 reports three UEK advisories, complete scan, no RHCK drift, and verified UEK posture, but omits package-family decision and DNF outcome.
- Run 40224439797 reclassified the same advisory IDs to investigate; no reboot or cleanup occurred. Its successful DNF task does not reveal the command exit code or output.
- `classify_findings.yml` selects investigate for missing package metadata, disallowed package family with DNF no-op, or unrecognized DNF output after the verified kernel checks.

## Diagnostic repair

The historical October 25 run's exact DNF output and full parsed package list were not recorded, so its selected `investigate` branch cannot be established retrospectively. The demonstrated defect is the reporting gap: the host-specific reason and branch inputs were computed but omitted from the issue and reboot result. The scanner issue now publishes the host reason, posture facts, advisory IDs, missing/disallowed package metadata, and bounded DNF exit code and recognized signals. Reboot reconciliation preserves and publishes the same preflight evidence. A three-advisory UEK/no-op fixture selects `reboot`; ambiguous and failed DNF fixtures remain `investigate` without approval.

Verification: finding classification, report rendering, reboot result, update reconciliation, and reboot preflight fixtures passed. Changed Ansible files passed `ansible-lint` at the production profile. A live scanner run after deployment is still required to identify which input caused the historical production-shaped `investigate` result.
