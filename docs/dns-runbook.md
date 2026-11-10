# DNS Runbook

The current production zone uses Cloudflare authoritative DNS with a proxied
`autographs` A record. Porkbun remains the registrar and retains its older DNS
record as a rollback copy. OCI public DNS zones are intentionally not managed
by Terraform because OCI public DNS is not available in the current free-tier
setup.

Wave 8 prepares the existing runtime Terraform root for the Cloudflare zone,
the owner's confirmed three DNS records, strict TLS, and cache rules. Porkbun
Connect already copied the records and delegated nameservers to Cloudflare.
The Cloudflare zone, three records, and SSL
setting were imported into the existing Terraform state by the
[import-only run](https://github.com/jetsaredim/autographs/actions/runs/42689411270).
The persistent GitHub `CLOUDFLARE_MODE` is now `proxied` (the Terraform input
defaults to `off` when unset); the reviewed sequence is `off` → `dns_only` →
`proxied`. Routine deploys inspect
a saved Terraform plan and refuse Cloudflare or Porkbun DNS changes. A controlled
operator apply uses the exact reviewed plan file; follow the
[Wave 8 migration sequence](../.planning/phases/08-admin-media-review-and-operational-posture/08-08-CLOUDFLARE-MIGRATION.md)
before any authoritative DNS change.
The Cloudflare `autographs` A record uses the confirmed 600-second TTL while
DNS-only and Cloudflare's automatic TTL when proxied.

The [manual Cloudflare reconciliation workflow](../.github/workflows/cloudflare-import.yml)
checked the active zone and exact three-record inventory, then imported the
zone, A/MX/TXT records, and SSL setting into OCI-backed state. No cache
ruleset existed to import. The initial import run did not apply DNS changes.
After aligning Terraform's SPF representation with Cloudflare's API, the
[verification run](https://github.com/jetsaredim/autographs/actions/runs/42693102566)
proposed only the cache ruleset creation and SSL update from `full` to
`strict`: one add, one change, zero destroy. The workflow's default
`verify_only` operation never applies. Its `apply_tls_and_cache` operation
requires the persistent mode to remain `dns_only`, verifies the live zone and
three-record inventory, checks the exact Full (strict) value and three cache
rule behaviors, accepts exactly those two Terraform actions, then
applies the saved plan from the same run. Any different plan stops without an
apply. Routine deploys still refuse DNS changes.

The [guarded proxy cutover](https://github.com/jetsaredim/autographs/actions/runs/42748453909)
passed active edge certificate and origin HTTPS checks, then updated only the
existing app A record. The
[post-cutover verification](https://github.com/jetsaredim/autographs/actions/runs/42748759564)
confirmed the three-record inventory and a no-change Terraform plan. Both
Cloudflare authoritative nameservers return edge addresses. Repeated GETs for
the adjusted WebP returned `CF-Cache-Status: MISS` then `HIT` while retaining
`Cache-Control: public, max-age=86400`; `/admin/` returned `no-store` and
`DYNAMIC`, and public HTML/JSON retained 60-second origin freshness.

For a future proxy cutover, set the persistent GitHub `CLOUDFLARE_MODE` variable to
`proxied`, then dispatch this workflow on `main` with `operation=apply_proxy`
and the confirmed zone ID. It requires the old DNS-only A record, an active
Cloudflare edge certificate covering `autographs.jetsaredim.net`, and valid
Caddy origin HTTPS. It applies only a saved plan that changes the one A record
from DNS-only TTL 600 to proxied automatic TTL. Follow with `verify_only` and
the [CDN cache probes](cdn-cache-contract.md). For first rollback, set the
persistent mode to `dns_only` and dispatch `operation=apply_dns_only`; its
saved-plan guard accepts only the reverse A-record change. Leave MX and SPF
untouched in both operations. The Cloudflare token needs SSL and Certificates
Read permission for the edge certificate preflight.

## DNS Model

The proxied `A` record for the app hostname still targets the runtime VM IP
inside Cloudflare; public DNS returns Cloudflare edge addresses:

```text
autographs.jetsaredim.net -> runtime_public_ip
```

The runtime VM public IP is produced by Terraform and currently also mirrored in
the `VM_PUBLIC_IP` GitHub Variable for deploy fallback behavior.

## Proxied A record and Porkbun rollback copy

Cloudflare's authoritative `jetsaredim.net` zone currently has:

```text
Type: A
Host: autographs
Origin address: <runtime_public_ip>
Proxy: enabled
TTL: automatic
```

Porkbun retains the former DNS record as a rollback copy; editing it does not
change live DNS while Cloudflare nameservers are delegated. Use the current
Terraform output to check the Cloudflare A-record address:

```bash
terraform -chdir=infra/terraform output -raw runtime_public_ip
```

Or check the GitHub Variable fallback:

```bash
gh variable list --repo jetsaredim/autographs --json name,value --jq '.[] | select(.name == "VM_PUBLIC_IP") | .value'
```

## TLS

The deployed edge container uses Caddy, which automatically obtains and renews a
Let's Encrypt certificate for `autographs.jetsaredim.net`. Certificate issuance
requires all of the following before the deploy runs:

- the Cloudflare A record targets the runtime VM public IP and the edge can reach its HTTPS origin
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

After Cloudflare proxy activation, verify DNS and the public HTTPS edge:

```bash
dig NS jetsaredim.net
dig A autographs.jetsaredim.net
curl --fail --silent --show-error --head https://autographs.jetsaredim.net/
curl --fail --silent --show-error --head https://autographs.jetsaredim.net/admin/
curl --fail --silent --show-error --head --resolve autographs.jetsaredim.net:443:129.213.166.159 https://autographs.jetsaredim.net/
```

Expect Cloudflare nameservers and edge A addresses from public DNS, HTTP 200
for the public and admin paths, and `Cache-Control: no-store` for `/admin/`.
The `--resolve` request checks the Caddy origin certificate and response
directly while retaining the hostname. The proxied Cloudflare record uses
automatic TTL; recursive resolvers may briefly retain the former DNS-only
answer during propagation. Confirm edge media caching with repeated GETs and
`CF-Cache-Status: HIT` as described above.
