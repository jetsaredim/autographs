---
phase: 08-admin-media-review-and-operational-posture
plan: "08"
status: cloudflare_dns_authoritative_imported_proxy_pending
approach: terraform_managed_full_zone
---

# Cloudflare DNS and CDN migration for Wave 8

## Decision and ownership

Use the existing `infra/terraform/` runtime root and its current OCI state object for the Cloudflare full zone, every reviewed `jetsaredim.net` DNS record, strict TLS, and cache rules. The owner prefers one Terraform root and state. The existing deploy workflow auto-applies this root, so it refuses Cloudflare or Porkbun DNS changes. Ordinary runtime deployments may proceed only when the DNS plan has no changes; a controlled operator plan/apply handles DNS changes.

Porkbun remains the registrar, and Cloudflare is now authoritative. Proxy only `autographs.jetsaredim.net`; mail records remain DNS-only. Keep the current `porkbun_dns_record.autographs` and its import block until Cloudflare has passed the live checkpoint. Both old and new autograph A records may be managed during migration because they are at different DNS providers. After verification, retire the Porkbun resource with a reviewed `removed` block and `destroy = false`, preserving its real record for rollback.

This is a full-zone plan. Partial/CNAME setup would need a separate revised design. The owner confirms DNSSEC is disabled and does not want it enabled. Public `dig DS jetsaredim.net` returned no record on 2026-11-06; keep DNSSEC off and the registrar without a DS record.

The adjusted-media before/after URL gate passed in `08-08-CHECKPOINT.md`.
Porkbun Connect has now created the Cloudflare zone, copied records, and
delegated nameservers. Public Cloudflare authoritative queries show the
`autographs` A record DNS-only with TTL 600, MX priority 1, and the SPF TXT
record, each matching the confirmed inventory. Two old apex ACME challenge
records were removed because Caddy manages the only site's certificate.
The zone, A/MX/TXT records, and SSL setting were imported into the existing
OCI-backed Terraform state by [run 42689411270](https://github.com/jetsaredim/autographs/actions/runs/42689411270).
The import-only plan reported one cache ruleset creation, an SSL setting
update from `full` to `strict`, and a quote-only SPF update. There was no
destruction and no apply. Terraform's SPF source must match the imported
quoted representation before any controlled apply. The proxy is not active.

## Confirmed inputs

- The owner confirmed the complete Porkbun inventory: `autographs.jetsaredim.net A 129.213.166.159 TTL 600`; `jetsaredim.net MX 1 smtp.google.com TTL 600`; `jetsaredim.net TXT "v=spf1 include:_spf.google.com ~all" TTL 600`. Public DNS returned the same values on 2026-11-06. Keep a copy for rollback.
- Cloudflare account ID: `162e3fcbeb9e3479a73529a6d51806bc` (provided by the owner and saved as repository GitHub Actions variable `CLOUDFLARE_ACCOUNT_ID`).

## Inputs still required

- A controlled saved plan/apply for Full (strict) TLS and the cache rules,
  after Terraform's SPF source matches the imported state.
- The persistent GitHub mode variable has been set to `dns_only`; preserve it
  before any routine deployment and review the subsequent no-destroy plan.

## Staged implementation

### A. Prepare the existing Terraform root

1. The Cloudflare provider and full-zone, record, strict TLS, and cache-rule configuration are committed in `infra/terraform/`. The live `autographs` record remains DNS-only. Import the zone and records created by Porkbun Connect before any Terraform apply.
2. Keep the Porkbun provider and autograph record during transition. Use one persistent `CLOUDFLARE_MODE` input with `off`, `dns_only`, and `proxied` stages; reject any other value. CI and deploy must fail if the scoped token or account ID is missing when the mode is not `off`.
3. Add a pre-apply DNS-change guard to `.github/workflows/deploy.yml`: inspect the saved Terraform plan and stop ordinary deploys when Cloudflare or Porkbun DNS resources would change. A controlled operator operation must review `terraform show` and apply the exact saved plan file. A routine deploy after cutover must have a no-change DNS plan and preserve `proxied = true`.
4. Configure admin/API bypass; public HTML/JSON origin-respecting or at most 60-second freshness; fingerprinted media one-day origin TTL and assets one-hour origin TTL. Use disjoint hostname/path matches and verify last-match rule behavior. Avoid sitewide cache-everything and custom cache keys.
5. CI provider validation, GitHub secret/variable documentation, and operator runbooks were updated. Do not apply further Cloudflare changes until the live inventory is checked and the state import and plan are reviewed.

### B. Import the authoritative Cloudflare zone into Terraform state

1. Run the import-only workflow from reviewed `main`. It must verify the active full-zone account and exact three-record A/MX/TXT inventory, then import the zone, records, SSL setting, and any existing cache ruleset into the existing OCI-backed state. It does not run `terraform apply`.
2. Review the sanitized `dns_only` production plan. Reject unexpected updates, replacements, destruction, duplicate DNS records, or proxy activation. Apply any required strict TLS or cache-rule changes through a separate controlled operator saved plan. Set the persistent GitHub mode to `dns_only` only after state import and plan review.
3. Recheck Caddy origin HTTPS, admin/API no-store, public document 60-second freshness, and media one-day TTL. Confirm the registrar has no DS record and Cloudflare DNSSEC stays off.

### C. Enable the proxy after state reconciliation

1. Nameservers already point to Cloudflare; verify both authoritative answers, mail, and `autographs` while it remains DNS-only. Record results before enabling proxying.
2. Wait for Cloudflare Universal SSL to become active for `autographs.jetsaredim.net`; confirm Full (strict) and cache rules. Do not proxy before these checks pass.
3. Advance the persistent mode from `dns_only` to `proxied`. Review and run the controlled operator DNS plan/apply, then verify Cloudflare answers and Caddy origin TLS. A following routine deploy plan must show no DNS changes and retain `proxied = true`.
4. Verify `/admin/` no-store, two authenticated 200 `GET /admin/api/status` responses with no-store and no edge HIT, and Cloudflare Trace matching the bypass rule without recording cookies or private bodies. Repeated current fingerprinted media GETs must reach `CF-Cache-Status: HIT`; public HTML/JSON must remain rollback-friendly.
5. Record rollback evidence. First rollback is a reviewed change of the persistent mode back to `dns_only` with the controlled DNS apply. If Cloudflare authoritative DNS fails, restore the backed-up Porkbun zone and registrar nameservers, allowing for propagation.

### D. Retire Porkbun Terraform ownership

After production verification and a documented rollback window, replace `porkbun_dns_record.autographs` with a Terraform `removed` block using `destroy = false`. Remove its import block and Porkbun-only provider variables/secrets/workflow inputs only after a plan proves state-only removal. Keep the real Porkbun record and zone export for emergency reversal. Update the DNS runbook and Phase 8 summary to the new current state.

## Required gates

- Nameserver delegation has already happened through Porkbun Connect; verify the authoritative record inventory and complete the state import before any further DNS change.
- No routine deployment may apply a DNS mutation; controlled operator DNS applies use the exact reviewed plan.
- No proxy activation until Cloudflare DNS is active, edge/origin TLS is valid, rules are configured, and origin cache checks pass.
- No Phase 8 completion until authenticated API bypass, media HIT, public freshness, and rollback evidence are recorded.
- No Porkbun resource retirement until the Cloudflare path passes production verification and the plan shows state-only removal.
