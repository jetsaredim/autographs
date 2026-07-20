# Phase 8: Taxonomy Media and AI-Assisted Ingest - Context

**Gathered:** 2026-07-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 8 improves the existing single-admin Rust/static catalog workflow with two
connected capabilities:

1. Optional taxonomy media cues that make public item detail metadata more
   useful and visually scannable, starting with franchise, product line, set,
   and non-default language context.
2. Advisory admin-side OCR/AI suggestions that improve metadata consistency and
   new-item ingest speed without replacing manual control.

This phase starts with public detail page taxonomy cue polish because that work
is useful before AI provider integration and can be tested against upcoming
event items plus current catalog gaps. AI work remains admin-only, explicit,
reviewed before save, and never automatic publish behavior.

This phase does not add public accounts, multi-admin roles, bulk import, public
direct Object Storage URLs, a public split frontend/backend service, automatic
AI-created catalog records, or public AI surfaces.

</domain>

<decisions>
## Implementation Decisions

### Taxonomy Cue Scope
- **D-08-01:** Start with an event-ready starter set plus current catalog gaps.
  Upcoming event items should provide practical test material, but first-pass
  cue values should stay grounded in actual or expected catalog taxonomy.
- **D-08-02:** The short-term priority is public detail page UI polish. Ingestion
  can receive light hinting, but the admin flow should assume the operator
  usually knows what is being uploaded.
- **D-08-03:** First-pass cue types are franchise, product line, non-default
  language, and set only when product line is filled in. Do not create
  free-floating set cues.
- **D-08-04:** Format should not receive a visual cue and should be visually
  minimized on the detail page because item images usually make format obvious
  to viewers.
- **D-08-05:** Start cue assets as curated image thumbnails. Keep the model open
  to later stylized/icon fallback if safe, useful image coverage for all
  franchises/product lines/languages/sets becomes difficult.
- **D-08-06:** Text fallback is mandatory. No taxonomy value should require a
  media cue to remain understandable or publishable.

### Cue Creation And Approval
- **D-08-07:** Most cue assets will come from dedicated uploads or web-sourced
  candidate images, not existing item photos, because most item images do not
  include card backs, franchise art, or product-line identity images.
- **D-08-08:** Use research-assisted candidate discovery when useful, with
  manual upload as the fallback. The first pass does not need an in-app web
  image search interface.
- **D-08-09:** Web-sourced images are candidates only. Final cue assets require
  explicit operator approval before public use.
- **D-08-10:** Candidate cue assets may appear in private/admin preview, but
  generated public output must include approved assets only.
- **D-08-11:** Cue asset review must account for copyright/publication risk,
  privacy, and source suitability. Do not auto-publish scraped or unreviewed
  imagery.

### Detail Page Placement
- **D-08-12:** Detail-page taxonomy cues should live inside the revealed metadata
  panel, not in the always-visible header. The signed item remains the visual
  hero.
- **D-08-13:** Inside revealed metadata, signers stay first. The taxonomy cue
  treatment follows signers and enriches or replaces the existing plain
  Classification group rather than becoming a separate flashy banner.
- **D-08-14:** Render classification identity as a compact media-first visual tag
  row. Visible labels should be minimal; each cue must still identify itself
  through hover/focus text, accessible names, and text fallback.
- **D-08-15:** Multiple franchise cues should be supported but are expected to be
  rare and limited to a couple values. Do not overbuild franchise overflow for
  the first pass.
- **D-08-16:** Non-default language should appear as a compact cue; English
  should stay hidden or minimized by default. Language can use a simpler
  flag/text cue rather than requiring curated art.
- **D-08-17:** Generic or weak set names such as `Base Set` should not force fake
  visual identity. The product-line cue can carry the visual identity while set
  appears as small text or is omitted when it adds no useful context.
- **D-08-18:** Cue rendering must not assume square chips. Card backs and many
  product-line assets are rectangular, so visual tags should support constrained
  height with flexible width/aspect.
- **D-08-19:** Use one approved image plus focal/crop metadata per cue asset for
  the first pass. Multiple light/dark/square/wide variants are not required yet.
- **D-08-20:** Use shared cue asset storage with type-specific validation and
  rendering rules. Examples: set cues require product-line context; language
  can use simpler fallback behavior; product-line/set cues can use rectangular
  rendering.

### AI Suggestion Flow
- **D-08-21:** AI remains part of the admin-side product intent. Its role is to
  improve metadata consistency and ingest speed, not to replace the operator's
  knowledge.
- **D-08-22:** First AI pass should include explicit image/OCR analysis, not only
  typed-field consistency checks.
- **D-08-23:** AI suggestion should be an explicit admin action such as
  `Suggest / Check metadata`, not an automatic background save/publish path.
- **D-08-24:** OCR/AI analysis uses the primary image by default, with an option
  to choose another image when needed. Do not design the first pass around
  multi-image AI comparison; most items are expected to have one image.
- **D-08-25:** AI suggestions should consider uploaded image content plus any
  typed fields and may suggest signer credits, character/title, franchise,
  product line, set, language, origin, loose tags, and inscription/OCR text.
- **D-08-26:** AI suggestion UI should combine a review panel summary with
  inline accept/ignore controls near relevant fields. The panel should group
  existing matches, new candidate values, warnings/consistency issues, and OCR
  text.
- **D-08-27:** AI must distinguish reuse of existing values from newly proposed
  values. New signers, franchises, product lines, and sets must be visibly
  marked as new candidates.
- **D-08-28:** Accepting AI suggestions stages values in the existing admin form
  until Save. AI must not invent a separate immediate-create process for
  signers or taxonomy values.
- **D-08-29:** AI-assisted signer suggestions should use the existing signer row
  flow. Existing signer suggestions fill/select an existing signer profile; new
  signer suggestions stage the display name in the row. The current save path
  creates or reuses signer profiles, and duplicate warnings/merge repair remain
  the guardrails.
- **D-08-30:** AI suggestions should show compact confidence and rationale when
  available, especially for new values, OCR/inscription text, and consistency
  warnings.
- **D-08-31:** Accepted AI-assisted values should be lightly marked in item edit
  history so future review can understand why a value was chosen. This remains
  a single-admin edit-history aid, not an enterprise audit system.
- **D-08-32:** Manual item entry must remain fully functional when OCR/AI is
  unavailable, inaccurate, unconfigured, or ignored.

### Planning Safety
- **D-08-33:** If taxonomy cue coverage, image sourcing, cue rendering, or OCR/AI
  behavior proves fuzzier than expected, create a focused spike rather than
  forcing the main implementation through uncertainty.

### the agent's Discretion
- Exact cue asset table/field names, provided shared storage and type-specific
  rules are preserved.
- Exact first-pass cue asset count and starter values, provided they balance
  upcoming event usefulness with current catalog gaps.
- Exact detail-page visual styling, provided the image remains the hero and the
  cue row stays inside revealed metadata after signers.
- Exact AI provider and prompt design, provided suggestions are explicit,
  advisory, reviewed before save, privacy-reviewed, and documented.
- Exact edit-history wording for AI-assisted changes, provided accepted
  AI-assisted field changes remain traceable without expanding into a broad
  audit system.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Requirements
- `.planning/ROADMAP.md` - Phase 8 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - Pending `AI-01` through `AI-05` requirements.
- `.planning/PROJECT.md` - Product constraints, v1 out-of-scope boundaries, and
  the decision to prioritize taxonomy media cues before AI-specific ingest.
- `.planning/STATE.md` - Current project state, Phase 8 focus, and accumulated
  Phase 5-7 decisions.

### Prior Phase Decisions
- `.planning/phases/07-metadata-taxonomy-and-public-facets/07-CONTEXT.md` -
  Reusable signer profiles, signer credits, first-class taxonomy fields, public
  facet behavior, and detail/default-hiding decisions.
- `.planning/phases/06-admin-collection-workflow/06-CONTEXT.md` - Admin
  workflow, save/publish separation, edit history, same-origin static admin,
  and single-admin constraints.
- `.planning/phases/05-static-runtime-migration-foundation/05-CONTEXT.md` -
  Static runtime, private controller, generated derivatives, privacy boundary,
  and publish validation decisions.

### Codebase Maps
- `.planning/codebase/ARCHITECTURE.md` - Current Rust/static architecture,
  schema version 2 public facets, and Phase 8 absence.
- `.planning/codebase/INTEGRATIONS.md` - OCI, Oracle, Object Storage, Caddy,
  admin auth, pending taxonomy media/OCR/AI integrations, and secret handling.
- `.planning/codebase/TESTING.md` - Validation commands and Phase 8 testing
  gaps for taxonomy media cues and OCR/AI.

### UI And Product Notes
- `.planning/quick/260713-iqt-fix-admin-item-list-visual-alignment-and/SUMMARY.md`
  - Product note recommending curated taxonomy icon/thumbnail assets rather
  than arbitrary item-thumbnail inference.
- `.planning/quick/260717-public-detail-signer-icons/SUMMARY.md` - Current
  public detail signer-row polish and tests.

### Current Implementation Surfaces
- `controller/static-public/templates/detail.html` - Current generated public
  detail page structure.
- `controller/static-public/assets/site.css` - Current detail metadata panel,
  fact chips, signer profile icon, and responsive detail styles.
- `controller/src/publisher.rs` - Static detail generation, public detail
  groups, detail facts, signer rows, public DTO projection, and privacy scans.
- `controller/src/contracts.rs` - Schema version 2 public DTO fields for
  signer/taxonomy data.
- `controller/static-admin/admin.js` - Current signer row, signer suggestions,
  taxonomy suggestion, item form payload, and signer management behavior.
- `controller/src/catalog.rs` - Memory repository signer/taxonomy behavior,
  signer suggestion matching, signer profile creation/reuse, and edit history.
- `controller/src/oracle_catalog.rs` - Oracle signer profile creation/reuse and
  persistence behavior.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `controller/static-public/templates/detail.html`: Provides the generated
  detail page shell with title/signers, image viewer, and revealed metadata
  panel where the taxonomy cue row should land.
- `controller/static-public/assets/site.css`: Provides detail metadata panel,
  fact chip, signer row, profile icon, and responsive layout styling to extend
  carefully.
- `controller/src/publisher.rs`: Generates detail facts/groups, signer rows,
  public artifact paths, privacy validation, and generated static releases.
- `controller/src/contracts.rs`: Defines public schema version 2 fields already
  carrying franchises, productLine, setName, format, origin, language, signer
  names, and signer roles.
- `controller/static-admin/admin.js`: Already supports signer suggestions,
  duplicate warnings, signer row staging, taxonomy datalist suggestions, item
  save payloads, and signer profile management.
- `controller/src/catalog.rs` and `controller/src/oracle_catalog.rs`: Already
  create or reuse signer profiles during item save when signer rows include a
  displayName without signerId.
- Existing tests in `controller/tests/admin_workflow.rs`,
  `controller/tests/static_admin.rs`, `controller/tests/publisher.rs`, and
  `controller/tests/static_contract.rs` cover the relevant route, admin source,
  publisher, and public artifact behavior.

### Established Patterns
- Public output is generated static HTML/JSON/media derivatives and must remain
  published-only and privacy-safe.
- Saving private metadata and publishing public static releases remain separate
  operations.
- Admin collection management uses the single-admin HTTP-only session-cookie
  path and same-origin `/admin/api/*` calls.
- Static admin remains plain HTML/CSS/JavaScript with DOM node creation and
  text rendering, not a frontend build system.
- Public detail pages hide boring defaults such as `Language: English` and
  `Origin: Official`.
- Live Oracle/Object Storage proofs remain operator-run because they require
  real credentials and tenancy state.

### Integration Points
- Add cue asset persistence and approval state through the controller/catalog
  boundary.
- Add admin cue preview/approval surfaces without exposing candidate assets
  publicly before approval.
- Extend publisher/static contract behavior so approved cues can render in
  public detail metadata and unapproved/missing cues fall back to text.
- Extend public artifact privacy scans to cover cue asset paths and source
  metadata.
- Add OCR/AI suggestion route(s) behind the admin session boundary.
- Add admin suggestion review panel and inline accept/ignore affordances that
  stage values in the existing form.
- Extend edit history to mark accepted AI-assisted values lightly.

</code_context>

<specifics>
## Specific Ideas

- The first detail-page cue row should feel like a row of visual tags, each with
  hover/focus identification such as `Franchise: Star Wars` or
  `Product line: Star Wars CCG`.
- Cue row ordering should generally be franchise cue(s), product line cue, set
  cue/text when useful, then non-default language cue.
- Generic set names such as `Base Set` should not force a fake thumbnail. Show
  small text or omit the set cue if the product-line cue already carries the
  useful identity.
- Card-back or product-line cues may be rectangular; do not design only for
  square chips.
- Web/research-assisted cue image discovery can happen outside the app at first,
  with final upload/approval happening through the admin workflow.
- AI suggestion examples include `High: matched existing signer name`,
  `Medium: visible card text resembles Star Wars CCG`, and
  `Low: possible set, needs review`.
- For completely new items, AI may propose new signer/franchise/product/set
  values, but those proposals are staged and explicitly accepted or ignored.

</specifics>

<deferred>
## Deferred Ideas

- A full in-app web image search interface for cue assets is deferred. Use
  research-assisted discovery and manual upload fallback first.
- Stylized or symbolic cue fallback can be added after curated thumbnails are
  tested against real franchise/product/set/language coverage gaps.
- Multi-image AI comparison is deferred; the first pass analyzes the primary
  image by default with optional alternate image selection.
- Broader public browse facet/card visual redesign is deferred unless it falls
  out naturally from the detail-page cue model.

</deferred>

---

*Phase: 8-Taxonomy Media and AI-Assisted Ingest*
*Context gathered: 2026-07-20*
