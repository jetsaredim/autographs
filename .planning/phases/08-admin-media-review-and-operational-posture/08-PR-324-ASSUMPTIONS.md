---
phase: 08-admin-media-review-and-operational-posture
pr: 324
status: proposed_for_review
---

# PR #324 corrected assumptions

These assumptions govern the Wave 8 documentation correction after the third review.

| ID | Corrected assumption | Evidence | Consequence |
|---|---|---|---|
| A-01 | `/admin/` is static shell content served by Caddy without a session gate; it has `Cache-Control: no-store`. | `deploy/ansible/roles/autographs_deploy/files/Caddyfile` `/admin/*` file-server handler; `controller/tests/caddy_static_routes.rs`. | Do not describe shell delivery as authenticated. Cloudflare must still bypass it. |
| A-02 | `/admin/api/*` is reverse proxied to the controller and the private data/action routes require a valid admin session. | Caddy API handler; `controller/src/routes.rs`; `controller/tests/auth_and_health.rs`. | Verify authenticated 200 bypass, not just anonymous 401. |
| A-03 | Current production DNS is Porkbun authoritative. Cloudflare edge cache cannot be observed before a supported proxy setup is active. | `dig NS jetsaredim.net` on 2026-11-06; live Caddy-only response headers; Cloudflare zone setup documentation linked from `docs/dns-runbook.md`. | Pre-proxy checks are origin-only; edge checks follow full-zone or eligible partial setup. |
| A-04 | The public media fingerprint is the first 16 hex characters of the SHA-256 of final WebP bytes. A current URL/hash match does not prove a changed URL. | `controller/src/publisher.rs::public_derivative_fingerprint`; current public detail JSON and served bytes. | Capture old/new URLs for one same-image adjustment publish before proxy enablement. |
| A-05 | Edge participation needs a demonstrated cache HIT for a repeated fingerprinted media request. | Cloudflare cache status contract in `docs/cdn-cache-contract.md` and round-one review. | Repeated MISS/BYPASS cannot close `cdn_media_cache_evidence`. |
| A-06 | A real production apply run is separate from a dry-run exercise. | GitHub Actions run 42042147529 succeeded; run 41503635270 proves a successful scanner playbook. | Record exact run IDs and do not relabel real apply or local smoke as dry-run/live smoke. |

The phase retains its original scope: no UI changes, no new Cloudflare implementation code, and no claim that the human CDN checkpoint has passed.
