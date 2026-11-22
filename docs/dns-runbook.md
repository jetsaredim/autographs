# DNS Runbook

The current production zone uses Cloudflare authoritative DNS with a proxied
`autographs` A record. Porkbun remains the registrar and retains its older DNS
record as a rollback copy. OCI public DNS zones are intentionally not managed
by Terraform because OCI public DNS is not available in the current free-tier
setup.

The existing `infra/terraform/` root and OCI state manage the full Cloudflare
zone, the three DNS records, Full (strict) TLS, and cache rules. Porkbun
Connect copied the records and delegated nameservers. The one-time
[state import](https://github.com/jetsaredim/autographs/actions/runs/42689411270)
and [settings apply](https://github.com/jetsaredim/autographs/actions/runs/42738867482)
are complete. The Terraform default for `cloudflare_mode` is `proxied`;
the GitHub variable is optional and normally unset. The Cloudflare app A record
uses automatic TTL. Reviewed Cloudflare and Porkbun DNS changes apply through
the ordinary release deployment from the shared Terraform root and state.

The [guarded proxy cutover](https://github.com/jetsaredim/autographs/actions/runs/42748453909)
passed active edge certificate and origin HTTPS checks, then updated only the
existing app A record. The
[post-cutover verification](https://github.com/jetsaredim/autographs/actions/runs/42748759564)
confirmed the three-record inventory and a no-change Terraform plan. Both
Cloudflare authoritative nameservers return edge addresses. Repeated GETs for
the adjusted WebP returned `CF-Cache-Status: MISS` then `HIT` while retaining
`Cache-Control: public, max-age=86400`; `/admin/` returned `no-store` and
`DYNAMIC`, and public HTML/JSON retained 60-second origin freshness.

## Ongoing Cloudflare changes

Change the Terraform configuration through a reviewed PR. Inspect the
production Terraform plan in CI on both the normal PR and Release PR. CI fails
if either plan contains destructive managed-resource actions until the
`terraform-destructive-plan-approved` label is applied for that PR head. After
the Release PR merges, the
[release and deploy workflow](../.github/workflows/deploy.yml) creates and
applies a fresh saved plan from the same root and state on `main`. There is no
second apply approval gate. The release job prints changed resource addresses
and actions without printing plan values.
Check the production deploy log and verify the affected DNS or cache behavior
after deployment. Unexpected plan actions require stopping the release before
the Release PR merge or investigating the deployment before any retry.

The active `jetsaredim.net` zone and its imported Terraform state remain in
the shared root. A missing state resource requires investigation; routine
deployment does not import it. For DNS-only rollback, set the optional
`CLOUDFLARE_MODE` GitHub variable to `dns_only` before the Release PR CI run,
then review the one-record proxy change before merge. Remove the override
before the next Release PR CI run to restore
the default `proxied` posture through a later reviewed release.
Keep MX and SPF unchanged. The original staged migration and
recovery details remain in the
[Wave 8 migration record](../.planning/phases/08-admin-media-review-and-operational-posture/08-08-CLOUDFLARE-MIGRATION.md).

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
