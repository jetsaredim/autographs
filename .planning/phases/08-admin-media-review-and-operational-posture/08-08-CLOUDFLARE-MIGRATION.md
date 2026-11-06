---
phase: 08-admin-media-review-and-operational-posture
plan: "08"
status: inventory_pending
approach: terraform_managed_full_zone
---

# Cloudflare DNS and CDN migration for Wave 8

## Decision and boundaries

Use a full-zone Cloudflare setup for `jetsaredim.net` unless the owner reports a Business or Enterprise account and explicitly chooses eligible partial/CNAME setup. Porkbun remains the registrar. Cloudflare becomes authoritative DNS only after every existing Porkbun record has been inventoried and reproduced. Proxy only `autographs.jetsaredim.net`; unrelated hosts and mail records remain DNS-only. The adjusted-media before/after URL gate passed in `08-08-CHECKPOINT.md`.

The current production runtime Terraform root (`infra/terraform`) manages `porkbun_dns_record.autographs` and is applied automatically by `.github/workflows/deploy.yml`. Keep that resource and its import block during preparation. Do not let a routine deploy change nameserver delegation or enable the Cloudflare proxy. Never remove the Porkbun resource block alone: Terraform would plan to destroy the record. Retire it with a reviewed `removed` block and `destroy = false` only after the Cloudflare path has passed the production checkpoint.

## Inputs still required

- Complete Porkbun DNS export or dashboard record inventory, including apex, mail, TXT verification, CAA, delegated subdomains, wildcards, and any service-specific records. Public `dig` and Cloudflare quick scan are incomplete inventories.
- Cloudflare zone plan and account ID. Use a scoped API token through the secret store; never commit it or place it in Terraform state values.
- Current registrar DNSSEC/DS setting, Cloudflare-assigned nameservers, and a known-good rollback copy of the Porkbun zone.
- Confirmation of the DNS cutover window and who will change Porkbun nameservers.

## Staged implementation

### A. Prepare Terraform without production DNS changes

1. Add a dedicated domain DNS Terraform root and OCI backend key separate from the runtime state. Define the Cloudflare full zone and the inventoried domain-wide DNS records there. Keep records DNS-only by default. Use explicit imports if Cloudflare onboarding or scanning already created a zone or records; a Terraform plan must not propose duplicate records or destructive replacements.
2. Add Cloudflare DNS record management for `autographs` to the runtime root, preserving its dependency on `module.compute.public_ip`. Create the Cloudflare record as DNS-only while the Porkbun record remains managed. Gate new provider configuration with explicit variables and scoped credentials so the existing production deploy can continue before Cloudflare setup is ready.
3. Add Terraform-managed Full (strict) TLS and ordered cache rules: admin/API bypass; public HTML/JSON origin-respecting or at most 60-second freshness; fingerprinted media one-day origin TTL and assets one-hour origin TTL. Do not use a sitewide cache-everything or custom cache key. Confirm any overlapping rule uses the intended last-match behavior.
4. Add CI syntax/schema validation for the new root and update operator configuration docs. Keep domain-wide zone applies separate from ordinary app deploys. The runtime root's `autographs` record remains in the existing deploy apply path, so its Cloudflare enablement and proxy flags must default to disabled and be changed only in a reviewed cutover step.

### B. Validate prepared Cloudflare zone

1. Compare the complete Porkbun inventory with Terraform's Cloudflare DNS plan and the resulting Cloudflare zone record list. Verify mail and unrelated hosts are DNS-only and identical in meaning, including MX priority and TXT content.
2. Verify direct origin HTTPS using the current runtime IP and hostname, Caddy's valid certificate, admin/API no-store, public document 60-second freshness, and media one-day TTL.
3. Verify Cloudflare Universal SSL is active for `autographs.jetsaredim.net`. Keep the `autographs` record DNS-only until the edge certificate, Full (strict) mode, and cache rules are ready.
4. If DNSSEC is enabled, disable the old registrar DS record before the nameserver switch; re-enable through Cloudflare after the new delegation is active.

### C. Cut over DNS authority, then proxy

1. Change Porkbun registrar nameservers to the assigned Cloudflare pair. Verify authoritative NS answers, apex and mail records, unrelated hosts, and `autographs` while the record remains DNS-only. Record results before enabling proxying.
2. Through a separately reviewed Terraform plan/apply, set the Cloudflare `autographs` record to proxied. Confirm public DNS answers traverse Cloudflare and Full (strict) connects to Caddy without redirect loops or certificate errors.
3. Verify two authenticated 200 `GET /admin/api/status` responses remain `no-store` with no edge HIT; verify Cloudflare Trace matches the admin bypass rule without recording cookies or private bodies. Verify `/admin/` no-store, public HTML/JSON freshness, and a repeated current fingerprinted media GET reaches `CF-Cache-Status: HIT`.
4. Record rollback evidence. First rollback is the Cloudflare `autographs` record back to DNS-only while Cloudflare remains authoritative. If authoritative DNS itself fails, restore the backed-up Porkbun zone and registrar nameservers, allowing for DNS propagation.

### D. Retire Porkbun Terraform ownership

After a documented verification and rollback window, replace `porkbun_dns_record.autographs` with a Terraform `removed` block using `destroy = false`; remove its import block and Porkbun-only variables, secrets, provider, and workflow inputs. Confirm the plan removes only Terraform state ownership and leaves the actual Porkbun record intact. Keep the Porkbun zone export for emergency reversal. Update the DNS runbook and Phase 8 summary to the new current state.

## Required gates

- No nameserver change until the complete record inventory and Cloudflare plan match.
- No proxy activation until Cloudflare DNS is active, edge and origin TLS are valid, rules are configured, and origin cache checks pass.
- No Phase 8 completion until authenticated API bypass, media HIT, public freshness, and rollback evidence are recorded.
- No Porkbun resource retirement until Cloudflare production verification passes and the plan explicitly shows state-only removal.
