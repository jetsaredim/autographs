# Phase 9: Taxonomy Media Cues - Context

**Gathered:** 2026-07-20
**Realigned:** 2026-07-28
**Status:** Future phase; ready for planning after Phase 8

<domain>
## Phase Boundary

Phase 9 adds optional public-safe media cues for taxonomy values after the admin
media review/adjustment foundation exists. It focuses on franchise, product
line, set, and non-default language values. Text metadata remains canonical and
fully sufficient.

This phase does not add OCR/AI provider integration, AI metadata suggestions,
public accounts, multi-admin roles, bulk import, direct Object Storage URLs, or
a public split frontend/backend service.

</domain>

<decisions>
## Implementation Decisions

### Taxonomy Cue Scope
- **D-09-01:** Start with an event-ready starter set plus current catalog gaps.
- **D-09-02:** First-pass cue types are franchise, product line, non-default
  language, and set only when product line is filled in. Do not create
  free-floating set cues.
- **D-09-03:** Format should not receive a visual cue and should be visually
  minimized on the detail page because item images usually make format obvious.
- **D-09-04:** Text fallback is mandatory. No taxonomy value should require a
  media cue to remain understandable or publishable.

### Cue Assets And Approval
- **D-09-05:** Start cue assets as curated image thumbnails, with room for later
  stylized/icon fallback if safe image coverage proves difficult.
- **D-09-06:** Most cue assets will come from dedicated uploads or web-sourced
  candidates, not arbitrary existing item photos.
- **D-09-07:** Web-sourced images are candidates only. Final cue assets require
  explicit operator approval before public use.
- **D-09-08:** Candidate cue assets may appear in private/admin preview, but
  generated public output must include approved assets only.
- **D-09-09:** Cue asset review must account for copyright/publication risk,
  privacy, and source suitability.

### Detail Page Placement
- **D-09-10:** Detail-page taxonomy cues should live inside the revealed
  metadata panel, not in the always-visible header. The signed item remains the
  visual hero.
- **D-09-11:** Inside revealed metadata, signers stay first. The taxonomy cue
  treatment follows signers and enriches or replaces the existing plain
  Classification group.
- **D-09-12:** Render classification identity as a compact media-first visual tag
  row with accessible names and text fallback.
- **D-09-13:** Card backs and many product-line assets are rectangular, so visual
  tags should support constrained height with flexible width/aspect.
- **D-09-14:** Use one approved image plus focal/crop metadata per cue asset for
  the first pass. Multiple variants are not required yet.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` - Phase 9 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - `CUE-01`.
- `.planning/PROJECT.md` - Product constraints and phase split.
- `.planning/STATE.md` - Current project state and accumulated decisions.
- `.planning/phases/08-admin-media-review-and-operational-posture/08-CONTEXT.md`
  - Admin media preview/adjustment foundation that Phase 9 can reuse.
- `.planning/phases/07-metadata-taxonomy-and-public-facets/07-CONTEXT.md`
  - Reusable signer profiles, taxonomy fields, public facets, and detail
  defaults.
- `controller/static-public/templates/detail.html`
- `controller/static-public/assets/site.css`
- `controller/src/publisher.rs`
- `controller/src/contracts.rs`
- `controller/static-admin/admin.js`
- `controller/src/catalog.rs`
- `controller/src/oracle_catalog.rs`

</canonical_refs>

<specifics>
## Specific Ideas

- The first detail-page cue row should feel like a row of visual tags, each with
  hover/focus identification such as `Franchise: Star Wars` or
  `Product line: Star Wars CCG`.
- Cue row ordering should generally be franchise cue(s), product line cue, set
  cue/text when useful, then non-default language cue.
- Generic set names such as `Base Set` should not force fake visual identity.
- Research-assisted cue image discovery can happen outside the app at first,
  with final upload/approval happening through admin.

</specifics>

<deferred>
## Deferred Ideas

- Full in-app web image search for cue assets.
- Stylized/icon fallback after curated thumbnail coverage is tested.
- Broad public browse/card visual redesign unless it falls out naturally from
  the detail-page cue model.
- Advisory OCR/AI metadata suggestions; those are Phase 10.

</deferred>

---

*Phase: 9-Taxonomy Media Cues*
*Context gathered: 2026-07-20; realigned: 2026-07-28*
