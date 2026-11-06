---
phase: 08-admin-media-review-and-operational-posture
plan: "08"
status: single_root_revision_record_details_pending
approach: terraform_managed_full_zone
---

# Cloudflare DNS and CDN migration for Wave 8

## Decision and ownership

Use the existing `infra/terraform/` runtime root and its current OCI state object for the Cloudflare full zone, every reviewed `jetsaredim.net` DNS record, strict TLS, and cache rules. The owner prefers one Terraform root and state. The existing deploy workflow auto-applies this root, so it must refuse Cloudflare or Porkbun DNS changes unless an explicit controlled DNS-change input is set. Ordinary runtime deployments may proceed only when the DNS plan has no changes.

Porkbun remains the registrar. Cloudflare becomes authoritative only after every Porkbun record is inventoried and reproduced. Proxy only `autographs.jetsaredim.net`; unrelated hosts and mail records remain DNS-only. Keep the current `porkbun_dns_record.autographs` and its import block until Cloudflare has passed the live checkpoint. Both old and new autograph A records may be managed during migration because they are at different DNS providers. After verification, retire the Porkbun resource with a reviewed `removed` block and `destroy = false`, preserving its real record for rollback.

This is a full-zone plan. Partial/CNAME setup would need a separate revised design. The owner confirms DNSSEC is disabled and does not want it enabled. Public `dig DS jetsaredim.net` returned no record on 2026-11-06; keep DNSSEC off and the registrar without a DS record.

The adjusted-media before/after URL gate passed in `08-08-CHECKPOINT.md`. No Cloudflare zone, nameserver, or proxy change has been applied.

## Inputs still required

- The owner reports that Porkbun has exactly three DNS records: an `autographs` A record, an apex MX record, and an apex TXT record. Public DNS on 2026-11-06 returned `autographs.jetsaredim.net A 129.213.166.159`, `jetsaredim.net MX 1 smtp.google.com.`, and `jetsaredim.net TXT "v=spf1 include:_spf.google.com ~all"`. Confirm the exact Porkbun dashboard values, including record TTLs, before treating this as the migration inventory. Keep a copy for rollback.
- Cloudflare account ID and a scoped API token supplied through the protected secret store; never commit the token or place it in Terraform variable values/state.
- Cloudflare-assigned nameservers, a known-good rollback copy of the Porkbun zone, and the owner-selected cutover window.

## Staged implementation

### A. Prepare the existing Terraform root

1. Add the Cloudflare provider to `infra/terraform/`. Define the full zone, all inventoried DNS records, `autographs` A record derived from `module.compute.public_ip`, Full (strict) TLS, and cache rules in this same root. Keep the Cloudflare `autographs` record DNS-only initially. Import any records Cloudflare onboarding already created rather than duplicating them.
2. Keep the Porkbun provider and autograph record during transition. Add explicit Cloudflare account/zone and proxy inputs. The proxy desired state must be persistent and required in CI and deploy; missing inputs fail before apply rather than silently reverting to DNS-only.
3. Add a pre-apply DNS-change guard to `.github/workflows/deploy.yml`: inspect the saved Terraform plan and stop ordinary deploys when Cloudflare or Porkbun DNS resources would change. Only a dedicated reviewed, manually dispatched DNS operation may permit such a plan. The same reviewed plan must be the one applied. A routine deploy after cutover must have a no-change DNS plan and preserve `proxied = true`.
4. Configure admin/API bypass; public HTML/JSON origin-respecting or at most 60-second freshness; fingerprinted media one-day origin TTL and assets one-hour origin TTL. Use disjoint hostname/path matches and verify last-match rule behavior. Avoid sitewide cache-everything and custom cache keys.
5. Update CI provider validation, GitHub secret/variable documentation, and operator runbooks. Do not merge or apply the implementation until the complete inventory, Cloudflare credentials, and reviewed plan are available.

### B. Apply Cloudflare configuration while Porkbun remains authoritative

1. Compare all Porkbun records with the Cloudflare Terraform plan. The plan must show no destructive replacement and preserve mail priorities, TXT content, unrelated hosts, and the DNS-only `autographs` A record.
2. Run the dedicated controlled DNS apply. Confirm Cloudflare zone, records, Full (strict), and cache rules in the dashboard. Keep registrar nameservers at Porkbun, so public traffic still reaches Caddy directly.
3. Verify Caddy origin HTTPS using the current VM IP and hostname, admin/API no-store, public document 60-second freshness, and media one-day TTL. Confirm the registrar remains without a DS record and Cloudflare DNSSEC stays off.

### C. Change nameservers, then enable the proxy

1. Change Porkbun registrar nameservers to Cloudflare's assigned pair. Verify authoritative NS answers, mail, unrelated hosts, and `autographs` while the record remains DNS-only. Record results before enabling proxying.
2. After the full zone is active, wait for Cloudflare Universal SSL to become active for `autographs.jetsaredim.net`; confirm Full (strict) and cache rules. Do not proxy before these checks pass.
3. Set the persistent proxy input to true. Review and run the dedicated DNS plan/apply, then verify Cloudflare answers and Caddy origin TLS. A following routine deploy plan must show no DNS changes and retain `proxied = true`.
4. Verify `/admin/` no-store, two authenticated 200 `GET /admin/api/status` responses with no-store and no edge HIT, and Cloudflare Trace matching the bypass rule without recording cookies or private bodies. Repeated current fingerprinted media GETs must reach `CF-Cache-Status: HIT`; public HTML/JSON must remain rollback-friendly.
5. Record rollback evidence. First rollback is a reviewed change of the persistent proxy input back to DNS-only with the dedicated DNS apply. If Cloudflare authoritative DNS fails, restore the backed-up Porkbun zone and registrar nameservers, allowing for propagation.

### D. Retire Porkbun Terraform ownership

After production verification and a documented rollback window, replace `porkbun_dns_record.autographs` with a Terraform `removed` block using `destroy = false`. Remove its import block and Porkbun-only provider variables/secrets/workflow inputs only after a plan proves state-only removal. Keep the real Porkbun record and zone export for emergency reversal. Update the DNS runbook and Phase 8 summary to the new current state.

## Required gates

- No nameserver change until complete record inventory and Cloudflare plan match.
- No routine deployment may apply a DNS mutation; controlled DNS applies use the exact reviewed plan.
- No proxy activation until Cloudflare DNS is active, edge/origin TLS is valid, rules are configured, and origin cache checks pass.
- No Phase 8 completion until authenticated API bypass, media HIT, public freshness, and rollback evidence are recorded.
- No Porkbun resource retirement until the Cloudflare path passes production verification and the plan shows state-only removal.
