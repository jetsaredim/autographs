---
phase: 08-admin-media-review-and-operational-posture
plan: "08"
task: 3
status: awaiting_human_action
observed_at: 2026-11-06T14:45:56Z
---

# Wave 8 production CDN checkpoint

## Earlier public baseline

The public item JSON at `/data/items/5d6-ra-7-fivedesix.json` currently references these two variants for image 1:

| Variant | Public path | SHA-256 of served bytes |
|---|---|---|
| Thumbnail | `/media/5d6-ra-7-fivedesix/image-1-thumbnail-026b3952b058f219.webp` | `026b3952b058f2190cfe1196792f6420735f93d145a2778b9c42abf469f93be6` |
| Detail | `/media/5d6-ra-7-fivedesix/image-1-detail-70c219182f1ace41.webp` | `70c219182f1ace4167b0c501753e4c2421779df15a7932c92d46e9bb0d130d72` |

Both 16-character filename fingerprints match the first 16 hex characters of their served-byte SHA-256 digests. This item was an initial candidate; the owner selected `5d6-ra-7` below for the actual adjustment test.

## Adjusted-media publish proof: `5d6-ra-7`

At 2026-11-06 15:09 UTC, `/data/items/5d6-ra-7.json` referenced image 1 with these public variants, and both URLs returned 200 with `Cache-Control: public, max-age=86400`:

| Variant | Before public path | SHA-256 of served bytes |
|---|---|---|
| Thumbnail | `/media/5d6-ra-7/image-1-thumbnail-d63126d844b4bbcb.webp` | `d63126d844b4bbcb62e8363981326d5467029fd2fef9d02f5dc0bea0d3ebfe57` |
| Detail | `/media/5d6-ra-7/image-1-detail-913001cc8d9fa261.webp` | `913001cc8d9fa2614069bc6a840f42420ef2076a6351c7b2b9d8b4a24584f9f1` |

After the owner's adjustment and publish, the same public item JSON referenced these changed variants at 2026-11-06 15:38 UTC:

| Variant | After public path | SHA-256 of served bytes |
|---|---|---|
| Thumbnail | `/media/5d6-ra-7/image-1-thumbnail-89079abbdcfc1574.webp` | `89079abbdcfc1574c58362f4c064382ab0cf104e8a1318894ef1abced9f327c4` |
| Detail | `/media/5d6-ra-7/image-1-detail-0ddf88ebddaf88e8.webp` | `0ddf88ebddaf88e89a42b7bf740e44b9cb1a1ad71b4717399ebeb5b0d000cfcb` |

Both new URLs returned 200 with `Cache-Control: public, max-age=86400`; the hashes match their new filename fingerprints. The before and after fingerprints differ for both variants. The public detail HTML references the new detail URL, and `/data/collection.json` references the new thumbnail URL. Both old URLs still returned 200 at the after observation. This proves immediate retained-old-path availability, not the full retention duration.

## Origin and DNS observations

On 2026-11-06, public responses showed `/admin/` 200 with `Cache-Control: no-store`, unauthenticated `/admin/api/status` 401 with `no-store`, `/data/collection.json` 200 with `public, max-age=60, must-revalidate`, and the current detail WebP 200 with `public, max-age=86400`. The public responses showed `server: Caddy` and no `CF-Cache-Status` header. `dig NS jetsaredim.net` returned four Porkbun nameservers. These are origin observations and do not prove Cloudflare participation or authenticated API bypass.

## Remaining gate

1. Choose a supported Cloudflare full-zone migration or eligible partial/CNAME setup. Configure the documented bypass and public cache rules, then enable proxying. The adjusted-media and origin pre-proxy checks above passed.
2. Capture post-proxy `/admin/` no-store, two authenticated 200 `GET /admin/api/status` responses with no-store and no edge HIT, and a matching Cloudflare Rule Trace without cookies or private bodies.
3. Capture repeated current media requests reaching `CF-Cache-Status: HIT`, public document freshness, and rollback evidence or a precise reason a rollback exercise could not be safely run.

Do not close `08-08-SUMMARY.md` or Phase 8 until the human production gate is satisfied.
