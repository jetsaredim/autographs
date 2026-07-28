# Phase 8: Admin Media Review and Operational Posture - Context

**Gathered:** 2026-07-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 8 improves the existing Rust/static single-admin workflow before adding
taxonomy media cues or advisory AI. It has two connected goals:

1. Restore operational confidence by repairing the production security patching
   workflow and running a repo-wide source/docs/workflow/config posture pass.
2. Make uploaded item images visible and correctable in admin by adding
   authenticated previews plus non-destructive adjustment metadata that the
   publisher applies to generated public derivatives.

This phase does not add taxonomy cue assets, OCR/AI provider integration,
public accounts, multi-admin roles, direct Object Storage URLs, bulk import, or
a split public frontend/backend service.

</domain>

<decisions>
## Implementation Decisions

### Security Patching Repair
- **D-08-01:** Treat the current security scanner failure as a runtime scan
  design problem, not a workflow syntax problem. The latest weekly scan failed
  after runtime IP resolution succeeded, during the Ansible security scan step.
- **D-08-02:** Reduce host-side scan work to the minimum inventory needed for
  approval and drift checking. Prefer collecting package specs, advisory IDs,
  severity, and concise CVE identifiers from the instance over looping through
  slow per-advisory detail commands.
- **D-08-03:** Where richer advisory detail is useful in the GitHub issue, prefer
  authoritative external Oracle Linux sources instead of making the live VM
  gather all detail. Phase planning should verify exact source shape, but
  official candidates include Oracle Linux errata and CVE pages plus DNF's
  narrowed `updateinfo list security` / `updateinfo list cves` outputs.
- **D-08-04:** Preserve the existing guarded approval model: scanner-created
  issue, allowlisted approval label actor, package-set drift refusal, result or
  failure comments, and stale approval-label cleanup.

### Repo-Wide Posture Pass
- **D-08-05:** The posture pass should review source organization, docs,
  workflows, deployment/process scripts, configuration names, stale
  planning/codebase maps, and validation gaps.
- **D-08-06:** The posture pass should fix narrow, low-risk inconsistencies when
  practical and explicitly track larger follow-ups rather than mixing broad
  refactors into unrelated image/UI work.
- **D-08-07:** Current known posture concerns include over-worded or stale
  service/config naming, unnecessary create/enable-style Terraform booleans
  where resources are intended as end-state managed infrastructure, and current
  docs that still describe the old Phase 8 taxonomy/AI bundle.
- **D-08-08:** Cloudflare/CDN fronting was already evaluated and deferred in
  Phase 6. Phase 8 should revalidate that decision as part of the operational
  posture pass, but actual CDN enablement should be separately planned unless
  the review proves it is only a small docs/config change with clear admin/API
  bypass, cache purge, rollback, TLS, and privacy behavior.

### Admin Image Preview
- **D-08-09:** Admin image previews must remain private, authenticated, and
  same-origin. They must not expose direct Object Storage URLs, bucket names,
  namespaces, object keys, image UUIDs in public output, or original filenames.
- **D-08-10:** Preview generation can use small controller-rendered derivatives
  or existing generated-safe derivative logic, but public static artifacts must
  still only publish intentional generated media.
- **D-08-11:** Admin image tiles should become inspectable image controls, not
  metadata-only rows. Preview visibility is the foundation for later adjustment,
  cue-asset approval, and AI image-selection work.

### Image Adjustment
- **D-08-12:** Store adjustment metadata, not edited originals. Private originals
  remain unchanged.
- **D-08-13:** First-pass adjustments should cover practical small corrections:
  rotation, crop/zoom, and pan. Perspective/deskew should remain a later tier
  unless planning proves it is small enough and well-supported.
- **D-08-14:** The admin UI should support preview, before/after comparison,
  reset, cancel/reject, and save. Browser-side preview can be fast and local,
  while server-side preview should confirm the exact publisher result when
  needed.
- **D-08-15:** Static publishing must apply saved adjustments before resizing and
  encoding public thumbnail/detail derivatives. The derivative cache key must
  include adjustment metadata so stale unadjusted derivatives are not reused.
- **D-08-16:** Adjustment changes are private metadata changes and should
  participate in pending-change/publish visibility and edit history at a useful
  single-admin level.

### UI Polish
- **D-08-17:** Image review should feel like part of normal collection
  management. Avoid a maintenance-only hidden tool unless planning proves the
  daily admin page would become too crowded.
- **D-08-18:** Keep the existing static admin technology choice: plain
  HTML/CSS/JavaScript, same-origin requests, no browser storage, no frontend
  build system.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Requirements
- `.planning/ROADMAP.md` - Phase 8 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - `MEDIA-05`, `MEDIA-06`, `ADMIN-06`, `OPS-01`,
  and `OPS-02`.
- `.planning/PROJECT.md` - Product constraints, v1 out-of-scope boundaries, and
  the decision to split admin media/posture, taxonomy cues, and AI ingest.
- `.planning/STATE.md` - Current project state, known operational concern, and
  accumulated Phase 5-7 decisions.

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
- Oracle Linux DNF security update guidance:
  `https://docs.oracle.com/en-us/iaas/oracle-linux/oci/security-updates-using-dnf.htm`
- Oracle Linux errata/CVE listing guidance:
  `https://docs.oracle.com/en/operating-systems/oracle-linux/uln/uln-TrackSecurityUpdatesandErrataReleases.html`

</canonical_refs>

<code_context>
## Existing Code Insights

- Admin image UI currently renders metadata and actions only; it does not show
  uploaded image previews.
- `controller/src/derivatives.rs` currently decodes, resizes, and encodes WebP
  derivatives without adjustment metadata.
- `controller/src/publisher.rs` builds public thumbnail/detail derivatives and
  caches by source checksum; adjustment metadata must become part of cache
  invalidation.
- `autograph_images` currently stores object/storage metadata, checksum/etag,
  primary/sort/alt text, and filenames; it has no adjustment column yet.
- Security patching currently validates at syntax level locally, but weekly
  GitHub runs still fail in the Ansible scan step after IP resolution.

</code_context>

<specifics>
## Specific Ideas

- Add an `Adjust` action on each admin image tile after previews exist.
- Use small bounded rotation controls such as `-3` to `+3` degrees first.
- Use normalized crop/pan/zoom metadata so future source dimensions do not leak
  into public output or hard-code one derivative size.
- Keep perspective/deskew as an explicit later option unless the plan finds a
  low-risk Rust image transform library and a simple enough UI.
- For security scanning, consider a two-stage model: quick host inventory first,
  then external advisory enrichment for issue body details.

</specifics>

<deferred>
## Deferred Ideas

- Automatic deskew detection and four-corner perspective correction.
- Taxonomy cue asset upload/approval and public rendering.
- OCR/AI image analysis or metadata suggestions.
- Broad admin redesign outside the image-management workflow and posture fixes.

</deferred>

---

*Phase: 8-Admin Media Review and Operational Posture*
*Context gathered: 2026-07-28*
