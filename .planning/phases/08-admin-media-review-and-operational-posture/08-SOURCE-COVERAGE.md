# Phase 8 source coverage

This audit maps the Phase 8 goal, requirements, research, and locked context decisions to the canonical execution plans. PR-specific `*-REVISED-WORKPLAN.md` files are review records, not additional phase execution plans.

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
| OPS-01 | 08-01, 08-08 | Scanner and live apply/reboot evidence are recorded; the apply workflow has no dry-run mode. |
| OPS-02 | 08-02, 08-03, 08-07, 08-08 | Production CDN, admin bypass, and media HIT are evidenced; the IaC operating review remains open. |
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

The [Wave 8 checkpoint](08-08-CHECKPOINT.md) records a same-image before/after thumbnail and detail URL pair, hashes matching the final-byte fingerprints, the production Cloudflare cutover, repeated media `MISS` then `HIT`, and authenticated admin bypass evidence. The owner deferred a live rollback rehearsal because changing production DNS or proxy behavior without an incident would introduce avoidable availability risk; the DNS runbook retains the reviewed recovery path. The credential-gated automated static smoke was not run live in 08-07. The Terraform/Ansible operating inventory remains open before Phase 8 closeout.
