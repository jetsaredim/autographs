# Phase 8: Taxonomy Media and AI-Assisted Ingest - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-20
**Phase:** 8-Taxonomy Media and AI-Assisted Ingest
**Areas discussed:** Taxonomy Cue Scope, Cue Creation And Approval, Admin/Public Placement, AI Suggestion Flow

---

## Taxonomy Cue Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Event-ready starter set | Prioritize product-line/set cues useful for expected upcoming event items. | ✓ |
| Current catalog gaps | Prioritize values already present in the catalog. | ✓ |
| Trading-card-first | Focus first on card backs, sets, and product lines. | |
| Tiny curated pilot | Pick only a handful of high-confidence values and prove the workflow. | |

**User's choice:** Event-ready starter set plus current catalog gaps.
**Notes:** The user is attending an event soon and expects new signed items that can test Phase 8. The first cue set should be useful for that event but grounded against existing catalog values where possible.

### Eligible Cue Types

| Option | Description | Selected |
|--------|-------------|----------|
| Product line only | Simple and broad. | |
| Product line + set | Supports broad families and specific sets. | |
| Set only when product line exists | Avoids free-floating set cues. | ✓ |
| Agent decides | Planner chooses the narrowest useful model. | |

**User's choice:** Set cues only when product line is filled in, expanded to include franchise and language when useful.
**Notes:** Franchise cues are in scope. Language cues should appear when meaningful/non-default. Format should not get a cue and should be minimized because viewers can usually tell whether an item is a trading card, comic, etc.

### Cue Feel

| Option | Description | Selected |
|--------|-------------|----------|
| Curated image thumbnails | Small approved images such as card-back cues or franchise/product-line thumbnails. | ✓ |
| Symbolic icons | Simple safer icons/badges. | |
| Mixed | Use curated thumbnails when available, otherwise text/icon fallback. | |

**User's choice:** Start with curated image thumbnails.
**Notes:** The user wants to start with the richest/most collectible-feeling version, while keeping room to switch to more stylized/icon fallback if safe image coverage becomes difficult.

---

## Cue Creation And Approval

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated uploaded cue assets | Operator uploads/approves specific cue thumbnails. | ✓ |
| Derived from existing item images | Use approved crops from item images. | |
| Both, but explicit | Allow uploads and derived crops with explicit approval. | |

**User's choice:** Mostly dedicated uploaded assets or web-sourced candidates.
**Notes:** Most items do not have card backs or franchise/product images, so existing item photos should not be assumed to contain useful cue material.

### Discovery

| Option | Description | Selected |
|--------|-------------|----------|
| Manual only | Operator finds/downloads/uploads assets manually. | ✓ fallback |
| Research-assisted | App/planning workflow helps find candidates; operator approves final asset manually. | ✓ preferred |
| Admin candidate search | Admin UI includes in-app candidate image search. | |

**User's choice:** Research-assisted discovery plus manual upload fallback.
**Notes:** The first pass does not need a full in-app web image search product. Candidate imagery must be reviewed before public use.

### Approval

| Option | Description | Selected |
|--------|-------------|----------|
| Approved only | Only approved cue assets publish. | |
| Approved + local preview | Candidate assets can appear privately, but public output includes approved assets only. | ✓ |
| Soft warning | Candidate assets can publish with warning state. | |

**User's choice:** Approved + local preview.
**Notes:** Private/admin preview can show candidate cue assets, but generated public output must include explicitly approved cue assets only.

---

## Admin/Public Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Below title/signers | Prominent first-viewport identity strip. | |
| Above title | Context before item title. | |
| Inside revealed metadata panel | Preserves current image-first detail page feel. | ✓ |
| Near image/gallery | More visual, but risks competing with the autograph. | |

**User's choice:** Inside revealed metadata panel.
**Notes:** The signed item remains the hero. The taxonomy identity row appears after metadata reveal.

### Metadata Position

| Option | Description | Selected |
|--------|-------------|----------|
| At top of metadata | First thing inside revealed panel. | |
| After signers | Signers remain first metadata group. | ✓ |
| Inside classification | Replaces/enriches existing Classification group. | ✓ |

**User's choice:** After signers plus integrated with classification.
**Notes:** Signers remain the most important metadata after the image. Taxonomy identity should enrich or replace the plain Classification group.

### Labeling

| Option | Description | Selected |
|--------|-------------|----------|
| Labeled | Field labels remain visible. | |
| Visual first | Thumbnail/value row with labels secondary. | ✓ |
| Hybrid | Visible labels where needed and compact chips where obvious. | ✓ |

**User's choice:** Media-first visual tag row.
**Notes:** The row should feel like a listing of visual taxonomy tags. Hover/focus labels and accessible names identify each cue; mobile/accessibility cannot rely on hover only. Text fallback remains mandatory.

### Asset Shape

| Option | Description | Selected |
|--------|-------------|----------|
| One asset | One approved thumbnail/icon per value. | |
| One image + focal metadata | One approved image with focal/crop preference. | ✓ |
| Multiple variants | Square/wide/light/dark variants from the start. | |

**User's choice:** One image plus focal/crop metadata.
**Notes:** Card backs are not square and should not be forced into square chips. Rendering should support constrained-height, flexible-width rectangular visual tags.

### Cue Model

| Option | Description | Selected |
|--------|-------------|----------|
| Shared cue model | One cue system keyed by type and value. | |
| Separate fields per type | Each taxonomy type owns cue fields. | |
| Hybrid | Shared storage with type-specific rules. | ✓ |

**User's choice:** Hybrid.
**Notes:** Shared storage keeps implementation sane, while type-specific rules handle set/product-line dependency and language fallback behavior.

---

## AI Suggestion Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Taxonomy consistency hints | Suggest existing/recent values from typed fields. | |
| Image-assisted suggestions | Use uploaded image/OCR explicitly. | ✓ |
| Research-assisted metadata | Help find likely product/set/franchise info from research. | |

**User's choice:** Image/OCR in the first AI pass.
**Notes:** The original project intent includes admin-side AI to improve data consistency and new-item ingest. AI should be advisory and consistency-focused, not an automatic ingestion engine.

### Image Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Selected image only | Admin chooses the best image. | |
| Primary image by default | Use primary image unless another is chosen. | ✓ |
| All images | Send all attached images for comparison. | |

**User's choice:** Primary image by default.
**Notes:** Most items will only have one image, so do not overbuild multi-image AI comparison.

### Result UI

| Option | Description | Selected |
|--------|-------------|----------|
| Field-by-field suggestions | Each form field receives local suggestions. | |
| Review panel | One grouped panel lists suggestions. | |
| Inline + panel | Summary panel plus inline accept/ignore near fields. | ✓ |

**User's choice:** Inline + panel.
**Notes:** Suggestions should be grouped into existing matches, new candidates, warnings/consistency issues, and OCR text. Inline controls keep acceptance close to the relevant fields.

### Persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Stage until Save | Accepted suggestions fill the form but do not persist until item save. | ✓ |
| Create immediately | Accepting creates records immediately. | |
| Depends on type | Some types create immediately; others stage. | |

**User's choice:** Stage until Save.
**Notes:** New AI-assisted processes should not bypass the existing signer row flow. New signer suggestions stage a display name in the item form; the current save path creates/reuses signer profiles and duplicate warnings/merge repair remain the guardrails.

### Confidence And History

| Option | Description | Selected |
|--------|-------------|----------|
| Confidence only | Show high/medium/low labels. | |
| Rationale only | Show short reasons. | |
| Confidence + rationale | Show both compactly when available. | ✓ |
| No confidence | Just show suggestions. | |

**User's choice:** Confidence + rationale.
**Notes:** Especially important for new values, OCR/inscription text, and consistency warnings.

| Option | Description | Selected |
|--------|-------------|----------|
| No special history | Accepted suggestions become normal field edits. | |
| History marks AI-assisted | Edit history notes accepted AI-assisted values. | ✓ |
| Only audit suggestion run | Record suggestion generation, not accepted fields. | |

**User's choice:** History marks AI-assisted.
**Notes:** Keep this lightweight and aligned with existing single-admin edit history.

---

## the agent's Discretion

- Exact first-pass cue count and starter values.
- Exact visual styling of the revealed metadata cue row.
- Exact cue asset persistence schema.
- Exact AI provider and prompt details, pending research/security review.
- Exact edit-history wording for AI-assisted accepted values.

## Deferred Ideas

- Full in-app web image search for cue assets.
- Stylized/icon fallback after curated thumbnail coverage is tested.
- Multi-image AI comparison.
- Broad public browse/card visual redesign unless it falls out naturally from the detail-page cue model.
