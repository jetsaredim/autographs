---
phase: 08-admin-media-review-and-operational-posture
pr: 294
review_round: 2
status: revised_assumptions
source_review: 08-07-PR-294-REVIEW.md
---

# PR 294 public media assumptions

## Shared invariants

1. Candidate media remains private until `current` changes to the validated release. After that atomic pointer change, Caddy can serve new media through `current/media` while shared storage is populated.
2. Previously published fingerprinted URLs remain fetchable for at least the documented 24-hour origin window and while retained releases reference them. Emergency takedown deliberately overrides this guarantee for exact affected URLs.
3. An emergency-removed URL stays denied at the origin even if a shared file, retained release, rollback target, or derivative cache still has its bytes. A future candidate that references the same URL fails before promotion.
4. Public dimensions match verified WebP bytes. Round 1's dimension fix is complete and must remain intact.
5. Deployment of the Caddy route and publisher is not atomic. Publication must be frozen while the old publisher can still run its pre-promotion shared copy; resume only after the revised publisher and both Caddy origin routes are confirmed active. A local controlled callback plus real disposable Caddy HTTP fixture proves the precommit boundary; deployed smoke proves the postcommit path without requiring a production pause hook.

## Producer and consumer inventory

| Boundary | Producer, consumer, or action | Evidence |
| --- | --- | --- |
| Private source to candidate | `build_public_items` generates adjusted WebP, fingerprints paths, writes JSON/HTML; `validate_candidate` checks artifact inventory and privacy before promotion. | `controller/src/publisher.rs` |
| Public commit | `publish_locked` prepares media and `promote_candidate` atomically changes `current`. Current pre-promotion copy exposes candidate bytes through Caddy's shared root. | `controller/src/publisher.rs:872`, `:2047`; `deploy/ansible/roles/autographs_deploy/files/Caddyfile` |
| Normal serving | Caddy serves shared `/media/*`, falling back to `current/media`; HTML/JSON freshness is 60 seconds and media freshness is 24 hours. Both public and loopback sites use this route. | `Caddyfile`; `docs/cdn-cache-contract.md` |
| Rollback and retention | Operators can repoint `current`; post-promotion cleanup refreshes demoted media then prunes releases and aged shared files. | `controller/src/publisher.rs:2122`, `:2166`; CDN runbook |
| Emergency removal | Current instructions delete shared and retained copies and purge exact CDN URLs, but manifests and cache can still refer to those paths. Caddy fallback can serve a missed copy. | `docs/cdn-cache-contract.md:91`; `controller/src/publisher.rs:2061` |
| Reporting and tests | Cleanup errors become safe warnings; publisher/Caddy tests prove local filesystem behavior; credentialed Caddy and Cloudflare proof remains a Wave 8 gate. | `controller/src/publisher.rs:2166`; `controller/tests/publisher.rs`; `controller/tests/caddy_static_routes.rs` |

## Failure matrix

| Event | Current effect | Required behavior |
| --- | --- | --- |
| Candidate copy completes while promotion waits or fails | New URL is anonymously readable. | Candidate-only URL remains unavailable until pointer change. |
| Pointer changes before shared copy | New URL exists in `current/media`. | Caddy serves it via fallback; shared copy is retryable without reverting the committed release. |
| Normal correction, unpublish, or rollback | Cached documents can reference old URLs. | Shared/retained media bridges origin misses for the bounded window. |
| Emergency removal deletes files but misses a retained copy | Cleanup or rollback can restore the URL. | Durable exact-path deny wins over every physical copy and prevents future publication. |
| Emergency removal deletes a manifest-listed file | Release pruning fails while rereading it. | Cleanup skips intentionally denied media and still prunes the release. |
| Old publisher runs during Caddy/controller rollout | It can still expose a candidate through shared media. | Block admin publish operations until the revised publisher and Caddy routes pass readiness probes. |
| Marker is written on a different or unreadable mount | Publisher may see it while Caddy still serves the URL, or vice versa. | Confirm controller/Caddy volume identity, marker readback in Caddy, and 404 at both public and preview origin routes before declaring origin denial. |

## Revised decisions

- Move candidate-to-shared copying after the atomic pointer change. Before promotion, migrate only already-active media. Candidate files stay in the private release directory; Caddy's `current/media` fallback covers the narrow post-promotion interval.
- Use durable exact-path tombstone marker files on the shared static volume. Caddy checks for a marker before either media file lookup. The publisher refuses candidates containing a tombstoned path, never recopies such media, and allows pruning a release with intentionally removed media while unrelated missing media still fails closed.
- Provide one operator action that creates the marker before deleting shared and retained copies, then instructs exact CDN purge and origin/edge 404 verification. Marker files persist across publish, rollback, and process restart.
- During rollout, freeze publication before replacing either process, keep the active release served, validate both new Caddy routes and the revised controller, then explicitly resume publication. The operator takedown action is complete only after both origin routes return 404 for the marked URL.
- Keep normal media retention and no-routine-purge semantics. Emergency 404 is an explicit exception.

Official Caddy documentation supports a named `file` matcher with an independent `root` to test whether a marker exists at the requested path; use a `route` block to make denial precede file lookup: [request matcher docs](https://caddyserver.com/docs/caddyfile/matchers#file), [try_files docs](https://caddyserver.com/docs/caddyfile/directives/try_files). Validate with an actual Caddy process when available; source assertions alone are insufficient for Wave 8.

## Alternatives considered

Retiring every affected retained release can remove old copies, but it sacrifices rollback targets and does not prevent the same fingerprinted URL from being generated again. Tombstones provide a durable serving and publishing rule without relying on every retained copy being found manually.
