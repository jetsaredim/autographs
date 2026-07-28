# Phase 10: Advisory AI-Assisted Ingest - Context

**Gathered:** 2026-07-20
**Realigned:** 2026-07-28
**Status:** Future phase; refresh after Phase 8 and Phase 9

<domain>
## Phase Boundary

Phase 10 adds explicit, advisory OCR/AI metadata suggestions to the existing
single-admin upload/edit workflow. It comes after the manual admin workflow,
richer taxonomy model, admin media preview/adjustment foundation, and optional
taxonomy media cue layer.

This phase does not add automatic catalog records, automatic publishing, public
AI surfaces, public accounts, multi-admin roles, bulk import, or a split public
frontend/backend service.

</domain>

<decisions>
## Implementation Decisions

### AI Suggestion Flow
- **D-10-01:** AI remains admin-side and advisory. Its role is to improve
  metadata consistency and ingest speed, not to replace the operator's
  knowledge.
- **D-10-02:** First AI pass should include explicit image/OCR analysis, not
  only typed-field consistency checks.
- **D-10-03:** AI suggestion should be an explicit admin action such as
  `Suggest / Check metadata`, not an automatic background save/publish path.
- **D-10-04:** OCR/AI analysis uses the primary image by default, with an option
  to choose another image when needed. Do not design the first pass around
  multi-image AI comparison.
- **D-10-05:** AI suggestions may propose signer credits, character/title,
  franchise, product line, set, language, origin, loose tags, and
  inscription/OCR text.

### Review And Persistence
- **D-10-06:** AI suggestion UI should combine a review panel summary with
  inline accept/ignore controls near relevant fields.
- **D-10-07:** AI must distinguish reuse of existing values from newly proposed
  values. New signers, franchises, product lines, and sets must be visibly
  marked as new candidates.
- **D-10-08:** Accepting AI suggestions stages values in the existing admin form
  until Save. AI must not invent a separate immediate-create process for
  signers or taxonomy values.
- **D-10-09:** AI-assisted signer suggestions should use the existing signer row
  flow. The current save path creates or reuses signer profiles, and duplicate
  warnings/merge repair remain guardrails.
- **D-10-10:** AI suggestions should show compact confidence and rationale when
  available.
- **D-10-11:** Accepted AI-assisted values should be lightly marked in item edit
  history without expanding into a broad audit system.
- **D-10-12:** Manual item entry must remain fully functional when OCR/AI is
  unavailable, inaccurate, unconfigured, slow, or ignored.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` - Phase 10 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - `AI-02` through `AI-05`.
- `.planning/PROJECT.md` - Product constraints and phase split.
- `.planning/STATE.md` - Current project state and accumulated decisions.
- `.planning/phases/08-admin-media-review-and-operational-posture/08-CONTEXT.md`
  - Admin image preview and image-selection foundation.
- `.planning/phases/09-taxonomy-media-cues/09-CONTEXT.md` - Taxonomy media cue
  decisions and public-safe asset approval model.
- `.planning/phases/07-metadata-taxonomy-and-public-facets/07-CONTEXT.md`
  - Reusable signer profiles and taxonomy fields AI suggestions should target.
- `controller/static-admin/admin.js`
- `controller/src/routes.rs`
- `controller/src/catalog.rs`
- `controller/src/oracle_catalog.rs`
- `controller/tests/admin_workflow.rs`
- `controller/tests/static_admin.rs`

</canonical_refs>

<deferred>
## Deferred Ideas

- Multi-image AI comparison.
- Automatic save or automatic publish from AI output.
- Public AI explanations or public AI-generated metadata badges.
- Bulk import.

</deferred>

---

*Phase: 10-Advisory AI-Assisted Ingest*
*Context gathered: 2026-07-20; realigned: 2026-07-28*
