# CDN Cache Contract

Phase 8 defines this contract before admin media adjustment work begins. Caddy
remains the origin authority for cache headers, and Cloudflare must respect
those headers while keeping private admin traffic out of CDN cache.

## Origin headers

The deployed Caddyfile owns the origin `Cache-Control` posture:

| Path | Origin header | Contract |
|------|---------------|----------|
| `/admin` | `Cache-Control: no-store` | Redirects to `/admin/` without being stored. |
| `/admin/*` | `Cache-Control: no-store` | Serves the private admin shell only through the authenticated admin boundary. |
| `/admin/api/*` | `Cache-Control: no-store` | Proxies same-origin private controller API calls and must never be cached. |
| `/media/*` | `Cache-Control: public, max-age=86400` | Serves generated public-safe WebP derivatives with fingerprinted URLs. |
| `/assets/*` | `Cache-Control: public, max-age=3600` | Serves generated public assets with origin freshness respected. |
| `/favicon.ico`, `/icon.png` | `Cache-Control: public, max-age=3600` | Serves small static assets with origin freshness respected. |
| `/`, `/index.html`, `/404.html`, `/collection/*`, `/items/*`, `/architecture/*`, `/data/*`, `/manifest.json` | `Cache-Control: public, max-age=60, must-revalidate` | Keeps public documents and JSON rollback-friendly. |

The public edge must not expose Object Storage URLs, bucket names, object keys,
private image UUIDs, or original filenames. Public media is validated generated
output. Caddy serves `/media/*` from the shared static media directory, with a
fallback to `current/media` for the active release. Before either lookup, both
origin listeners deny exact URLs with a durable `.media-tombstones` marker.

## Cloudflare cache rules

Configure Cloudflare rules in this order when production CDN proxying is
enabled:

1. `Bypass admin and API`
   - Match `/admin*` and `/admin/api/*`.
   - Bypass cache and respect the origin `Cache-Control: no-store` response.
   - Also bypass requests carrying the admin session cookie if Cloudflare cookie
     matching is available in the account plan.
2. `Respect rollback-sensitive public documents`
   - Match `/`, `/index.html`, `/404.html`, `/collection/*`, `/items/*`,
     `/architecture/*`, `/data/*`, and `/manifest.json`.
   - Respect origin cache headers or cap edge/browser freshness at 60 seconds.
   - Keep rollback visible quickly after the `current` static release points to
     a prior release.
3. `Cache fingerprinted media and assets`
   - Match `/media/*` and `/assets/*`.
   - Respect origin cache headers.
   - Treat `/media/*` paths as immutable for routine operation because the
     filename includes a content fingerprint.

Do not enable a cache-everything rule that includes `/admin*`, `/admin/api/*`,
or HTML/JSON paths without the rule order above.

## Routine image adjustment cache behavior

Routine image corrections must create new generated WebP bytes and new
fingerprinted media URLs such as
`/media/<item-slug>/<image-slug>-detail-<fingerprint>.webp`. A saved crop,
rotation, pan, deskew, or perspective correction must become part of the
derivative cache key before publication.

Normal adjustment publishes therefore use a new `/media/*-{fingerprint}.webp`
URL instead of requiring manual purge. Existing cached derivatives can expire
naturally because public HTML/JSON with the new media reference remains
rollback-friendly at 60 seconds.

Before moving `current`, the publisher copies only already active media into
the shared namespace and rejects candidate paths with emergency markers. The
atomic `current` pointer change is the public visibility commit. Caddy serves
new active media through its `current/media` fallback immediately, then the
publisher copies that committed release into shared media. A postcommit copy
failure keeps the active release and reports a safe cleanup warning; repair
and verify its shared media before journaled static rollback. The controller
fsyncs a private `.media-commit-history/intent.json` before pointer replacement
and finalizes a versioned `history.json` receipt afterward. An interrupted
intent is reconciled from the pointer before another publish or deploy audit;
intent alone never proves public commitment. A postcommit history-write failure
keeps the active release and blocks further publish or prune until recovery.
Each receipt binds a release manifest digest, exact media paths and full byte
hashes, first commit time, and last-active end time. Later deployment audits
use that history to preserve A after B replaces it. Existing shared
files must have identical bytes for their fingerprinted URL. The publisher retains shared URLs
while a **committed** retained release references them, and for at least 24 hours after
their last active publication. Cleanup runs after successful promotion; the
normal release count remains five. This covers a cache miss for a still-fresh
document during correction, unpublish, and rollback. The active release's
manifest remains the inventory of currently published media; shared media is
only a compatibility serving namespace and is not included in candidate
manifests or public artifact scans.

## Purge and rollback

Rollback uses a journaled static release pointer transition, not broad cache deletion:

1. Freeze publishing and stop the controller. Use the protected
   `static_release_rollback.py` helper to verify the target's committed receipt,
   media bytes, and markers, then atomically repoint `current` and finalize the
   transition history. Direct symlink changes are refused by the next audit.
2. Keep public HTML, JSON, and `manifest.json` freshness at 60 seconds or
   origin-respecting freshness so browsers and Cloudflare converge quickly.
3. Confirm that public pages now reference the media fingerprints from the
   restored release and both origin routes serve unmarked active and retained
   old URLs through an origin miss before unfreezing.

Purge is reserved for emergency remediation: accidental public exposure, legal
or privacy takedown, CDN incident response, or a verified stale `/media/*`
object that cannot wait for TTL expiry. Use exact URL purge where possible.
Purge `/media/*` broadly only when emergency remediation requires it; routine
image adjustments publish new fingerprints and should not need manual purge.
Normal unpublish removes pages and JSON immediately at the origin, but a known
old media URL remains available during the retention window and may persist in
browser or edge caches for its 24-hour TTL. For an urgent takedown, publish the
unpublish first when possible, then create a durable exact-path marker and
prove that both Caddy origin routes return 404 before deleting origin copies.
Remove the affected derivative from shared media and every retained release,
then purge the exact CDN URL and verify edge 404. A missed copy cannot bypass
the marker; release cleanup skips marked files and never restores them. Future
candidates referencing the URL fail before promotion. Do not remove a marker
for rollback. Reinstating content requires a separate decision and new
fingerprint. This intentional emergency 404 overrides cached-document
availability. Follow the volume-identity, marker-readback, and dual-route
steps in [static-runtime-runbook.md](static-runtime-runbook.md).

## Verification

Production CDN enablement waits until adjusted-media cache behavior is proven by
publishing an adjusted image and observing that the public HTML/JSON points to a
new fingerprinted `/media/*` URL. Run these probes before and after enabling
Cloudflare proxying:

```bash
curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/"
curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/api/status"
curl -I "https://${AUTOGRAPHS_DOMAIN}/data/collection.json"
curl -I "https://${AUTOGRAPHS_DOMAIN}/media/...webp"
curl -I "https://${AUTOGRAPHS_DOMAIN}/media/...webp"
```

Expected behavior:

- `/admin/` and `/admin/api/status` return `Cache-Control: no-store`.
- `/data/collection.json` returns
  `Cache-Control: public, max-age=60, must-revalidate`.
- `/media/...webp` returns `Cache-Control: public, max-age=86400`.
- After Cloudflare proxying is enabled, inspect `CF-Cache-Status` on the two
  repeated `/media/...webp` requests. The first request may miss or revalidate;
  the second should demonstrate the edge behavior for the fingerprinted
  derivative while preserving the origin `Cache-Control` header.

Rollback behavior stays the same with or without Cloudflare: restore the prior
static release, keep HTML/JSON freshness at 60 seconds, and purge `/media/*`
only for emergency remediation because normal adjustment publishes produce new
fingerprints.
