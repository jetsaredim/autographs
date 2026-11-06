---
phase: 08-admin-media-review-and-operational-posture
pr: 324
status: reassessment_required
review_round: 3
critical_open: 0
warning_open: 2
---

# PR #324 Wave 8 review convergence reassessment

The third review is the mandatory convergence checkpoint. No further point fix is authorized until the assumptions and implementation plan below have been revised and independently reviewed. A status-only change to `ready_to_resume` is invalid.

## Finding lineage

| Finding | First seen | Classification | Current state | PR evidence |
|---|---:|---|---|---|
| DNS topology from Porkbun to Cloudflare was unspecified | Round 1 | Independent | Resolved in `203cc2e` | [finding](https://github.com/jetsaredim/autographs/pull/324#discussion_r4489373430) |
| Old/new adjusted-media URL proof was optional | Round 1 | Independent | Resolved in `203cc2e` | [finding](https://github.com/jetsaredim/autographs/pull/324#discussion_r4489374171) |
| Repeated media MISS could pass CDN cache check | Round 1 | Independent | Resolved in `203cc2e` | [finding](https://github.com/jetsaredim/autographs/pull/324#discussion_r4489374729) |
| Anonymous 401 was used as sole admin API bypass evidence | Round 2 | Sibling-path miss | Resolved in `dd92c6c` | [finding](https://github.com/jetsaredim/autographs/pull/324#discussion_r4489393931) |
| Static runbook asks for edge observations before proxy enablement | Round 3 | Sibling-path miss | Open | [finding](https://github.com/jetsaredim/autographs/pull/324#issuecomment-6677889197) |
| CDN contract calls the static admin shell authenticated | Round 3 | Independent | Open | [finding](https://github.com/jetsaredim/autographs/pull/324#issuecomment-6677894390) |

The first four lineages were fixed in the plan and primary CDN/DNS documentation. The fifth arose because the static runbook was a direct consumer of the same activation sequence and retained older wording. The sixth is an inaccurate architecture assumption introduced in the initial security description.

## Shared invariants

1. **Activation order:** prove a changed adjusted-media URL at the origin, prepare a supported DNS route and rules, enable proxying, then observe edge behavior. A pre-proxy check can establish only origin behavior.
2. **Admin boundary:** Caddy serves the static `/admin/` shell with `no-store`; session authentication protects `/admin/api/*` data and actions. Edge rules must bypass both paths, including authenticated 200 API responses.
3. **Cache proof:** the public media path must be final-byte fingerprinted, and a post-proxy repeated request must reach an edge HIT. Public HTML/JSON must remain short-lived for rollback.

## Producer, consumer, and action inventory

| Boundary | Producers | Direct consumers and reporting/action paths | Current evidence |
|---|---|---|---|
| Public media URL | Adjustment repository, publisher, public JSON/HTML | Browser, Caddy shared media, CDN, static smoke, CDN contract, static runbook, 08-08 plan | Current URL/hash match; old/new pair pending |
| Admin shell/API | Caddy `/admin/*` file server and `/admin/api/*` reverse proxy; controller session auth | Browser, Cloudflare bypass rule, route tests, security review, DNS/CDN/deployment runbooks, 08-08 checkpoint | Origin no-store and anonymous API 401; authenticated edge 200 pending |
| DNS and edge cache | Porkbun authoritative zone, Cloudflare supported zone setup and cache rules | Operator migration, CDN contract, DNS runbook, deployment runbook, 08-08 summary and phase verification | Porkbun NS and Caddy origin headers; no Cloudflare edge proof |
| Rollback | Publisher release pointer, retained media, short-lived documents | Operator runbook, CDN purge procedure, 08-08 checkpoint | Source contract documented; post-proxy exercise pending |

## Assumption audit

| Assumption | Verdict | Consequence |
|---|---|---|
| A current fingerprinted URL proves the image URL changed after adjustment | False | Require old/new URL pair for one same-image publish. |
| A 401 API response proves CDN bypass for authenticated admin data | False | Require repeated authenticated 200 response headers and Rule Trace. |
| Repeated media MISS proves caching | False | Require an observed edge HIT. |
| A Cloudflare proxied record can be enabled while Porkbun remains authoritative on any plan | False | Choose a supported full-zone migration or eligible partial/CNAME setup first. |
| Edge cache observations can be collected before proxy enablement | False | Keep pre-proxy origin probes separate from post-proxy edge probes. |
| The static admin shell itself requires a session | False | Describe shell as no-store static content; API data and actions are session protected. |

## Failure matrix

| Case | Expected result | Failure signal |
|---|---|---|
| Before proxy, origin admin/API and public paths | Caddy no-store for admin/API; 60s documents; 1d media | Wrong origin header blocks enablement. |
| Before proxy, adjusted image publish | New thumbnail and detail URLs plus matching bytes; old URLs retained | Missing old/new pair or same fingerprint blocks enablement. |
| DNS setup | Supported zone setup with all records preserved | Porkbun remains authoritative without eligible partial setup, or missing records. |
| After proxy, anonymous admin shell | no-store and no edge HIT | Cached shell or missing no-store. |
| After proxy, authenticated API 200 | no-store, no edge HIT, matching bypass Rule Trace | Only 401 evidence, edge HIT, or unmatched bypass rule. |
| After proxy, media URL | Origin public TTL and edge HIT after repeat | Repeated MISS/BYPASS or absent edge status. |
| Rollback | Prior release documents converge within short TTL; retained media serves | Stale references beyond contract or missing retained media. |

## Revised plan requirements

- Replace static-runbook language that requests edge observations before CDN enablement with the explicit origin-before, edge-after sequence. Inspect every other direct consumer for the same ordering claim.
- Correct the CDN contract's `/admin/*` architecture claim: static shell is anonymously served with `no-store`; session auth belongs to private API routes. Preserve CDN bypass for both.
- Align the 08-08 plan, security review, DNS/deployment/static runbooks, and source coverage with those two invariants in one coherent documentation change. Do not weaken the old/new URL, authenticated 200 bypass, or media HIT gates.
- Run focused source consistency, repository hygiene, and `git diff --check` checks. Post the implementation evidence on PR #324 and request an independent review of the revised plan and final diff.

## Resume criteria

1. `08-PR-324-ASSUMPTIONS.md` records the corrected architecture and activation assumptions.
2. `08-PR-324-REVISION.md` gives one coherent edit/verification sequence and is reviewed independently.
3. This artifact records links to both the reviewed assumptions and the reviewed plan, plus the PR review evidence, before `status` changes to `ready_to_resume`.
4. Only after criterion 3 may coder work resume. Phase 8 completion still requires the separate human CDN checkpoint.

## Evidence references before resume

- Assumptions artifact: pending
- Revised plan artifact: pending
- Independent plan review: pending
