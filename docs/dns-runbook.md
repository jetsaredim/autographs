# DNS Runbook

The current production zone uses Cloudflare authoritative DNS with a DNS-only
`autographs` A record. Porkbun remains the registrar and retains its older DNS
record as a rollback copy. OCI public DNS zones are intentionally not managed
by Terraform because OCI public DNS is not available in the current free-tier
setup.

Wave 8 prepares the existing runtime Terraform root for the Cloudflare zone,
the owner's confirmed three DNS records, strict TLS, and cache rules. Porkbun
Connect already copied the records and delegated nameservers to Cloudflare.
The A record remains DNS-only; the Cloudflare resources are not yet imported
into Terraform state. `CLOUDFLARE_MODE` defaults to `off`; the reviewed
sequence is `off` → `dns_only` → `proxied`. Routine deploys inspect
a saved Terraform plan and refuse Cloudflare or Porkbun DNS changes. A controlled
operator apply uses the exact reviewed plan file; follow the
[Wave 8 migration sequence](../.planning/phases/08-admin-media-review-and-operational-posture/08-08-CLOUDFLARE-MIGRATION.md)
before any authoritative DNS change.
The Cloudflare `autographs` A record uses the confirmed 600-second TTL while
DNS-only and Cloudflare's automatic TTL when proxied.

Before setting the persistent GitHub `CLOUDFLARE_MODE=dns_only`, run the
manual [Cloudflare import workflow](../.github/workflows/cloudflare-import.yml)
on `main` with the zone ID from Cloudflare Overview. It checks the active zone
and exact three-record inventory, then imports the zone, A/MX/TXT records, the
SSL zone setting, and any existing cache ruleset into the existing OCI-backed
Terraform state. It prints a sanitized plan summary and does not apply the
plan or change DNS. Review any proposed changes before a separate controlled
operator plan/apply. A routine deploy with `dns_only` before import would see
the live resources as new; its DNS guard must reject that plan.

## DNS Model

Until CDN proxy activation, keep the direct DNS-only `A` record for the app
hostname:

```text
autographs.jetsaredim.net -> runtime_public_ip
```

The runtime VM public IP is produced by Terraform and currently also mirrored in
the `VM_PUBLIC_IP` GitHub Variable for deploy fallback behavior.

## DNS-only A record and Porkbun rollback copy

Cloudflare's authoritative `jetsaredim.net` zone currently serves:

```text
Type: A
Host: autographs
Answer: <runtime_public_ip>
TTL: 600
```

Porkbun retains the former DNS record as a rollback copy; editing it does not
change live DNS while Cloudflare nameservers are delegated. Use the current
Terraform output to check the Cloudflare A-record address:

```bash
terraform -chdir=infra/terraform output -raw runtime_public_ip
```

Or check the GitHub Variable fallback:

```bash
gh variable get VM_PUBLIC_IP --repo jetsaredim/autographs
```

## TLS

The deployed edge container uses Caddy, which automatically obtains and renews a
Let's Encrypt certificate for `autographs.jetsaredim.net`. Certificate issuance
requires all of the following before the deploy runs:

- the authoritative Cloudflare DNS-only `A` record resolves to the runtime VM public IP
- OCI ingress allows ports 80 and 443
- the VM firewall allows HTTP and HTTPS

OCI NSG and VM firewall rules are already managed for ports 80 and 443 by the
committed Terraform and bootstrap scripts.

## Cloudflare/CDN Decision

Phase 6 reviewed Cloudflare as an optional CDN front door and left the direct
Porkbun-to-Caddy path in place while the site was small. Phase 8 makes
CDN/cache implementation explicit:
define the cache contract before admin media adjustment work, then enable and
verify production CDN after adjusted image derivatives produce new
fingerprinted `/media/*` paths and can be tested end to end.

On 2026-11-06, Porkbun was authoritative. Porkbun Connect has since copied the
zone into the owner's Cloudflare account and delegated nameservers. The
Cloudflare DNS-only A, MX, and SPF records were checked against authoritative
answers; the unused apex ACME challenge TXT records were removed. Keep the
Porkbun zone backup until the CDN cutover and rollback window are complete.

Cloudflare enablement checklist:

1. Import the live Cloudflare zone and records into Terraform and review the
   `dns_only` plan. Apply the strict TLS and cache rules through a separate
   controlled operator plan, then proxy only the `autographs.jetsaredim.net`
   record. Verify nameservers, DNS answers, and the production site.
2. Use Cloudflare SSL/TLS Full (strict) so Cloudflare validates the Caddy origin
   certificate; Cloudflare documents that Full (strict) requires an unexpired
   origin certificate whose hostname matches the request.
3. Add Cache Rules only for anonymous public paths. Cloudflare Cache Rules can
   control eligibility and TTL, and Cloudflare documents bypassing cache when a
   request cookie matches a rule.
4. Bypass caching for `/admin`, `/admin/*`, `/admin/api/*`, and any request with
   the admin session cookie. Admin shell/API responses must remain origin-only
   and `Cache-Control: no-store`.
5. Keep HTML, JSON, and `manifest.json` short-lived or origin-controlled so a
   rollback to a previous static release is visible quickly.
6. Cache `/media/*` at the edge with the origin's one-day TTL to reduce image
   traffic against the OCI VM. Generated derivative URLs include a 16-hex
   content fingerprint, for example
   `/media/<item-slug>/<image-slug>-detail-<fingerprint>.webp`, so replacement
   and rollback publish paths matching the derivative bytes instead of reusing
   the same public URL for different content.
7. Document purge access before turning on proxying. Cloudflare single-file
   purge removes a cached URL from the CDN and the next request re-fetches it
   from origin; reserve purge for emergency takedown, accidental public
   exposure, or CDN incident response and use exact UTF-8 URLs for single-file
   purge.

Rollback if Cloudflare causes stale public content or admin issues:

1. For a full-zone setup, set the Cloudflare `autographs` record to DNS-only
   first; do not assume a registrar nameserver reversal is immediate. For a
   supported partial setup, restore the prior Porkbun hostname record.
   If full-zone migration must be reversed, restore the backed-up Porkbun zone
   and registrar nameservers, then allow for DNS propagation.
2. Purge the affected public URLs or the hostname cache before/after the DNS
   change.
3. Verify origin behavior directly with `curl --resolve` against the runtime VM
   IP and then through public DNS.

## Phase 8 CDN/cache contract

The detailed Phase 8 CDN/cache rule, purge, rollback, and verification contract
lives in [cdn-cache-contract.md](cdn-cache-contract.md). Use that contract when
creating Cloudflare rules named `Bypass admin and API`,
`Respect rollback-sensitive public documents`, and
`Cache fingerprinted media and assets`. Production CDN enablement happens after
adjusted-media cache behavior is proven through the contract probes, not before
the admin media adjustment and publisher cache-key work exists.

For `cdn_admin_bypass_evidence` and `cdn_media_cache_evidence`, capture
post-proxy response headers after the cache rules are active. The complete
Phase 8 production verification record, including
`adjusted_media_publish_evidence` and `rollback_evidence`, is defined in
[cdn-cache-contract.md](cdn-cache-contract.md). An origin-only response is not
proof that Cloudflare bypasses admin traffic or caches fingerprinted media.
For admin bypass, include an authenticated 200 API response and Cloudflare
Rule Trace; an unauthenticated 401 alone is insufficient.

References checked during this decision:

- <https://developers.cloudflare.com/cache/how-to/cache-rules/>
- <https://developers.cloudflare.com/cache/how-to/cache-rules/examples/bypass-cache-on-cookie/>
- <https://developers.cloudflare.com/cache/how-to/purge-cache/purge-by-single-file/>
- <https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full-strict/>

## Verification

After Cloudflare DNS is delegated, verify DNS and the public HTTPS origin:

```bash
dig NS jetsaredim.net
dig A autographs.jetsaredim.net
curl --fail --silent --show-error --head https://autographs.jetsaredim.net/
curl --fail --silent --show-error --head https://autographs.jetsaredim.net/admin/
```

Expect Cloudflare nameservers, the runtime VM A address, HTTP 200 for both
paths, and `Cache-Control: no-store` for `/admin/`. These DNS-only responses
do not establish CDN cache behavior. DNS changes can take time to propagate
through recursive resolver caches; the A record has a 600-second TTL.
