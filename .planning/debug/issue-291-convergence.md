---
status: complete
created: 2026-10-26
updated: 2026-10-26
pull_request: 292
---

# Issue 291 review convergence reassessment

## Finding lineage

| Round | Finding | Lineage | Evidence |
|---|---|---|---|
| 1 | Public scanner/reboot result templates included blacklist-redacted arbitrary DNF output. | Initial public-output invariant violation. | PR inline review `discussion_r4373440926`; fixed in `1d2e2e5`. |
| 2 | Failed reboot preflight wrote raw DNF output to failure context, which cleanup posted publicly. | Sibling-path miss of the same invariant. | PR comment `issuecomment-6406733153`; fixed in `fe3efcf`. |
| 3 | Structural automation test still requires the removed stream-limit default and raw-output excerpt fact. | Fix regression and test weakness: two stale shape assertions were missed when the output contract changed. | PR comment `issuecomment-6406777364`; CI run `40258435362` Workflow checks failed in repository automation tests; `scripts/test_security_patching_create_issue_tasks.py:397,426`. |

## Shared invariants

1. Public security issue bodies and comments may contain only bounded, known-safe DNF evidence: exit status, output length, allowlisted status signals, and SHA-256 fingerprints. Arbitrary stdout/stderr remains private. Truncation and blacklist redaction do not establish this invariant.
2. Reboot and installonly cleanup require a complete current scan, exact approved advisory set, verified UEK posture, reboot-family-only advisory metadata, a contemporaneous DNF no-op, and verified installed boot target. Ambiguous or failed DNF proof blocks mutation.
3. Structural tests must assert the current behavior and safety contract. They must not require retired configuration fields merely because an older implementation used them.

## Consumer and producer inventory

- OpenSCAP parser `scripts/oracle_linux_oscap_results.py` produces published advisory package names, including architecture alternatives. `classify_findings.yml` uses the shared reboot-family regex in `defaults/main.yml` and DNF probe evidence.
- `security-report.md.j2` publishes scanner host classification; `security-reboot-result.md.j2` publishes reboot reconciliation. Both now consume allowlisted DNF evidence.
- `validate_reboot_state.yml` runs the second DNF proof and writes a failure-context file with allowlisted evidence. `cleanup_failed_request.yml` truncates that file and posts it to a public issue. The new failed-preflight fixture follows this path.
- `classify_reboot_request.yml` gates mutation; `reboot_cleanup.yml` selects an installed `kernel-uek-core` boot target, reboots, verifies health, then removes old installonly kernels.
- `scripts/test_security_patching_create_issue_tasks.py` checks workflow structure. Its lines 397 and 426 still require an obsolete stream excerpt limit and stdout excerpt fact; no runtime task uses either field.

## Assumption audit

- Confirmed by production scan `40257319058`: three unchanged advisories, complete OpenSCAP status, verified running/default UEK, zero RHCK drift, DNF rc 0 and `Nothing to do.`, and 11 `kernel-uek64k*` advisory package names. The shared regex excluded those names. Oracle documents `kernel-uek64k` as an Arm UEK 64K-page variant; the package list is advisory metadata, not installed-package inventory.
- Corrected assumption: bounding arbitrary DNF output by characters or redacting familiar credential syntax does not make public comments safe. A safe allowlist plus hashes is required for every public path.
- Corrected assumption: removing a now-unused default or derived fact is not complete until structural tests and documentation are checked for references. Both line-397 (stream limit) and line-426 (stdout excerpt fact) assertions are stale because raw stream excerpts no longer exist. A search limited to the removed default missed the fact assertion.
- Still to verify: a new branch scan after the regex change should classify the production host as `reboot`. No reboot or cleanup is authorized by this reassessment.

## Failure matrix

| Input | Expected action/public output | Mutation |
|---|---|---|
| Three UEK advisories including `kernel-uek64k*`, verified UEK, DNF rc 0/no-op | `reboot`; package-family evidence clear | Only after separate approval and full preflight |
| Unrelated package in advisory list | `investigate`; name listed | Blocked |
| DNF ambiguous or nonzero | `investigate`; safe signals, lengths, hashes only | Blocked |
| Second DNF proof fails with bearer-token-shaped output | Failure cleanup comment omits token and removes approval label | Blocked |
| Structural test expects removed stream limit or stdout excerpt | Both assertions should check the safe-evidence contract and absence of raw excerpts | No runtime mutation |

## Revised implementation plan requirements

1. Replace both stale assertions at lines 397 and 426 with behavioral/structural assertions tied to the current public-output contract: the obsolete stream limit is absent; second preflight evidence includes safe signals, lengths, and hashes; failure context does not reference raw stdout/stderr excerpts. Search for both retired `stream_max_chars` and `stdout_excerpt`/`stderr_excerpt` names before editing.
2. Keep the existing failed-preflight Ansible fixture with a bearer token; assert its prepared public cleanup details omit the token and reboot remains blocked.
3. Run the repository automation unit test, reboot-state Ansible fixture, affected Ansible lint, and PR CI. Inspect the resulting PR diff for unrelated changes.
4. After clean CI and review, run the weekly scanner from the PR branch and inspect issue 268. Do not apply a reboot label as part of this plan.

## Resume criteria

- The assumptions and revised plan above have been reviewed, with review evidence linked here.
- Evidence references for both the assumption review and plan review are recorded below before status changes to `ready_to_resume`.
- Coder work is limited to the stale structural assertion and any directly necessary contract test adjustment. A new Warning or Critical finding at this checkpoint requires another reassessment of the shared contract before further fixes.

## Review evidence

- Initial assumption/plan review: PR comment `issuecomment-6406793030` required the two-assertion correction above.
- Assumption review after correction: SOUND, PR comment `issuecomment-6406802231` (https://github.com/jetsaredim/autographs/pull/292#issuecomment-6406802231).
- Revised plan review after correction: SOUND, PR comment `issuecomment-6406802231` (https://github.com/jetsaredim/autographs/pull/292#issuecomment-6406802231).

## Closeout evidence

- Reviewed plan executed in commit `a1154bd`; 14 repository automation tests passed, and Workflow checks passed in PR run `40258803438`.
- Post-reassessment reviewer reported no actionable findings in PR comment `issuecomment-6406832436`.
- All required PR checks passed at `a1154bd`.
- Branch scanner run `40259034186` succeeded and refreshed issue 268 to `next_action: reboot` for the same three advisories, with complete scan, verified UEK posture, DNF no-op, and zero disallowed packages. No reboot or installonly cleanup occurred.
