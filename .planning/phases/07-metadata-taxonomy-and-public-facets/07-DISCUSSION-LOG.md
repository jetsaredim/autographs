# Phase 7: Metadata Taxonomy and Public Facets - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-05
**Phase:** 07-metadata-taxonomy-and-public-facets
**Areas discussed:** Signer model, Taxonomy fields, Migration path, Public/admin presentation

---

## Signer Model

| Option | Description | Selected |
|--------|-------------|----------|
| Profile-level roles | A signer has general roles like Actor, Author, Voice Actor reused everywhere. | |
| Item-specific roles | The same signer can have a different role per item. | |
| Both | Store common roles on signer profile, allow item-specific role/context when needed. | ✓ |

**User's choice:** Both.
**Notes:** Common path should stay simple; exceptions need an honest place to live.

| Option | Description | Selected |
|--------|-------------|----------|
| Store links only | Store optional Wikipedia/IMDb URLs but do not show them yet. | |
| Store and show on item detail | Store links and render them on public detail pages. | ✓ |
| Store for future signer pages | Store links mainly for later signer profile pages. | |

**User's choice:** Store and show on item detail.
**Notes:** Use icons for the respective sites next to the signer name, not full links. Do not show these on collection cards.

| Option | Description | Selected |
|--------|-------------|----------|
| Compact text | `A + B`; `A, B + N more`. | ✓ |
| Primary signer only | First signer plus count. | |
| All names until wrap | Show all names on the card. | |

**User's choice:** Compact text.
**Notes:** Detail pages and accessible labels carry the full signer list.

| Option | Description | Selected |
|--------|-------------|----------|
| Inline create/select | Typeahead reuse, create new signer on save. | ✓ |
| Separate signer management first | Maintain signer profiles separately before attaching. | |
| Name-first now, dedupe later | Keep plain names for now. | |

**User's choice:** Inline create/select with guardrails.
**Notes:** User raised typo/deduplication risk. Decision adds typeahead reuse, near-duplicate warnings, and minimal signer edit/merge repair.

---

## Taxonomy Fields

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as title for now | `title` remains the only character/display field. | |
| Add characters[] now | Character becomes first-class. | ✓ |
| Hybrid | Keep title and optional characters without primary character facet. | |

**User's choice:** Add characters[] now, while keeping title as flexible display title.
**Notes:** All items have at least one character. Game/product line and set should not be crammed into title by default.

| Option | Description | Selected |
|--------|-------------|----------|
| Add set now | Optional first-class set field. | ✓ |
| Defer set | Add later after product line. | |
| Use loose tags for set | Avoid schema field. | |

**User's choice:** Add optional set now.
**Notes:** Useful for trading cards; blank for comic books and other formats where not applicable.

| Option | Description | Selected |
|--------|-------------|----------|
| Single product line per item | One optional product/game/product-line value. | ✓ |
| Multiple product lines per item | Multi-value product lines. | |
| Start single, allow migration later | Single now, future migration if needed. | |

**User's choice:** Single product line per item.
**Notes:** Custom cards use relevant product line plus `Set: Custom` and `Origin: Custom`.

| Option | Description | Selected |
|--------|-------------|----------|
| Single franchise per item | One franchise/universe. | |
| Multiple franchises per item | Supports crossovers/oddballs. | ✓ |
| Infer franchise from product line | Less entry, more hidden logic. | |

**User's choice:** Multiple franchises per item.
**Notes:** Common one-franchise case should still be easy in admin.

| Option | Description | Selected |
|--------|-------------|----------|
| Advanced loose labels only | Tags remain for leftovers, not main taxonomy. | ✓ |
| Private admin-only notes/labels | Tags do not show publicly. | |
| Remove tags entirely | Everything maps to structured fields. | |

**User's choice:** Keep tags for loose extras but not as main collection filters.
**Notes:** With structured fields, tag list should be smaller per item.

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as tag | Language remains a loose tag. | |
| Add first-class language | Single language field, default English. | ✓ |
| Make language a primary public filter | Also prominent in public filters. | |

**User's choice:** Add first-class language.
**Notes:** Existing Japanese cards and future Chinese cards make language important enough to promote out of tags. Public collection filter later includes Language as primary.

---

## Migration Path

| Option | Description | Selected |
|--------|-------------|----------|
| Conservative report first | Generate proposed mapping/report before live changes. | ✓ |
| Auto-apply obvious mappings | Apply clear mappings, report uncertain ones. | |
| Fully automatic | Apply all known mappings directly. | |

**User's choice:** Conservative report first.
**Notes:** Generate operator-reviewable PL/SQL. User can run it through SQL Developer before merge/deploy.

| Option | Description | Selected |
|--------|-------------|----------|
| Committed mapping config | Temporary mapping file for repeatable review/backfill. | ✓ |
| Hardcoded script logic | Mappings live inside script. | |
| Manual SQL only | Hand-write migration. | |

**User's choice:** Temporary committed mapping file.
**Notes:** Not permanent runtime taxonomy config; archive/summarize after live migration and verification.

| Option | Description | Selected |
|--------|-------------|----------|
| Report only | Detect likely duplicate workaround records but do not merge. | ✓ |
| Admin merge tool in Phase 7 | Build item merge UI/API. | |
| Manual SQL migration only | Handle known duplicates entirely outside app. | |

**User's choice:** Report only.
**Notes:** Current production likely has one two-signer duplicated item; larger multi-signer cases are mostly future entries.

| Option | Description | Selected |
|--------|-------------|----------|
| Schema/data first, then deploy code | Manual PL/SQL, deploy code, full rebuild. | ✓ |
| Deploy backward-compatible code first | Code supports old/new shape. | |
| One deploy does everything | Deployment runs migration/backfill/code. | |

**User's choice:** Schema/data first, then full application update, then full rebuild.
**Notes:** The deployed code step includes admin API/UI, publisher, and public UI changes.

| Option | Description | Selected |
|--------|-------------|----------|
| Keep legacy columns temporarily | Safety/backward reference. | ✓ |
| Remove/deprecate immediately | Cleaner but riskier. | |
| Keep forever | Simple but confusing. | |

**User's choice:** Keep temporarily.
**Notes:** Include explicit deprecation/cleanup step in Phase 7 plan.

---

## Public/Admin Presentation

| Option | Description | Selected |
|--------|-------------|----------|
| Signer, Franchise, Format | Focused primary filters. | |
| Signer, Franchise, Product Line, Format | More powerful but busier. | |
| Custom primary set | Signer, Franchise, Product Line, Format, Language. | ✓ |

**User's choice:** Primary filters are Signer, Franchise, Product Line, Format, and Language.
**Notes:** Keep current public collection filter hide/reveal UX. Origin, Role, and Tags are secondary/advanced.

| Option | Description | Selected |
|--------|-------------|----------|
| Icons/badges plus text | Compact but accessible. | ✓ |
| Text only | Simplest. | |
| Icons only | Compact but ambiguous. | |

**User's choice:** Semantic values with flag UX.
**Notes:** Language facet values remain text (`English`, `Japanese`, `Chinese`) for JSON/URLs/query params. Dropdown can render country flags for compactness with accessible labels.

| Option | Description | Selected |
|--------|-------------|----------|
| Hide boring defaults | Hide `English`/`Official`; show non-defaults. | ✓ |
| Always show all structured metadata | Complete but noisier. | |
| Compact metadata table with all fields | Complete but dense. | |

**User's choice:** Hide boring defaults.
**Notes:** Existing optional metadata behavior remains for description, inscription, event/source/location/year/certification.

| Option | Description | Selected |
|--------|-------------|----------|
| Three sections | Identity, Classification, Details. | ✓ |
| One long form | Simple implementation, more clutter. | |
| Tabs/accordion sections | Cleaner but more navigation. | |

**User's choice:** Three sections.
**Notes:** Identity: Title, Characters, Signers. Classification: Format, Franchise, Product Line, Set, Language, Custom item. Details: Description, Inscription, Event/source/location/year/certification, Loose tags.

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing values inline | Suggestions from existing values, allow new entries. | ✓ |
| Controlled vocab only | Separate management required for new values. | |
| Free text only | Repeats drift risk. | |

**User's choice:** Reuse existing values inline.
**Notes:** Do not require a separate taxonomy-management screen for routine entry.

---

## the agent's Discretion

- Exact schema names and internal DTO shapes.
- Exact near-duplicate signer warning algorithm.
- Exact admin control implementation, as long as the locked grouping and flow hold.
- Exact public filter layout inside the existing hide/reveal pattern.
- Exact migration report/PLSQL generation mechanics.

## Deferred Ideas

- AI/OCR metadata suggestions remain Phase 8.
- Public signer profile pages are not required for Phase 7.
- Automatic item merging is deferred; report likely duplicates for manual cleanup.
