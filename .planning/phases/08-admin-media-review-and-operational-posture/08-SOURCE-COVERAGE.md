# Phase 8 source coverage

This audit maps the Phase 8 goal, requirements, research, and locked context decisions to the canonical execution plans. PR-specific `*-REVISED-PLAN.md` files are review records, not additional phase execution plans.

## Goal coverage

| Goal outcome | Plans | Evidence |
|---|---|---|
| Repair production security patching | 08-01, 08-08 | Scanner implementation and operator verification guidance |
| Restore repository and process hygiene | 08-02, 08-08 | Pre-media PR #207, CI hygiene gate, closeout review |
| Define and verify CDN/cache posture | 08-03, 08-07, 08-08 | Origin contract, fingerprinted adjusted derivatives, production edge checkpoint |
| Review and correct private images | 08-04 through 08-08 | Adjustment model, Oracle persistence, private workbench, publisher, owner UAT |

## Requirement coverage

| Requirement | Plans | Closeout state |
|---|---|---|
| OPS-01 | 08-01, 08-08 | Implementation complete; live scan and dry-run evidence remain operator checks. |
| OPS-02 | 08-02, 08-03, 08-07, 08-08 | Posture and cache contract complete; production CDN enablement and edge verification remain open. |
| MEDIA-05 | 08-06, 08-08 | Private authenticated preview implemented; security review and owner UAT recorded at closeout. |
| MEDIA-06 | 08-04, 08-05, 08-06, 08-07, 08-08 | Non-destructive corrections and adjusted publishing implemented; operator UAT supports the workflow. |
| ADMIN-06 | 08-06, 08-08 | Focused image workbench implemented; owner UAT and closeout review complete the requirement. |

## Research coverage

| Research concern | Plans | Evidence |
|---|---|---|
| Minimal host scan with external advisory enrichment and fail-safe approval | 08-01, 08-08 | Scanner role, issue report, production runbook |
| CI hygiene and stale-map enforcement | 08-02, 08-08 | `validate_repo_hygiene.py`, CI, codebase map refresh |
| Origin cache contract before CDN enablement | 08-03, 08-08 | Caddy route tests and CDN contract |
| Authenticated private previews and redacted errors | 08-06, 08-08 | Admin route tests and security review |
| Non-destructive adjustment, source-bound cache key, final-byte fingerprint | 08-04 through 08-08 | Adjustment, publisher, static contract, and live procedure |

## Decision coverage

| Decision | Plans | Decision | Plans |
|---|---|---|---|
| D-08-01 | 08-01, 08-02, 08-03, 08-08 | D-08-02 | 08-03, 08-08 |
| D-08-03 | 08-03, 08-08 | D-08-04 | 08-01 |
| D-08-05 | 08-01 | D-08-06 | 08-01 |
| D-08-07 | 08-01 | D-08-08 | 08-01 |
| D-08-09 | 08-01 | D-08-10 | 08-01 |
| D-08-11 | 08-01, 08-08 | D-08-12 | 08-01 |
| D-08-13 | 08-02 | D-08-14 | 08-02 |
| D-08-15 | 08-02 | D-08-16 | 08-02 |
| D-08-17 | 08-03, 08-08 | D-08-18 | 08-03, 08-07 |
| D-08-19 | 08-03, 08-08 | D-08-20 | 08-06 |
| D-08-21 | 08-06 | D-08-22 | 08-06 |
| D-08-23 | 08-06 | D-08-24 | 08-06 |
| D-08-25 | 08-04, 08-05, 08-06 | D-08-26 | 08-04, 08-06 |
| D-08-27 | 08-06 | D-08-28 | 08-06 |
| D-08-29 | 08-04, 08-05, 08-06 | D-08-30 | 08-06 |
| D-08-31 | 08-04 through 08-08 | D-08-32 | 08-04, 08-05, 08-07, 08-08 |
| D-08-33 | 08-06 | D-08-34 | 08-06, 08-08 |

## Live evidence boundary

The owner reports multiple production publishes and image adjustments. On 2026-11-06, the public item JSON for `5d6-ra-7-fivedesix` referenced `/media/5d6-ra-7-fivedesix/image-1-detail-70c219182f1ace41.webp`; the URL returned 200 and its SHA-256 began with `70c219182f1ace41`. This proves the current public reference and final-byte fingerprint agree. No earlier URL was retained for this item, so this observation alone does not prove a before/after URL transition. The credential-gated automated static smoke was not run live in 08-07. CDN cache participation remains unverified.
