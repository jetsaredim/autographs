---
phase: 08-admin-media-review-and-operational-posture
plan: "08"
task: 3
status: awaiting_human_action
observed_at: 2026-11-06T14:45:56Z
---

# Wave 8 production CDN checkpoint

## Earlier public baseline

The public item JSON at `/data/items/5d6-ra-7-fivedesix.json` currently references these two variants for image 1:

| Variant | Public path | SHA-256 of served bytes |
|---|---|---|
| Thumbnail | `/media/5d6-ra-7-fivedesix/image-1-thumbnail-026b3952b058f219.webp` | `026b3952b058f2190cfe1196792f6420735f93d145a2778b9c42abf469f93be6` |
| Detail | `/media/5d6-ra-7-fivedesix/image-1-detail-70c219182f1ace41.webp` | `70c219182f1ace4167b0c501753e4c2421779df15a7932c92d46e9bb0d130d72` |

Both 16-character filename fingerprints match the first 16 hex characters of their served-byte SHA-256 digests. This item was an initial candidate; the owner selected `5d6-ra-7` below for the actual adjustment test.

## Adjusted-media publish proof: `5d6-ra-7`

At 2026-11-06 15:09 UTC, `/data/items/5d6-ra-7.json` referenced image 1 with these public variants, and both URLs returned 200 with `Cache-Control: public, max-age=86400`:

| Variant | Before public path | SHA-256 of served bytes |
|---|---|---|
| Thumbnail | `/media/5d6-ra-7/image-1-thumbnail-d63126d844b4bbcb.webp` | `d63126d844b4bbcb62e8363981326d5467029fd2fef9d02f5dc0bea0d3ebfe57` |
| Detail | `/media/5d6-ra-7/image-1-detail-913001cc8d9fa261.webp` | `913001cc8d9fa2614069bc6a840f42420ef2076a6351c7b2b9d8b4a24584f9f1` |

After the owner's adjustment and publish, the same public item JSON referenced these changed variants at 2026-11-06 15:38 UTC:

| Variant | After public path | SHA-256 of served bytes |
|---|---|---|
| Thumbnail | `/media/5d6-ra-7/image-1-thumbnail-89079abbdcfc1574.webp` | `89079abbdcfc1574c58362f4c064382ab0cf104e8a1318894ef1abced9f327c4` |
| Detail | `/media/5d6-ra-7/image-1-detail-0ddf88ebddaf88e8.webp` | `0ddf88ebddaf88e89a42b7bf740e44b9cb1a1ad71b4717399ebeb5b0d000cfcb` |

Both new URLs returned 200 with `Cache-Control: public, max-age=86400`; the hashes match their new filename fingerprints. The before and after fingerprints differ for both variants. The public detail HTML references the new detail URL, and `/data/collection.json` references the new thumbnail URL. Both old URLs still returned 200 at the after observation. This proves immediate retained-old-path availability, not the full retention duration.

## Origin and DNS observations

On 2026-11-06, public responses showed `/admin/` 200 with `Cache-Control: no-store`, unauthenticated `/admin/api/status` 401 with `no-store`, `/data/collection.json` 200 with `public, max-age=60, must-revalidate`, and the current detail WebP 200 with `public, max-age=86400`. The public responses showed `server: Caddy` and no `CF-Cache-Status` header. `dig NS jetsaredim.net` returned four Porkbun nameservers. These are origin observations and do not prove Cloudflare participation or authenticated API bypass.

## Current closeout gates

The Cloudflare cutover, authenticated admin bypass, public document headers,
repeated media cache HIT, and owner-approved rollback deferral are recorded
below. Remaining:

1. Complete the Terraform/Ansible operating-contract inventory and final
   P8-POSTURE-006 disposition.
2. Review `AGENTS.md` against the current architecture and Phase 8 status,
   update stale guidance and its source maps, and ensure the root `SECURITY.md`
   gives a usable vulnerability reporting path.
3. Write `08-08-SUMMARY.md` and update Phase 8 state and requirement status
   after those gates are satisfied.

The root `SECURITY.md` now exists. GitHub private vulnerability reporting was
disabled when checked on 2026-11-21, so the policy directs reporters to request
a private channel in a public issue without disclosing vulnerability details.
Confirm or improve that reporting route at final closeout.

## 2026-11-19 closeout progress

- [PR #371](https://github.com/jetsaredim/autographs/pull/371) merged as
  `abc17a0` with passing PR checks. It flags destructive actions in the
  authenticated CI plan and requires a fresh PR label-add event after review;
  routine release deployment retains its saved-plan apply. PR #378 removes
  the DNS comparison added later in PR #377.
- `security_scan_run_id: 43932933877` — the
  [2026-11-16 scanner run](https://github.com/jetsaredim/autographs/actions/runs/43932933877)
  succeeded. [Security report #326](https://github.com/jetsaredim/autographs/issues/326)
  records the earlier approval sequence: first
  [update run 43556618370](https://github.com/jetsaredim/autographs/actions/runs/43556618370)
  detected advisory drift (two approved, four current) and made no package
  changes; second
  [update run 43630320143](https://github.com/jetsaredim/autographs/actions/runs/43630320143)
  applied the four approved advisories and left two reboot/installonly
  findings; [reboot run 43631340188](https://github.com/jetsaredim/autographs/actions/runs/43631340188)
  rebooted, cleaned installonly packages, and reported zero remaining findings.
  All three jobs and their playbook steps succeeded; the issue closed.
  `dry_run_apply_run_id: not performed` because the approval-label update
  workflow has no dry-run mode and the production apply/reboot path was
  exercised live. The real runs are not described as dry runs.
- Local closeout gates passed: controller formatting, default tests, strict
  Clippy, production-persistence check, admin JavaScript syntax, repository
  automation tests and hygiene, Ansible syntax, and Ansible lint. The ignored
  live static publish test compiled and skipped because its explicit live flag
  was absent; compiling it exposed and resolved a digest-formatting error in
  `live_persistence_smoke.rs`.
- On 2026-11-19, public edge probes returned `/admin/` 200 with `no-store`
  and `CF-Cache-Status: DYNAMIC`, `/data/collection.json` 200 with
  `public, max-age=60, must-revalidate` and `DYNAMIC`, and two GETs of the
  adjusted fingerprinted detail WebP returned 200 with the one-day origin
  cache header and Cloudflare `MISS` then `HIT`.
- The operator supplied two signed-in browser Network screenshots of
  `GET /admin/api/publish/status` on 2026-11-19 at 05:07:43 and 05:07:55 UTC.
  Both responses were 200 with `Cache-Control: no-store` and
  `CF-Cache-Status: DYNAMIC`; neither was an edge HIT. This satisfies the
  repeated authenticated admin API header check on a live admin endpoint.
  The operator also supplied a Cloudflare Rules simulator screenshot for
  `GET /admin/api/publish/status`. It shows the matched Cache Rule
  "Bypass admin shell and private API". The matching Terraform rule has
  `action_parameters.cache = false`. The simulated request ended 401 without
  an admin session; the authenticated browser requests above establish the
  separate 200 response behavior. The browser connector reported no
  accessible host, so these screenshots are operator-provided evidence.
- On 2026-11-19, the owner confirmed the admin UI currently shows 0 pending
  changes. This is a later clean-state observation following the measured
  2026-11-15 publish, not a capture immediately after that run.
- `rollback_evidence: deferred by owner on 2026-11-20`. A rehearsal would
  intentionally change the live Cloudflare proxy or DNS delegation without an
  incident, introducing availability and admin-access risk for a low-probability
  recovery scenario. The owner judged that disruption unnecessary. The
  [DNS runbook](../../../docs/dns-runbook.md#ongoing-cloudflare-changes)
  documents a reviewed `CLOUDFLARE_MODE=dns_only` override and saved release
  plan/apply, followed by restoration to `proxied` after an incident. Short-lived
  HTML/JSON and fingerprinted media give release rollback its documented
  freshness behavior. No live rollback was performed or claimed.
- The Terraform/Ansible/workflow source inventory now has entry-specific
  inputs, supported states, failure/recovery paths, tests, and dispositions.
  On 2026-11-21, the owner named IAC-004 and IAC-007, asked to review six
  operational switch flows, and chose ordinary release deployment after PR
  review for Cloudflare changes. [PR #376](https://github.com/jetsaredim/autographs/pull/376)
  removed obsolete Node deploy cleanup, passed two reviews and CI, and merged
  as `b724d75`. [PR #377](https://github.com/jetsaredim/autographs/pull/377)
  changed the proxied default and shared release apply path, passed CI and its
  second review, and merged as `e84d58f`. Its first review
  found a critical unreviewed-DNS-action path; the revised gate binds DNS
  actions to the successful Release PR CI record. The owner subsequently
  clarified that this extra apply gate is unwanted; PR #378 removes it and
  leaves destructive-plan approval in normal and Release PR CI.
  The duplicate `CLOUDFLARE_MODE=proxied` GitHub variable was removed after
  the new default merged. The authenticated
  [post-removal Terraform CI job](https://github.com/jetsaredim/autographs/actions/runs/44908578055/job/136859340147)
  used the then-current workflow fallback `proxied` and reported no changes.
  PR #378 makes Terraform's default the sole source unless an override is set.
  No apply ran. The
  Release PR has not been merged or deployed.
  PR #375's
  inventory and `/tmp/autographs-iac-operational-review.html` now explain the
  switches and runtime-secret readiness. PR #375 passed its mandatory third
  review and CI; PR #377 still needs release-path verification;
  the owner has not yet accepted the revised inventory. P8-POSTURE-006 has no
  final owner disposition. Do not mark Plan 08-08 or Phase 8 complete.
