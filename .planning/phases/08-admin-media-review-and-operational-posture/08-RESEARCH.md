# Phase 08: admin-media-review-and-operational-posture - Research

**Researched:** 2026-07-30
**Domain:** Rust static publisher, static admin media workflow, production security patching, Caddy/Cloudflare cache posture
**Confidence:** HIGH for codebase responsibilities and locked scope; MEDIUM for external CDN/package recommendations where live account state was not inspected

## User Constraints (from CONTEXT.md)

The following locked decisions and deferred scope are copied verbatim from `.planning/phases/08-admin-media-review-and-operational-posture/08-CONTEXT.md`. [VERIFIED: codebase grep]

### Locked Decisions

### Phase Sequencing
- **D-08-01:** Repair production security patching first, then run aggressive
  repo hygiene cleanup in separate PRs, then define the CDN/cache contract, then
  build admin media previews and adjustments, then enable/verify production CDN
  after adjusted-media cache behavior is proven.
- **D-08-02:** Do not make the media review/editor wait on production CDN
  enablement. CDN readiness belongs before media work; CDN enablement belongs
  after media cache semantics are real enough to verify.
- **D-08-03:** Phase 8 roadmap and requirements should treat CDN
  review/implementation as explicit scope, not as an accidental extension of a
  posture review.

### Security Patching Repair
- **D-08-04:** Treat the current security scanner failure as a runtime scan
  design problem, not a workflow syntax problem. The latest weekly scan failed
  after runtime IP resolution succeeded, during the Ansible security scan step.
- **D-08-05:** Reduce host-side scan work to the minimum inventory needed for
  approval and drift checking. The VM should collect package specs plus the
  smallest advisory/CVE/errata list that is cheap and reliable.
- **D-08-06:** Enrich reports outside the slow host-detail loop. Prefer Oracle
  Linux OVAL data as the structured source; use static Oracle errata HTML pages
  as fallback and browser-facing link targets.
- **D-08-07:** If external enrichment fails but the VM produced a valid package
  inventory, still create/update the GitHub issue with minimal inventory, note
  that enrichment failed, and preserve approval/drift guardrails.
- **D-08-08:** Keep the hidden GitHub issue metadata block limited to exact
  package specs for approval and drift checks. Advisory IDs, CVEs, severity,
  summaries, and links are human-readable report detail, not the approval
  contract.
- **D-08-09:** The report should include concise summaries and links, not large
  copied advisory bodies. Use severity, CVE IDs, affected package rows, and
  Oracle/NVD links where available.
- **D-08-10:** When repeated scans find a changed package set while an issue is
  open, update the same scanner issue, replace stale metadata/body content, and
  remove stale approval state so old approvals cannot apply a drifted package
  set.
- **D-08-11:** Verification should include a live scanner proof and a dry-run
  apply-path exercise where feasible. Do not require installing real production
  updates unless a real approved package set exists.
- **D-08-12:** Report ordering is not locked. During development, re-review real
  scan output and prefer either DNF order or alphabetical package/version order
  if that makes approval review clearer.

### Repo-Wide Posture Pass
- **D-08-13:** The posture pass should review source organization, docs,
  workflows, deployment/process scripts, configuration names, stale
  planning/codebase maps, validation gaps, public-edge/CDN readiness, and cache
  hygiene.
- **D-08-14:** The posture pass should produce a concise findings register plus
  scoped fixes. Aggressive cleanup is welcome, but it should land in separate
  PRs before media work rather than being mixed into the image editor PR.
- **D-08-15:** Add enforceable CI hygiene guardrails where feasible. Prefer
  checks over purely human review notes for docs/codebase-map drift, workflow
  syntax, naming conventions, and privacy/static contracts.
- **D-08-16:** Current known posture concerns include over-worded or stale
  service/config naming, unnecessary create/enable-style Terraform booleans
  where resources are intended as end-state managed infrastructure, and current
  docs that still describe the old Phase 8 taxonomy/AI bundle.

### CDN and Cache Contract
- **D-08-17:** Define the CDN/cache contract before media implementation:
  admin shell/API routes bypass CDN caching and remain `no-store`; public
  HTML/JSON stay rollback-friendly; public assets and generated media use
  cacheable paths; cache purge and rollback behavior are documented.
- **D-08-18:** Adjusted images must produce new public derivative bytes and
  fingerprinted `/media/*` URLs so routine image correction does not require
  manual CDN purges.
- **D-08-19:** Production CDN enablement should happen after admin
  preview/adjustment and publisher cache invalidation are implemented, because
  those flows create the real cache behavior that CDN verification must prove.

### Admin Image Preview
- **D-08-20:** Admin image previews must remain private, authenticated, and
  same-origin. They must not expose direct Object Storage URLs, bucket names,
  namespaces, object keys, image UUIDs in public output, or original filenames.
- **D-08-21:** Saved image tiles should show public-detail-sized authenticated
  thumbnails by default. This makes image review part of normal item management
  rather than a hidden maintenance action.
- **D-08-22:** Each preview should have a focused inspect view. The focused view
  can compare private-latest state against public-current state when an image
  has unpublished changes.
- **D-08-23:** Private-latest previews refresh immediately after upload or
  replacement succeeds. For new or never-published images, show a private-only
  state with a clear unpublished/new badge rather than an empty comparison pane.
- **D-08-24:** Preview failures should show a redacted problem state with retry
  or replace context while keeping bucket, object, path, filesystem, and
  provider details out of browser-visible responses.

### Image Adjustment
- **D-08-25:** Store adjustment metadata, not edited originals. Private originals
  remain unchanged.
- **D-08-26:** Phase 8 adjustment controls must include small rotation,
  crop/zoom, pan, and required auto-assisted deskew/perspective correction.
  Automatic detection should be assisted, not mandatory: fall back to manual
  corner/edge handles when detection is uncertain.
- **D-08-27:** Use a dedicated image review view for adjustment work rather than
  inline controls on the tile. The existing image tiles should lead into this
  focused review view.
- **D-08-28:** The review view should provide multiple visual overlays, such as
  grid, centerline, and card-edge/rectangle guides, to make skew and alignment
  easier to see before saving.
- **D-08-29:** Adjustment edits are draft-local until explicit Save. Cancel
  abandons unsaved edits; Reset clears saved adjustment metadata.
- **D-08-30:** The review view should support before/after toggle plus split
  comparison for detailed checks without overcrowding the default view.
- **D-08-31:** Saved adjustment changes update private admin previews
  immediately, record useful edit-history/pending-change state, and become
  public only through the normal publish flow.
- **D-08-32:** Static publishing must apply saved adjustments before resizing and
  encoding public thumbnail/detail derivatives. The derivative cache key must
  include all adjustment metadata so stale unadjusted derivatives are not
  reused.

### UI Polish
- **D-08-33:** Image review should feel like part of normal collection
  management. Use a dedicated review view for focus, but keep the entry point
  visible from normal item image tiles.
- **D-08-34:** Keep the existing static admin technology choice: plain
  HTML/CSS/JavaScript, same-origin requests, no browser storage, no frontend
  build system.

### the agent's Discretion

No separate `## the agent's Discretion` section exists in `08-CONTEXT.md`; technical details not locked above remain planner discretion if they preserve the locked decisions. [VERIFIED: codebase grep]

### Deferred Ideas (OUT OF SCOPE)

- Taxonomy cue asset upload/approval and public rendering.
- OCR/AI image analysis or metadata suggestions.
- Broad admin redesign outside the image-management workflow and posture fixes.
- Advanced image enhancement beyond the required Phase 8 review/deskew/
  perspective controls, such as batch correction or content-aware automation.

## Project Constraints (from AGENTS.md)

- Use generated static public artifacts plus one Rust private controller for v1; do not introduce a split public frontend/backend service. [VERIFIED: AGENTS.md]
- Prefer OCI Always Free, Oracle Autonomous Database Free, private OCI Object Storage originals, GitHub Actions deploy on merge to `main`, least-privilege OCI access, and explicit secret handling. [VERIFIED: AGENTS.md]
- Keep public static artifacts free of private storage identifiers and unpublished records. [VERIFIED: AGENTS.md]
- Keep persistence/media details in controller adapters and services, not scattered through route handlers or static assets. [VERIFIED: AGENTS.md]
- Use plain static HTML/CSS/JavaScript for admin/public surfaces unless a later phase intentionally changes that constraint. [VERIFIED: AGENTS.md]
- Current validation habits are `cargo fmt`, `cargo test`, `cargo check --features production-persistence`, `cargo clippy`, Ansible syntax/lint checks, and mandatory privacy/static contract tests for public artifact changes. [VERIFIED: AGENTS.md]
- Do not re-scaffold the retired Next.js app, public accounts, multi-admin roles, direct Object Storage URLs, or a split service architecture for v1. [VERIFIED: AGENTS.md]
- Phase 8 must repair security patching first, run repo hygiene and CI guardrails before media work, define CDN/cache contract, then add admin-private preview/adjustment with auto-assisted deskew/perspective correction, and verify CDN after adjusted-media cache behavior is proven. [VERIFIED: AGENTS.md]
- Do not commit directly to `main` or `master`; current branch is `gsd/plan-phase-8`. [VERIFIED: git status]

## Summary

Phase 8 should be planned as five dependent workstreams: security patching repair, posture and CI guardrails, CDN/cache contract, private admin media review/adjustments, and post-media CDN verification. [VERIFIED: CONTEXT.md] The current code already has the right architecture for this: Caddy separates public static from `/admin` and `/admin/api/*`, the Rust controller owns authenticated mutations and private media reads, Oracle owns image metadata, and the publisher owns derivative generation and public privacy validation. [VERIFIED: codebase grep]

The highest-risk media implementation choice is not UI rendering; it is making image adjustment metadata part of every private preview, public derivative, cache key, publish validation, and edit-history path. [VERIFIED: codebase grep] The planner should preserve originals, add an adjustment DTO/model stored on `autograph_images`, generate private preview bytes through authenticated controller routes, and pass adjustment state into `generate_derivative` before resizing/encoding. [VERIFIED: codebase grep]

For security patching, the existing runtime failure is likely in `deploy/ansible/roles/security_patching/tasks/scan.yml`, where `dnf updateinfo info <advisory>` loops per advisory after `dnf updateinfo list --security --available`. [VERIFIED: codebase grep] Oracle documents both DNF security inventory and Oracle-hosted OVAL definition files, so the repair should keep host scanning minimal, treat OVAL or Oracle errata pages as enrichment sources, and keep package specs as the only hidden approval contract. [CITED: https://docs.oracle.com/en-us/iaas/oracle-linux/oci/security-updates-using-dnf.htm] [CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/9/oscap/auditing_for_vulnerabilities_by_using_oval_definitions.html]

**Primary recommendation:** Plan Phase 8 in strict dependency order: security scan repair, posture/CI cleanup, CDN contract docs/tests, admin preview/adjustment model and routes, publisher/cache integration, then live CDN/security verification. [VERIFIED: CONTEXT.md]

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MEDIA-05 | Admin can view authenticated, same-origin previews of private uploaded item images without exposing direct Object Storage URLs, bucket/object identifiers, original filenames, or unpublished media publicly. [VERIFIED: .planning/REQUIREMENTS.md] | Add session-cookie-only controller preview routes under `/admin/api/items/{id}/images/{image_id}/preview`, return derived WebP bytes with `Cache-Control: no-store`, and add static-admin tile/focused-review UI that never renders object keys or original filenames. [VERIFIED: codebase grep] |
| MEDIA-06 | Admin can store non-destructive adjustment metadata for rotation, crop/zoom, pan, deskew/perspective, manual fallback, and public derivative cache invalidation. [VERIFIED: .planning/REQUIREMENTS.md] | Add `ImageAdjustment` domain/schema fields, validate bounded normalized geometry, use `imageproc` projective transforms plus existing `image` resize/WebP path, and include canonical adjustment JSON in derivative cache keys. [CITED: https://docs.rs/imageproc/latest/imageproc/geometric_transformations/] [VERIFIED: codebase grep] |
| ADMIN-06 | Admin image management includes private review, previews, adjustment controls, reset/cancel/save, and normal workflow polish. [VERIFIED: .planning/REQUIREMENTS.md] | Extend `controller/static-admin/admin.js`, `admin.css`, and `index.html` with image tiles, focused review view, overlays, before/after/split views, and draft-local client state. [VERIFIED: codebase grep] |
| OPS-01 | Production security patching scan/apply workflows are repaired and verified with minimal host-side update inventory and authoritative enrichment where practical. [VERIFIED: .planning/REQUIREMENTS.md] | Replace or bypass the slow per-advisory DNF detail loop; use DNF list/CVE inventory on host and Oracle OVAL/errata enrichment off the live scan critical path. [VERIFIED: codebase grep] [CITED: https://linux.oracle.com/security/] |
| OPS-02 | Repo-wide posture pass fixes actionable findings in separate pre-media PRs, adds CI guardrails, and implements/verifies CDN after media cache behavior is proven. [VERIFIED: .planning/REQUIREMENTS.md] | Add a findings register, CI checks for stale maps/docs/cache/privacy contracts, Cloudflare/Caddy contract docs/tests before media, and production CDN verification after adjusted derivatives produce new fingerprinted URLs. [VERIFIED: CONTEXT.md] [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/] |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Security patch inventory and apply | Infrastructure / Delivery | GitHub Actions | Ansible role runs DNF on the runtime VM; GitHub workflows trigger scan/apply and manage issue approval. [VERIFIED: codebase grep] |
| Advisory enrichment | Infrastructure / Delivery | GitHub Actions | Oracle OVAL/errata enrichment should run outside the slow host loop while issue rendering remains in Ansible/GitHub. [CITED: https://linux.oracle.com/security/] |
| Repo posture and CI guardrails | CI / Documentation | Source tree | The pass spans docs, workflows, Terraform, Ansible, codebase maps, and static contract tests. [VERIFIED: CONTEXT.md] |
| CDN/cache contract | CDN / Static Edge | Caddy origin | Cloudflare decides edge cache eligibility, while Caddy origin headers already express `no-store`, media, asset, and HTML/JSON TTLs. [VERIFIED: codebase grep] [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/] |
| Admin private image previews | API / Backend | Browser / Client | The controller must authenticate, read private media, derive preview bytes, and redact failures; the browser only renders same-origin preview URLs. [VERIFIED: codebase grep] |
| Adjustment editing UI | Browser / Client | API / Backend | Draft-local controls, overlays, save/cancel/reset, and compare views are client responsibilities; save/reset persists through session-cookie API routes. [VERIFIED: CONTEXT.md] |
| Adjustment persistence | Database / Storage | API / Backend | Oracle image records need additive adjustment metadata; controller validates and records edit history. [VERIFIED: codebase grep] |
| Public derivative application | Static Publisher | Media Store | Publisher reads private originals, applies adjustment metadata, resizes/encodes derivatives, fingerprints bytes, validates public artifacts, and promotes releases. [VERIFIED: codebase grep] |
| Privacy/static contract validation | Rust tests | Static publisher | Existing `static_contract`, `publisher`, and Caddy tests already enforce public output privacy and cache headers. [VERIFIED: .planning/codebase/TESTING.md] |

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Rust controller | `rustc 1.96.0` locally | Admin routes, media service, domain validation, publisher integration. | Current implementation language and runtime boundary. [VERIFIED: local command] |
| `image` crate | `0.25.10` | Decode JPEG/PNG/WebP, resize, encode sanitized lossless WebP derivatives. | Already used by `controller/src/derivatives.rs`; docs confirm imageops and WebP encoder support. [VERIFIED: Cargo.toml] [CITED: https://docs.rs/image/0.25.10/image/codecs/webp/struct.WebPEncoder.html] |
| `imageproc` crate | `0.27.0` | Rotation, translation, projective transforms, perspective warp for manual/assisted deskew. | Docs expose geometric transformations, `Projection`, `warp`, and `rotate_about_center`; legitimacy seam returned OK. [CITED: https://docs.rs/imageproc/latest/imageproc/geometric_transformations/] [VERIFIED: package-legitimacy] |
| Axum | `0.8.9` | Authenticated admin preview/adjustment routes. | Current route stack in `controller/src/routes.rs`. [VERIFIED: Cargo.toml] |
| Oracle schema updates | project SQL migrations | Store additive adjustment metadata on `autograph_images`. | Current production persistence pattern uses `controller/db/schema.sql` and `controller/db/updates/*.sql`. [VERIFIED: codebase grep] |
| Caddy | deployed runtime config | Origin route separation and Cache-Control headers. | Existing Caddyfile already separates admin/API no-store from public media/assets/docs TTLs. [VERIFIED: codebase grep] [CITED: https://caddyserver.com/docs/caddyfile/directives/header] |
| Cloudflare Cache Rules | account configuration | CDN bypass/cache policy and purge verification. | Cloudflare docs expose cache rules, browser/edge TTL, cache keys, and purge controls. [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/] [CITED: https://developers.cloudflare.com/cache/how-to/purge-cache/] |
| Ansible | core `2.19.0` locally | Runtime security scan/apply repair and validation. | Existing security patching roles/playbooks use Ansible. [VERIFIED: local command] |
| GitHub Actions | repository workflows | Scheduled/manual scan and issue-label apply path. | Existing workflows use schedule, workflow_dispatch, issue label triggers, and scoped `issues: write`. [VERIFIED: codebase grep] [CITED: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows] |

### Supporting

| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| `serde_json` | `1.0.151` | Canonicalize adjustment metadata for API payloads and derivative cache keys. | Use for stable adjustment DTO serialization; ensure deterministic field order by serializing a typed struct or explicit tuple/order. [VERIFIED: Cargo.toml] |
| `sha2` | `0.10` | Source/derivative fingerprinting and cache keys. | Existing publisher uses SHA-256-derived fingerprints; extend the key input to include adjustment metadata. [VERIFIED: codebase grep] |
| `node --check` | Node `v22.22.2` locally | Static admin JS syntax validation. | Existing testing map includes `node --check controller/static-admin/admin.js`. [VERIFIED: local command] |
| Terraform | `1.15.8` locally | Cloudflare/CDN or DNS config only if represented in IaC. | Use if Phase 8 chooses Terraform-managed Cloudflare/edge state; otherwise document manual console/API configuration. [VERIFIED: local command] |
| Oracle Linux OVAL files | current Oracle-hosted XML/BZ2 files | Advisory enrichment and scan proof support. | Use outside the slow host loop or as operator-downloadable enrichment source. [CITED: https://docs.oracle.com/en/operating-systems/oracle-linux/9/oscap/auditing_for_vulnerabilities_by_using_oval_definitions.html] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `imageproc` projective transforms | Hand-written homography/warp math | Hand-rolled math risks subtle coordinate, interpolation, border, and crop bugs; `imageproc` already provides `Projection`, `warp`, and rotation APIs. [CITED: https://docs.rs/imageproc/latest/imageproc/geometric_transformations/] |
| Existing `image` lossless WebP encoder | libwebp-backed lossy encoder | `image` docs state the current WebP encoder is lossless-only; adding lossy WebP could reduce bytes but adds a native dependency and should not block Phase 8 adjustment semantics. [CITED: https://docs.rs/image/0.25.10/image/codecs/webp/struct.WebPEncoder.html] |
| Cloudflare manual dashboard rules | Terraform-managed Cloudflare provider | Terraform gives drift review, but no current Cloudflare provider or account config was found in the repo; planner should not assume credentials or provider setup exists. [VERIFIED: codebase grep] |
| Full OpenCV/AI deskew | Assisted heuristics plus manual corners | Phase 8 explicitly excludes OCR/AI and broad enhancement; manual fallback keeps the feature useful without a heavy native CV stack. [VERIFIED: CONTEXT.md] |

**Installation:**

```bash
cargo add imageproc
```

If `cargo add` is unavailable or changes feature sets unexpectedly, edit `controller/Cargo.toml` to add `imageproc = "0.27.0"` and let `cargo update -p imageproc` resolve the lockfile. [VERIFIED: cargo search] [VERIFIED: package-legitimacy]

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `image` | crates.io | published 2014-11-20 | 3,001,258 weekly | `https://github.com/image-rs/image` | OK | Already approved in project. [VERIFIED: cargo info] [VERIFIED: package-legitimacy] |
| `imageproc` | crates.io | published 2016-01-10 | 218,642 weekly | `https://github.com/image-rs/imageproc.git` | OK | Approved for Phase 8 if projective transforms are implemented in Rust. [VERIFIED: package-legitimacy] |

**Packages removed due to [SLOP] verdict:** none. [VERIFIED: package-legitimacy]
**Packages flagged as suspicious [SUS]:** none. [VERIFIED: package-legitimacy]

`cargo info imageproc` could not be completed in the sandbox because Cargo attempted to write registry cache files under a read-only home directory; the planner can optionally re-run it outside the sandbox, but `cargo search`, docs.rs, GitHub source, and the GSD package legitimacy seam were enough to plan with `imageproc`. [VERIFIED: local command]

## Architecture Patterns

### System Architecture Diagram

```text
Security scan schedule/manual
  -> GitHub Actions weekly-security-scan
  -> Resolve runtime IP
  -> Ansible security_patching scan
  -> DNF package/advisory inventory on VM
  -> Oracle OVAL/errata enrichment outside host loop
  -> GitHub issue body + package-spec metadata
  -> approved-production-update label
  -> Ansible drift check + dnf security apply or dry-run proof

Admin browser
  -> /admin static shell
  -> session-cookie /admin/api/*
  -> item image tile preview URL
  -> controller reads private original
  -> applies saved or draft adjustment metadata
  -> returns no-store WebP preview bytes
  -> focused review view saves/resets metadata
  -> Oracle autograph_images adjustment column + edit event
  -> publish
  -> publisher reads private original
  -> applies adjustment before resize/encode
  -> derivative cache key = source checksum + canonical adjustment + variant
  -> fingerprinted /media/* public paths
  -> Caddy origin headers
  -> Cloudflare cache/bypass rules
```

### Recommended Project Structure

```text
controller/src/
├── image_adjustments.rs      # typed adjustment metadata, validation, canonical cache-key serialization
├── derivatives.rs            # apply transforms before resize/encode
├── publisher.rs              # include adjustment metadata in cache source key and public derivative validation
├── catalog.rs                # AutographImage adjustment field, edit event integration
├── oracle_catalog.rs         # Oracle read/write of adjustment metadata
└── routes/
    └── admin_items.rs        # preview/adjustment API DTOs if route split grows

controller/static-admin/
├── index.html                # focused review view markup and overlay controls
├── admin.js                  # tile previews, review state, save/cancel/reset, comparison UI
└── admin.css                 # preview tiles, review work surface, overlays, no card nesting

deploy/ansible/roles/security_patching/
├── tasks/scan.yml            # minimal host inventory, no slow per-advisory detail loop
├── tasks/create_issue.yml    # same issue update, stale approval removal
└── templates/security-report.md.j2

docs/
├── security-patching.md
├── dns-runbook.md
├── static-runtime-runbook.md
└── phase-08-posture-findings.md
```

### Pattern 1: Session-Cookie-Only Admin API

**What:** Admin collection routes call `authorize_admin_session(&state, &method, &headers)` and reject unauthenticated or CSRF-invalid mutation requests. [VERIFIED: codebase grep]

**When to use:** Use for preview bytes, adjustment save/reset, auto-assist proposal routes, and comparison metadata routes. [VERIFIED: codebase grep]

**Example:**

```rust
// Source: controller/src/routes/admin_items.rs
if let Err(status) = authorize_admin_session(&state, &method, &headers) {
    tracing::warn!(status = %status, "rejected admin image adjustment request");
    return status.into_response();
}
```

### Pattern 2: Public-Safe Derivative Generation

**What:** The publisher reads private media, generates WebP thumbnail/detail derivatives, fingerprints public bytes, writes `/media/{slug}/{image_slug}-{variant}-{fingerprint}.webp`, and validates artifact fingerprints. [VERIFIED: codebase grep]

**When to use:** Apply adjustments before the existing resize/encode path and compute public fingerprints from the adjusted derivative bytes. [VERIFIED: codebase grep]

**Example:**

```rust
// Source: controller/src/publisher.rs
let derivative = generate_derivative(bytes, variant).map_err(|error| {
    tracing::error!(image_id = %image.id, variant = %variant.path_segment(), error = %error);
    error
})?;
let fingerprint = public_derivative_fingerprint(&derivative.bytes);
```

### Pattern 3: Additive Oracle Migration

**What:** Phase schema changes use `controller/db/schema.sql` for end state and `controller/db/updates/*.sql` for idempotent live updates. [VERIFIED: codebase grep]

**When to use:** Add an `adjustment_json` CLOB or explicit numeric columns to `autograph_images`; prefer one CLOB for flexible crop/perspective metadata if validation remains in Rust. [ASSUMED]

**Example:**

```sql
-- Source pattern: controller/db/updates/06-03-media-cleanup.sql
alter table autograph_images add adjustment_json clob;
```

### Pattern 4: Origin Cache Contract Before CDN

**What:** Caddy `header` directives set response headers, and `reverse_proxy` handles upstream admin API traffic. [CITED: https://caddyserver.com/docs/caddyfile/directives/header] [CITED: https://caddyserver.com/docs/caddyfile/directives/reverse_proxy]

**When to use:** Preserve `/admin` and `/admin/api/*` as `no-store`, keep HTML/JSON short-lived, and leave `/media/*` cacheable only when path fingerprints change with bytes. [VERIFIED: codebase grep]

**Example:**

```caddyfile
# Source: deploy/ansible/roles/autographs_deploy/files/Caddyfile
handle /admin/api/* {
    header Cache-Control "no-store"
    reverse_proxy autographs-controller:8080
}

@staticMedia path /media/*
header @staticMedia Cache-Control "public, max-age=86400"
```

### Anti-Patterns to Avoid

- **Serving private originals directly to the browser:** Use controller-generated preview derivatives instead; browser-visible responses must not reveal object keys, buckets, filenames, or UUIDs. [VERIFIED: CONTEXT.md]
- **Using object checksum alone for derivative cache keys:** Adjustment-only changes would reuse stale derivatives if the private original bytes did not change. [VERIFIED: codebase grep]
- **Putting adjustment state only in browser storage:** The project forbids browser storage for admin credentials/state assumptions and requires persisted adjustment metadata. [VERIFIED: CONTEXT.md]
- **Running CVE enrichment as a per-advisory VM loop:** The current scan.yml contains exactly this slow pattern; move enrichment outside that loop. [VERIFIED: codebase grep]
- **Enabling CDN before adjusted-media semantics are testable:** Phase 8 explicitly requires the cache contract first and CDN enablement after adjusted derivatives are proven. [VERIFIED: CONTEXT.md]
- **Large copied advisory bodies in GitHub issues:** Keep concise summaries and links; hidden metadata remains package specs only. [VERIFIED: CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Projective image transforms | Custom homography, sampling, interpolation, and border handling | `imageproc::geometric_transformations::{Projection, warp, rotate_about_center}` | Docs show built-in projective transformation and rotation support. [CITED: https://docs.rs/imageproc/latest/imageproc/geometric_transformations/] |
| Basic decode/resize/WebP derivative path | New image pipeline from scratch | Existing `image` crate path in `derivatives.rs` | Current path already bounds source size, resizes, converts to RGBA, and emits sanitized WebP. [VERIFIED: codebase grep] |
| CDN cache bypass/caching semantics | Ad hoc request routing assumptions | Caddy origin headers plus Cloudflare Cache Rules | Cloudflare docs expose bypass, browser TTL, edge TTL, and custom cache key settings; Caddy docs cover header setting. [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/] [CITED: https://caddyserver.com/docs/caddyfile/directives/header] |
| Security update approval workflow | Free-form issue comments or manual shell commands | Existing scanner issue metadata, labels, and Ansible drift check | Current workflows already gate on scanner labels, approver allowlist, and exact package specs. [VERIFIED: codebase grep] |
| Oracle Linux advisory source | Scraped arbitrary CVE blogs | Oracle Linux DNF updateinfo, OVAL, Oracle errata pages, NVD links | Oracle documents DNF security errata and Oracle-hosted OVAL definitions. [CITED: https://docs.oracle.com/en-us/iaas/oracle-linux/oci/security-updates-using-dnf.htm] |

**Key insight:** Phase 8 failures will mostly come from stale state and boundary leaks, not from missing UI widgets; plan every image adjustment through persistence, preview, publisher, cache key, public artifact validation, and CDN behavior. [VERIFIED: codebase grep]

## Common Pitfalls

### Pitfall 1: Adjustment Metadata Not Included In Cache Key

**What goes wrong:** A saved crop/deskew does not change the original checksum, so the publisher reuses old unadjusted derivatives. [VERIFIED: codebase grep]
**Why it happens:** Current `derivative_cache_source_key` is `checksum:{checksum}` only. [VERIFIED: codebase grep]
**How to avoid:** Build cache key from `{source_checksum, canonical_adjustment_json, variant, transform_version}` and keep public URL fingerprint based on final bytes. [ASSUMED]
**Warning signs:** Publisher tests pass replacement cases but fail adjustment-only publish cases; public `/media/*` URL remains unchanged after adjustment save and publish. [ASSUMED]

### Pitfall 2: Private Preview Route Leaks Storage Detail

**What goes wrong:** Browser-visible errors or URLs include object keys, bucket names, image UUIDs, original filenames, or filesystem paths. [VERIFIED: CONTEXT.md]
**Why it happens:** Current domain image records store object keys and filenames, and admin UI currently renders content type/size only. [VERIFIED: codebase grep]
**How to avoid:** Return preview bytes from opaque same-origin URLs and use generic error bodies; log classified errors server-side without provider details in response. [VERIFIED: codebase grep]
**Warning signs:** Static admin tests find `object_key`, `bucket`, `originalFilename`, or raw UUID-like media identifiers in public/admin-rendered response JSON where not needed. [ASSUMED]

### Pitfall 3: Auto-Deskew Becomes A Blocking CV Project

**What goes wrong:** The plan over-invests in edge detection and blocks useful manual correction. [ASSUMED]
**Why it happens:** Reliable automatic card-edge detection varies by background, glare, borders, sleeves, and memorabilia shape. [ASSUMED]
**How to avoid:** Make auto-assist a bounded proposal route and always provide manual corner/edge controls. [VERIFIED: CONTEXT.md]
**Warning signs:** Acceptance criteria require automatic detection to succeed on every image. [VERIFIED: CONTEXT.md]

### Pitfall 4: CDN Caches Admin Or Rollback-Sensitive Content

**What goes wrong:** Admin/API responses, HTML, JSON, or stale media remain cached beyond their intended freshness. [VERIFIED: CONTEXT.md]
**Why it happens:** CDN rules can override or ignore origin cache headers if configured too broadly. [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/]
**How to avoid:** Plan explicit Cloudflare bypass rules for `/admin*` and `/admin/api/*`, respect origin or short TTL for HTML/JSON, and cache `/media/*` by fingerprinted path. [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/]
**Warning signs:** `CF-Cache-Status` shows cached/admin hits or public JSON remains stale after rollback. [ASSUMED]

### Pitfall 5: Security Scan Still Fails Before Issue Creation

**What goes wrong:** A scan with valid package inventory fails because enrichment times out or an external source is unavailable. [VERIFIED: CONTEXT.md]
**Why it happens:** Existing `scan.yml` collects detail with a loop over advisory IDs; Phase 8 decisions require issue creation with minimal inventory even when enrichment fails. [VERIFIED: codebase grep]
**How to avoid:** Treat enrichment as best-effort report detail and render a degraded issue if package specs were collected. [VERIFIED: CONTEXT.md]
**Warning signs:** No GitHub issue is created even though `dnf updateinfo list --security --available` succeeded. [ASSUMED]

## Current Codebase Entry Points

| Area | Current Files | Planning Notes |
|------|---------------|----------------|
| Admin image tile UI | `controller/static-admin/admin.js`, `index.html`, `admin.css` | `renderImages` currently renders metadata/actions only; add thumbnails and review entry without a frontend build system. [VERIFIED: codebase grep] |
| Admin item/media routes | `controller/src/routes.rs`, `controller/src/routes/admin_items.rs` | Existing upload/replace/delete/primary routes are session-cookie protected; add preview and adjustment routes to the same boundary. [VERIFIED: codebase grep] |
| Domain model | `controller/src/catalog.rs` | `AutographImage` lacks adjustment metadata; edit event kinds lack adjustment-specific event. [VERIFIED: codebase grep] |
| Oracle persistence | `controller/src/oracle_catalog.rs`, `controller/db/schema.sql`, `controller/db/updates/` | `autograph_images` stores object metadata, checksum, primary/sort/alt/original filename, but no adjustment column. [VERIFIED: codebase grep] |
| Derivatives | `controller/src/derivatives.rs` | Current function decodes, resizes, encodes WebP; add optional transform before resize. [VERIFIED: codebase grep] |
| Publisher/cache | `controller/src/publisher.rs` | Current cache key uses image checksum only; add adjustment metadata and transform version. [VERIFIED: codebase grep] |
| Static privacy validation | `controller/tests/static_contract.rs`, `controller/tests/publisher.rs` | Extend tests to include adjusted derivatives and forbid adjustment/private fields in public output unless explicitly public-safe. [VERIFIED: .planning/codebase/TESTING.md] |
| Caddy cache origin | `deploy/ansible/roles/autographs_deploy/files/Caddyfile`, `controller/tests/caddy_static_routes.rs` | Current headers already encode the intended contract; Phase 8 should make CDN fronting explicit and test/admin bypass. [VERIFIED: codebase grep] |
| Security patching | `.github/workflows/weekly-security-scan.yml`, `.github/workflows/apply-security-updates.yml`, `deploy/ansible/roles/security_patching/`, `docs/security-patching.md` | Existing apply path has approval labels and drift checks; scan path has slow advisory detail loop to remove. [VERIFIED: codebase grep] |

## Code Examples

### Adjustment Cache Key Shape

```rust
// Source: recommended extension of controller/src/publisher.rs [ASSUMED]
let adjustment_key = image
    .adjustment
    .as_ref()
    .map(ImageAdjustment::canonical_cache_key)
    .unwrap_or_else(|| "adjustment:none".to_owned());
let cache_key = format!("checksum:{checksum};transform:v1;{adjustment_key}");
```

### Projective Transform Shape

```rust
// Source: docs.rs imageproc geometric_transformations [CITED: https://docs.rs/imageproc/latest/imageproc/geometric_transformations/]
use imageproc::geometric_transformations::{warp, Border, Interpolation, Projection};

let projection = Projection::from_control_points(source_corners, target_corners)
    .ok_or_else(|| "invalid perspective control points".to_owned())?;
let adjusted = warp(&rgba, projection, Interpolation::Bilinear, Border::Transparent);
```

### Private Preview Route Contract

```rust
// Source: recommended extension of controller/src/routes.rs [ASSUMED]
let mut response = (
    StatusCode::OK,
    [(header::CONTENT_TYPE, "image/webp")],
    preview.bytes,
)
    .into_response();
response
    .headers_mut()
    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
```

### Degraded Security Report Logic

```yaml
# Source: recommended extension of deploy/ansible/roles/security_patching/tasks/scan.yml [ASSUMED]
- name: Mark enrichment failure without discarding package inventory
  ansible.builtin.set_fact:
    security_patching_enrichment_status: degraded
    security_patching_enrichment_message: "Oracle advisory enrichment failed; package inventory is still available for review."
  when:
    - security_patching_update_package_specs | length > 0
    - security_patching_enrichment_failed | default(false)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Public app-mediated image streaming | Generated static public derivatives served by Caddy | Phase 5/6 | Phase 8 previews must be private admin-only routes; public media remains generated `/media/*`. [VERIFIED: .planning/codebase/ARCHITECTURE.md] |
| One-day `/media/*` cache with source-checksum cache reuse | Fingerprinted adjusted derivatives with adjustment-aware source keys | Phase 8 target | Adjustment-only saves must produce new bytes/paths after publish. [VERIFIED: CONTEXT.md] |
| Host-side `dnf updateinfo info <advisory>` detail loop | Minimal host package/advisory inventory plus external Oracle OVAL/errata enrichment | Phase 8 target | Scans should still create issues when enrichment fails. [VERIFIED: CONTEXT.md] |
| CDN as future posture note | Cloudflare/CDN contract and enablement as first-class Phase 8 work | Phase 8 scope update | Planner must include contract before media and enablement after adjusted-media proof. [VERIFIED: ROADMAP.md] |

**Deprecated/outdated:**
- Retired Next.js app or pnpm workspace: do not revive for Phase 8. [VERIFIED: AGENTS.md]
- Direct Object Storage/public URLs: still out of scope and conflicts with privacy requirements. [VERIFIED: .planning/REQUIREMENTS.md]
- Hidden security issue metadata containing advisories/CVEs: Phase 8 locks metadata to package specs only. [VERIFIED: CONTEXT.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | A single `adjustment_json` CLOB on `autograph_images` is preferable to many columns if Rust validation remains strict. | Architecture Patterns | Oracle query/report needs may prefer explicit numeric columns. |
| A2 | Auto-deskew can be implemented as bounded heuristic assistance rather than reliable full automatic detection. | Common Pitfalls | Planner may need a spike if the user expects robust automatic card-edge detection. |
| A3 | Canonical adjustment cache keys should include source checksum, adjustment JSON, variant, and transform version. | Common Pitfalls / Code Examples | Cache invalidation could miss changes if canonicalization is unstable. |
| A4 | Cloudflare verification can rely on response headers such as `CF-Cache-Status`. | Common Pitfalls | Account plan or proxy mode may expose different debugging headers. |

## Open Questions (RESOLVED)

1. **Cloudflare configuration ownership**
   - Resolution: Phase 8 plans must not assume Terraform-managed Cloudflare or live account/API access because no Terraform Cloudflare provider or account-specific zone configuration was found in the repo. [VERIFIED: codebase grep]
   - Planning consequence: Treat CDN ownership as a documented/manual operator configuration plus blocking production verification checkpoint. The plan should define exact Cloudflare rule names and cache semantics in docs first, then require live evidence after adjusted-media cache behavior exists per D-08-17, D-08-18, and D-08-19.
   - Live-fact boundary: Do not invent whether the current production hostname is proxied through Cloudflare. Record `CF-Cache-Status` or account-equivalent headers only during the planned live checkpoint.

2. **Adjustment metadata storage shape**
   - Resolution: Use a typed Rust adjustment DTO with one additive private Oracle `autograph_images.adjustment_json` CLOB for Phase 8.
   - Planning consequence: Rust owns validation, canonical serialization, edit-history semantics, and derivative cache-key generation; Oracle stores the private metadata without requiring SQL-visible crop/rotation/perspective columns. This follows D-08-25, D-08-29, D-08-31, and D-08-32 while avoiding public schema exposure.
   - Public-boundary rule: `adjustment_json` and derived private adjustment fields remain admin/private metadata and must be denied by public static contract tests.

3. **Auto-assist algorithm depth**
   - Resolution: Auto-assisted deskew/perspective correction is required but bounded: implement a deterministic heuristic proposal route with manual fallback, not OCR/AI or a full computer-vision subsystem per D-08-26 and the deferred Phase 10 AI scope.
   - Planning consequence: Tests must include one high-contrast skew fixture where `propose_image_adjustment` returns `status: "confident"` with four normalized corners, plus separate ambiguous/low-contrast fixtures where `status: "unavailable"` and the UI-spec fallback copy are valid. Unavailable is not acceptable for the deterministic high-contrast fixture.
   - UI/API consequence: The admin route and UI must preserve manual corner handles when auto-assist is unavailable, and must surface the confident proposal when the backend returns normalized corners.

4. **Live production security scan failure details**
   - Resolution: Treat the known failure as post-IP-resolution Ansible scan failure per D-08-04, but do not claim the exact live run/log details were inspected in this research artifact.
   - Planning consequence: Plan 08-01 should repair the likely slow host-side advisory loop, optionally record the latest failing run ID when `gh` access is available, and require scanner proof in operator verification per D-08-11.
   - Live-proof boundary: The definitive live scan proof is a planned verification artifact, not a research fact. It must capture a successful scanner issue create/update path, degraded enrichment behavior when exercised, stale approval removal, and dry-run apply-path evidence where feasible.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust toolchain | Controller/media/publisher work | yes | `rustc 1.96.0`, `cargo 1.96.0` | None needed. [VERIFIED: local command] |
| Node.js | Static admin JS syntax checks | yes | `v22.22.2` | None needed. [VERIFIED: local command] |
| Terraform | Optional CDN/IaC and existing infra validation | yes | `1.15.8` | Manual Cloudflare docs if no provider is added. [VERIFIED: local command] |
| Ansible | Security patching and deploy validation | yes with temp env | `ansible-core 2.19.0` | Set `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local` in sandboxed runs. [VERIFIED: local command] |
| ansible-lint | Ansible role validation | yes with temp env | `25.6.1+really25.2.1` | Set `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local`; otherwise home tmp is read-only here. [VERIFIED: local command] |
| Cloudflare account/API | CDN enablement | unknown | not probed | Manual dashboard/runbook verification or add explicit Terraform/API setup. [ASSUMED] |
| Live OCI runtime VM | Security scan/apply proof, admin preview live smoke | unknown | not probed | Local tests plus operator-run live smoke. [VERIFIED: .planning/codebase/TESTING.md] |

**Missing dependencies with no fallback:** none confirmed locally. [VERIFIED: local command]

**Missing dependencies with fallback:** Cloudflare account/API and live OCI runtime were not probed; use docs/manual operator verification if automation credentials are not available. [ASSUMED]

## Verification Strategy

Nyquist validation architecture is omitted because `.planning/config.json` sets `workflow.nyquist_validation` to `false`. [VERIFIED: .planning/config.json]

| Requirement | Fast Verification | Full / Live Verification |
|-------------|-------------------|--------------------------|
| OPS-01 | `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml`; ansible-lint security role. [VERIFIED: .planning/codebase/TESTING.md] | Manual/workflow-dispatch scan proves issue create/update, degraded enrichment, stale approval removal, and dry-run apply/drift path. [VERIFIED: CONTEXT.md] |
| OPS-02 | CI guardrail tests/scripts for docs/map drift, workflow syntax, Caddy cache contract, and static privacy checks. [VERIFIED: CONTEXT.md] | Posture findings register plus separate PR evidence and CDN verification after media cache proof. [VERIFIED: CONTEXT.md] |
| MEDIA-05 | Rust route tests for preview auth/no-store/redacted errors; static admin tests for preview tiles and no object identifiers/original filenames. [VERIFIED: .planning/codebase/TESTING.md] | Live admin preview against OCI media through same-origin `/admin/api/*` route. [VERIFIED: .planning/codebase/TESTING.md] |
| MEDIA-06 | Unit tests for adjustment validation/canonicalization; derivative tests for rotate/crop/perspective output dimensions and cache key misses when adjustment changes. [CITED: https://docs.rs/imageproc/latest/imageproc/geometric_transformations/] | Live static publish smoke verifies adjusted derivative bytes and new fingerprinted `/media/*` paths. [VERIFIED: CONTEXT.md] |
| ADMIN-06 | `node --check controller/static-admin/admin.js`, static admin DOM/source privacy tests, focused review interaction tests where existing harness supports it. [VERIFIED: .planning/codebase/TESTING.md] | Operator UAT: upload/replace image, preview, save/cancel/reset adjustment, compare unpublished vs public-current, publish, verify public output. [VERIFIED: CONTEXT.md] |

Suggested routine gates:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
node --check controller/static-admin/admin.js
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml
ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | yes | Existing single-admin HTTP-only session cookie for collection management. [VERIFIED: codebase grep] |
| V3 Session Management | yes | SameSite Strict session cookie, logout clearing, and no browser credential storage. [VERIFIED: codebase grep] |
| V4 Access Control | yes | Server-side `authorize_admin_session` on preview/adjustment routes; no bearer compatibility for media management. [VERIFIED: codebase grep] |
| V5 Input Validation | yes | Rust DTO validation for UUIDs, file type/size, normalized crop/corners, bounded rotation/zoom/pan, and reject invalid projections. [VERIFIED: codebase grep] |
| V6 Cryptography | yes | Do not hand-roll auth/session crypto; continue existing auth state/session machinery and SHA-256 fingerprinting patterns. [VERIFIED: codebase grep] |
| V9 Communications | yes | Same-origin admin routes behind Caddy; Cloudflare should not cache admin/API responses. [VERIFIED: codebase grep] |
| V12 File and Resources | yes | Validate image type/size, cap source bytes, preserve private originals, strip public metadata through derivative generation. [VERIFIED: codebase grep] |
| V14 Configuration | yes | Scoped GitHub token permissions, pinned production-sensitive actions, explicit CDN/cache runbook and Caddy tests. [VERIFIED: codebase grep] |

### Known Threat Patterns for Phase 8

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Object Storage identifier leakage in preview/public output | Information Disclosure | Same-origin opaque preview routes, redacted errors, static privacy tests, no direct object URLs. [VERIFIED: CONTEXT.md] |
| CDN caching admin/API responses | Information Disclosure | Cloudflare bypass rules for `/admin*` and Caddy `Cache-Control: no-store`. [CITED: https://developers.cloudflare.com/cache/how-to/cache-rules/settings/] [VERIFIED: codebase grep] |
| Stale public derivative after adjustment | Tampering / Integrity | Adjustment-aware derivative cache key and byte-fingerprinted public media path. [VERIFIED: CONTEXT.md] |
| Malicious or malformed image upload | Denial of Service | Existing 20 MiB source limit, content type checks, image decode validation, transform bounds. [VERIFIED: codebase grep] |
| Unauthorized patch apply through issue labels | Elevation of Privilege | Existing approver allowlist, scanner label checks, package-spec drift check, stale approval removal. [VERIFIED: codebase grep] |
| Advisory enrichment failure suppresses patch visibility | Operational Availability | Degraded issue creation with package inventory and enrichment failure note. [VERIFIED: CONTEXT.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/08-admin-media-review-and-operational-posture/08-CONTEXT.md` - locked decisions, sequencing, deferred scope. [VERIFIED: codebase grep]
- `.planning/REQUIREMENTS.md` - Phase 8 requirements `MEDIA-05`, `MEDIA-06`, `ADMIN-06`, `OPS-01`, `OPS-02`. [VERIFIED: codebase grep]
- `.planning/ROADMAP.md` - Phase 8 success criteria and explicit CDN/cache scope. [VERIFIED: codebase grep]
- `AGENTS.md` - project constraints and guardrails. [VERIFIED: codebase grep]
- `controller/src/routes.rs`, `routes/admin_items.rs`, `catalog.rs`, `derivatives.rs`, `publisher.rs`, `controller/db/schema.sql`, `controller/static-admin/admin.js`, Caddyfile, security patching role/workflows. [VERIFIED: codebase grep]
- Caddy official docs: `https://caddyserver.com/docs/caddyfile/directives/header`, `https://caddyserver.com/docs/caddyfile/directives/reverse_proxy`. [CITED: caddyserver.com]
- Oracle official docs: `https://docs.oracle.com/en-us/iaas/oracle-linux/oci/security-updates-using-dnf.htm`, `https://docs.oracle.com/en/operating-systems/oracle-linux/9/oscap/auditing_for_vulnerabilities_by_using_oval_definitions.html`, `https://linux.oracle.com/security/`. [CITED: docs.oracle.com/linux.oracle.com]
- docs.rs for `image` and `imageproc`: `https://docs.rs/image/0.25.10/`, `https://docs.rs/imageproc/latest/imageproc/geometric_transformations/`. [CITED: docs.rs]
- GitHub Actions official docs: `https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows`, `https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax`, `https://docs.github.com/en/actions/concepts/security/github_token`. [CITED: docs.github.com]

### Secondary (MEDIUM confidence)

- Cloudflare official docs: `https://developers.cloudflare.com/cache/how-to/cache-rules/settings/`, `https://developers.cloudflare.com/cache/how-to/purge-cache/`. These are authoritative for Cloudflare behavior but no account-specific config was inspected. [CITED: developers.cloudflare.com]

### Tertiary (LOW confidence)

- Assumptions about best adjustment storage shape, auto-deskew heuristic complexity, and Cloudflare live debug headers need implementation-time confirmation. [ASSUMED]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH for existing Rust/Caddy/Ansible/GitHub stack and MEDIUM for adding `imageproc`, because docs.rs and package legitimacy passed but `cargo info imageproc` could not complete in the sandbox. [VERIFIED: local command]
- Architecture: HIGH because current source and codebase maps identify the module boundaries directly. [VERIFIED: codebase grep]
- Pitfalls: HIGH for privacy/cache/security-patching pitfalls tied to source and locked decisions; MEDIUM for auto-deskew heuristic risk. [VERIFIED: CONTEXT.md]

**Research date:** 2026-07-30
**Valid until:** 2026-08-29 for codebase architecture; 2026-08-06 for Cloudflare/GitHub/Oracle external behavior. [ASSUMED]
