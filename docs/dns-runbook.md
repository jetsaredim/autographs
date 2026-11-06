# DNS Runbook

The current production zone uses Porkbun authoritative DNS and a direct
`autographs` A record. OCI public DNS zones are intentionally not managed by
Terraform because OCI public DNS is not available in the current free-tier
setup. Moving to Cloudflare proxying requires a deliberate DNS topology choice.

Wave 8 will use the existing runtime Terraform root for the Cloudflare zone,
reviewed DNS records, strict TLS, and cache rules. The current Porkbun record
and registrar nameservers remain active during preparation. Routine deploys
must refuse unapproved DNS plan changes; follow the
[Wave 8 migration sequence](../.planning/phases/08-admin-media-review-and-operational-posture/08-08-CLOUDFLARE-MIGRATION.md)
before any authoritative DNS change.

## DNS Model

Until CDN migration, keep `jetsaredim.net` hosted at Porkbun and use a direct
`A` record for the app hostname:

```text
autographs.jetsaredim.net -> runtime_public_ip
```

The runtime VM public IP is produced by Terraform and currently also mirrored in
the `VM_PUBLIC_IP` GitHub Variable for deploy fallback behavior.

## Porkbun Record

In Porkbun DNS for `jetsaredim.net`, create or update this record:

```text
Type: A
Host: autographs
Answer: <runtime_public_ip>
TTL: 600
```

Use the current Terraform output for the address:

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

- the Porkbun `A` record resolves to the runtime VM public IP
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

On 2026-11-06, `dig NS jetsaredim.net` returned Porkbun nameservers. A
proxied Cloudflare record cannot be activated merely by adding the hostname
to a Cloudflare dashboard while Porkbun remains authoritative on a Free or Pro
full-zone setup. Choose one supported setup before changing production DNS:

- **Full-zone setup:** Inventory and back up every `jetsaredim.net` DNS record,
  recreate them in Cloudflare, check mail and other service records, then change
  the registrar nameservers at Porkbun to Cloudflare's assigned nameservers.
  Keep unrelated records DNS-only unless their owners approve proxying.
- **Partial/CNAME setup:** Use this only if the Cloudflare account plan supports
  it for the hostname. Keep Porkbun authoritative, follow Cloudflare's
  partial-zone validation and CNAME instructions, and verify the resulting
  hostname actually traverses Cloudflare. Do not assume this option is
  available on a Free or Pro account.

Cloudflare documents the supported zone setups at
<https://developers.cloudflare.com/dns/zone-setups/>. The operator must
choose and verify one route before the Phase 8 CDN checkpoint can pass.

Cloudflare enablement checklist:

1. Complete the chosen supported zone setup above, then proxy only the
   `autographs.jetsaredim.net` record. Verify `dig NS jetsaredim.net`, the
   hostname's DNS answers, and the production site before changing cache rules.
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

After Porkbun is updated and the deploy workflow has completed, verify DNS,
HTTP, and HTTPS:

```bash
dig A autographs.jetsaredim.net
curl --fail --silent http://autographs.jetsaredim.net/health
curl --fail --silent https://autographs.jetsaredim.net/health
```

Expected response:

```json
{"ok":true,"service":"autographs","scope":"proof-of-life"}
```

DNS changes can take time to propagate through recursive resolver caches, but a
The provider-enforced 600-second TTL should make routine updates settle quickly.
