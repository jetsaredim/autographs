# Cloudflare domain DNS Terraform root

This root prepares a full Cloudflare zone for `jetsaredim.net` in a separate OCI state object (`envs/prod/domain-dns.tfstate`). It is **not** applied by the Autographs deploy workflow. Porkbun remains authoritative until the owner changes registrar nameservers at the reviewed cutover checkpoint.

The root owns the zone, zone settings, cache rules, and every reviewed domain DNS record **except** the `autographs.jetsaredim.net` A record. The runtime Terraform root will own that one Cloudflare A record because its content follows the OCI runtime VM IP. Other record types at the same hostname, if present in Porkbun, belong in the domain root.

Before any apply:

1. Export the complete Porkbun zone from the Porkbun dashboard. Compare it with Cloudflare's scan and add every missing record, including mail and verification records. The required `dns_records` input has no committed value until this inventory is reviewed.
2. Confirm the Cloudflare account ID and place a narrowly scoped API token in the protected environment as `CLOUDFLARE_API_TOKEN`. Do not commit tokens or Terraform state.
3. Initialize this root with the OCI backend coordinates in `backend.hcl.example`, using a protected local copy or explicit `-backend-config` arguments. Do not reuse the runtime root's state key.
4. Review the first plan for duplicate DNS records, unintended changes to unrelated hosts, and unexpected destruction. If Cloudflare onboarding already created the zone or records, import those objects before applying rather than creating duplicates.
5. Keep the registrar nameservers at Porkbun until the whole-zone record comparison passes. The owner keeps DNSSEC disabled and the registrar without a DS record. The activation, proxy, and rollback sequence is in `docs/dns-runbook.md` and the Wave 8 migration artifact.

The cache rules match only `autographs.jetsaredim.net`. Admin routes and rollback-sensitive documents bypass edge cache. Public media and assets remain eligible for caching while respecting Caddy's origin headers. Cloudflare's TLS mode is Full (strict); the existing Caddy public certificate remains the origin certificate.
