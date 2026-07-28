# DNS Runbook

This project uses Porkbun DNS directly for the public app hostname. OCI public
DNS zones are intentionally not managed by Terraform because OCI public DNS is
not available in the current free-tier setup.

## DNS Model

Keep `jetsaredim.net` hosted at Porkbun and create a direct `A` record for the
app hostname:

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
TTL: 300
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

Cloudflare enablement checklist:

1. Move DNS for `autographs.jetsaredim.net` behind a proxied Cloudflare record.
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

1. Set the Cloudflare DNS record to DNS-only, or move the hostname back to the
   direct Porkbun `A` record.
2. Purge the affected public URLs or the hostname cache before/after the DNS
   change.
3. Verify origin behavior directly with `curl --resolve` against the runtime VM
   IP and then through public DNS.

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
low TTL such as 300 seconds should make routine updates settle quickly.
