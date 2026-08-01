---
phase: 08-admin-media-review-and-operational-posture
plan: "01"
subsystem: operations
tags: [ansible, github-actions, security-patching, oracle-linux, oval]

requires:
  - phase: 07-metadata-taxonomy-and-public-facets
    provides: completed Rust/static admin and public taxonomy foundation before Phase 8 operational repair
provides:
  - Minimal production host security package inventory without per-advisory detail loops
  - Best-effort Oracle Linux OVAL advisory enrichment outside the runtime host loop
  - Package-spec-only scanner approval metadata with visible advisory/CVE report detail
  - Operator verification runbook for live scanner repair proof
affects: [phase-08, operations, security-patching, github-actions, ansible]

tech-stack:
  added: [python-stdlib-cli]
  patterns:
    - Runtime hosts collect only package/advisory inventory needed for drift checks
    - Localhost enrichment may degrade without blocking scanner issue creation
    - Hidden approval metadata remains exact package specs only

key-files:
  created:
    - scripts/oracle_linux_advisory_enrichment.py
    - scripts/test_oracle_linux_advisory_enrichment.py
  modified:
    - deploy/ansible/roles/security_patching/defaults/main.yml
    - deploy/ansible/roles/security_patching/tasks/scan.yml
    - deploy/ansible/roles/security_patching/tasks/create_issue.yml
    - deploy/ansible/roles/security_patching/templates/security-report.md.j2
    - docs/security-patching.md

key-decisions:
  - "Production security scans now avoid host-side per-advisory detail loops and derive approval package specs directly from dnf updateinfo list output."
  - "Oracle Linux advisory enrichment is report detail only; enrichment failure degrades the report but does not block issue creation when package inventory exists."
  - "Scanner approval metadata remains package-spec-only, with CVEs, severity, summaries, and advisory links kept out of the hidden metadata block."

patterns-established:
  - "Best-effort enrichment: run external advisory lookups from localhost with failed_when false and preserve the minimal approval contract."
  - "Stale approval reset: scanner issue updates replace labels with security_patching_issue_labels so approved-production-update cannot survive package drift."

requirements-completed: [OPS-01]

duration: 8min
completed: 2026-07-31
---

# Phase 08 Plan 01: Production Security Patching Repair Summary

**Production security scans now collect exact package specs quickly, enrich advisory detail off-host when possible, and keep approval metadata package-spec-only.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-31T20:03:53Z
- **Completed:** 2026-07-31T20:11:09Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Removed the slow runtime `dnf updateinfo info` advisory-detail loop from `scan.yml`.
- Added `scripts/oracle_linux_advisory_enrichment.py`, a standard-library CLI that reads scanner inventory JSON, fetches Oracle Linux OVAL data when available, and degrades safely when unavailable.
- Updated GitHub issue rendering so hidden metadata contains only exact `package_specs`, while visible rows can show advisory links, severity, and CVEs.
- Documented Phase 8 live scanner repair verification, degraded enrichment review, same-issue updates, stale approval removal, and dry-run apply-path checks.

## Task Commits

1. **Task 1: Replace slow runtime advisory loop with minimal package inventory** - `786155d` (fix)
2. **Task 2 RED: Add failing advisory enrichment tests** - `6537154` (test)
3. **Task 2 GREEN: Add best-effort Oracle advisory enrichment** - `88bfe2c` (feat)
4. **Task 3: Preserve stale approval removal and document scanner verification** - `6053ca7` (docs)

## Files Created/Modified

- `deploy/ansible/roles/security_patching/defaults/main.yml` - Adds default minimal enrichment status/message.
- `deploy/ansible/roles/security_patching/tasks/scan.yml` - Emits minimal package/advisory facts from `dnf updateinfo list --security --available`.
- `deploy/ansible/roles/security_patching/tasks/create_issue.yml` - Writes localhost inventory JSON, runs enrichment with `failed_when: false`, and indexes valid enrichment output by host.
- `deploy/ansible/roles/security_patching/templates/security-report.md.j2` - Keeps hidden approval metadata package-spec-only and renders visible enrichment status/advisory detail.
- `scripts/oracle_linux_advisory_enrichment.py` - Provides parse, OVAL load, errata-link, inventory enrichment, and CLI entrypoint functions.
- `scripts/test_oracle_linux_advisory_enrichment.py` - Covers errata links, OVAL CVE/severity parsing, degraded fallback, and CLI output.
- `docs/security-patching.md` - Updates scanner behavior docs and adds Phase 8 verification steps.

## Decisions Made

- Used Oracle's consolidated OVAL archive URL as the default enrichment source, matching Oracle's documented `linux.oracle.com/security/oval` convention.
- Kept enrichment out of the runtime host path and treated it as non-authoritative report detail.
- Preserved same-issue scanner updates with replacement labels so stale `approved-production-update` labels are removed on drift.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed temporary-directory lifetime in the new enrichment CLI test**
- **Found during:** Task 2 (GREEN)
- **Issue:** The new `test_main_writes_degraded_inventory_when_oval_fails` read its output after the temporary directory context exited.
- **Fix:** Moved the output read/assertions inside the temporary directory context.
- **Files modified:** `scripts/test_oracle_linux_advisory_enrichment.py`
- **Verification:** `python3 -m unittest scripts/test_oracle_linux_advisory_enrichment.py` passed.
- **Committed in:** `88bfe2c`

---

**Total deviations:** 1 auto-fixed (1 bug).
**Impact on plan:** The fix corrected the new test harness only; no scope expansion or approval-contract change.

## Issues Encountered

- Latest available weekly scan evidence was captured from GitHub: run `30255940980` failed on 2026-07-27 in job `Scan production security updates`, step `Run security scan playbook`.
- No post-repair production `workflow_dispatch` scan was started during local execution; `docs/security-patching.md` now contains the operator proof path.

## Verification

- `grep -v '^#' deploy/ansible/roles/security_patching/tasks/scan.yml | grep -c 'updateinfo info' | grep '^0$'`
- `python3 -m unittest scripts/test_oracle_linux_advisory_enrichment.py`
- `python3 scripts/oracle_linux_advisory_enrichment.py --input /tmp/nonexistent-security-input.json --output /tmp/nonexistent-security-output.json; test "$?" != "0"`
- `python3 -c "from pathlib import Path; text=Path('deploy/ansible/roles/security_patching/templates/security-report.md.j2').read_text(); start=text.index('autographs-security-patch-metadata'); end=text.index('-->', start); block=text[start:end]; assert 'package_specs' in block; forbidden=['cves','severity','summary','advisory']; leaked=[name for name in forbidden if name in block]; assert not leaked, leaked"`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/roles/security_patching deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml`
- `git diff --check`

## Known Stubs

None.

## Threat Flags

None. The new Oracle OVAL lookup and GitHub issue-rendering behavior are inside the plan's documented external-advisory and issue-renderer trust boundaries.

## User Setup Required

None - no new external service configuration required.

## Next Phase Readiness

OPS-01 is locally satisfied. Phase 8 can proceed to the repo-wide posture and CI hygiene work, with live scanner proof left as an operator-run verification step after this repair is deployed.

## Self-Check: PASSED

- Created/modified files exist on disk.
- Task commits `786155d`, `6537154`, `88bfe2c`, and `6053ca7` exist in git history.
- No stub patterns were found in created/modified plan files.

---
*Phase: 08-admin-media-review-and-operational-posture*
*Completed: 2026-07-31*
