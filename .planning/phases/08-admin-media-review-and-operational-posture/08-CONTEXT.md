# Phase 8: Admin Media Review and Operational Posture - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 8 improves the existing Rust/static single-admin workflow before adding
taxonomy media cues or advisory AI. It has three connected goals:

1. Restore operational confidence by repairing the production security patching
   workflow, running aggressive repo/process hygiene cleanup, and adding CI
   hygiene guardrails where feasible.
2. Make CDN/cache fronting explicit Phase 8 work: define the cache contract
   before admin media changes, then enable and verify production CDN only after
   adjusted-image cache behavior can be tested end to end.
3. Make uploaded item images visible, comparable, and correctable in admin by
   adding authenticated previews plus non-destructive adjustment metadata that
   the publisher applies to generated public derivatives.

This phase does not add taxonomy cue assets, OCR/AI provider integration,
public accounts, multi-admin roles, direct Object Storage URLs, bulk import, or
a split public frontend/backend service.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Requirements
- `.planning/ROADMAP.md` - Phase 8 goal, requirements, success criteria, CDN
  scope, posture ordering, and media adjustment expectations.
- `.planning/REQUIREMENTS.md` - `MEDIA-05`, `MEDIA-06`, `ADMIN-06`, `OPS-01`,
  and `OPS-02`.
- `.planning/PROJECT.md` - Product constraints, v1 out-of-scope boundaries, and
  Phase 8 sequencing decisions for security, posture, CDN/cache, and media.
- `.planning/STATE.md` - Current project state, accumulated Phase 5-8
  decisions, and resume file.

### Current Implementation Surfaces
- `.github/workflows/weekly-security-scan.yml`
- `.github/workflows/apply-security-updates.yml`
- `.github/actions/resolve-runtime-ip/action.yml`
- `deploy/ansible/playbooks/security-scan.yml`
- `deploy/ansible/playbooks/security-patch.yml`
- `deploy/ansible/playbooks/security-patch-cleanup.yml`
- `deploy/ansible/roles/security_patching/`
- `docs/security-patching.md`
- `docs/dns-runbook.md`
- `docs/static-runtime-runbook.md`
- `docs/deployment-runbook.md`
- `controller/static-admin/index.html`
- `controller/static-admin/admin.js`
- `controller/static-admin/admin.css`
- `controller/src/routes.rs`
- `controller/src/routes/admin_items.rs`
- `controller/src/catalog.rs`
- `controller/src/oracle_catalog.rs`
- `controller/src/derivatives.rs`
- `controller/src/publisher.rs`
- `controller/tests/admin_workflow.rs`
- `controller/tests/static_admin.rs`
- `controller/tests/publisher.rs`
- `controller/tests/static_contract.rs`

### External References To Verify During Planning
- Oracle Linux OVAL usage guidance:
  `https://docs.oracle.com/en/operating-systems/oracle-linux/9/oscap/auditing_for_vulnerabilities_by_using_oval_definitions.html`
- Oracle Linux security and OVAL index:
  `https://linux.oracle.com/security/`
- Oracle Linux errata pages:
  `https://linux.oracle.com/errata/ELSA-2025-20632.html`
- Oracle Linux DNF security update guidance:
  `https://docs.oracle.com/en-us/iaas/oracle-linux/oci/security-updates-using-dnf.htm`
- Oracle Linux errata/CVE listing guidance:
  `https://docs.oracle.com/en/operating-systems/oracle-linux/uln/uln-TrackSecurityUpdatesandErrataReleases.html`

</canonical_refs>

<code_context>
## Existing Code Insights

- Admin image UI currently renders metadata and actions only; it does not show
  uploaded image previews. `renderImages` in `controller/static-admin/admin.js`
  builds image tiles with alt text, content type, byte size, primary/remove/
  replace actions, and cleanup retry state.
- `controller/src/derivatives.rs` currently decodes, resizes, and encodes WebP
  derivatives without adjustment metadata or deskew/perspective transforms.
- `controller/src/publisher.rs` builds public thumbnail/detail derivatives and
  currently caches by image checksum. Adjustment metadata must become part of
  cache invalidation.
- `autograph_images` currently stores object/storage metadata, checksum/etag,
  primary/sort/alt text, and filenames; it has no adjustment column yet.
- The security patching scan currently runs `dnf updateinfo list --security
  --available`, then loops through `dnf updateinfo info <advisory>` to extract
  CVEs. That per-advisory detail loop is the likely timeout hotspot to remove
  from the critical host path.
- The existing security report template already separates hidden YAML metadata
  from the human-readable Markdown issue body. Phase 8 should keep exact
  package specs as the hidden approval contract.

</code_context>

<specifics>
## Specific Ideas

- Add preview thumbnails directly to saved admin image tiles.
- Use a focused inspect/review view for larger private preview, private-vs-
  public comparison, and adjustment work.
- Use public-detail-sized private previews for admin review instead of serving
  raw originals in the browser.
- Show private-latest image state immediately after upload or replacement.
- Use private-only focused display for images that do not have a published
  counterpart yet.
- In the adjustment view, provide multiple alignment overlays so skew,
  rectangular framing, and misalignment are visible while editing.
- Auto-assisted deskew should propose correction when it can, then allow manual
  corner/edge adjustment when detection is uncertain.
- For security reports, use Oracle OVAL as the preferred enrichment data source
  and Oracle errata HTML pages as fallback/link targets.
- Keep issue row ordering as a development-time choice after seeing real output;
  DNF order or alphabetical package/version are preferred candidates.

</specifics>

<deferred>
## Deferred Ideas

- Taxonomy cue asset upload/approval and public rendering.
- OCR/AI image analysis or metadata suggestions.
- Broad admin redesign outside the image-management workflow and posture fixes.
- Advanced image enhancement beyond the required Phase 8 review/deskew/
  perspective controls, such as batch correction or content-aware automation.

</deferred>

---

*Phase: 8-Admin Media Review and Operational Posture*
*Context gathered: 2026-07-28*
