# Phase 07 Brief: Metadata Taxonomy and Public Facets

## Purpose

Upgrade the catalog model so it reflects the real collection rather than forcing unrelated concepts into `signer`, `category`, and freeform `tags`.

The live public facets show that tags are currently carrying IP/franchise, game or product line, signer role, custom/official origin, and miscellaneous attributes. The collection also includes cases where one physical item has multiple signers, and some signers appear across several separate items. Phase 7 should make those relationships first-class before AI-assisted ingest starts suggesting metadata.

## Current Signals

- The current public facet groups are `signer`, `category`, and `tag`.
- The live `category` values include partial entries such as `Tr` and `Tra` alongside `Trading Card`, which indicates free-text taxonomy drift.
- The live `tag` values include franchise-like values (`Star Wars`, `Star Trek`, `Monty Python`), product-line values (`Young Jedi`, `Force Attax`, `Lorcana`, `Magic: The Gathering`), role values (`actor`, `artist`, `author`, `game designer`, `voice actor`), and attributes (`custom`, `Japanese`, `Player's Committee`, `Virtual Slip`).
- Some items are currently duplicated per signer when the underlying physical item has multiple signatures.
- Character names are often being used as the item title/display name, which is distinct from the signer identity.
- At least some signers appear on multiple items, so reusable signer records would reduce repeated entry and inconsistent spelling.
- Signer enrichment links such as Wikipedia and IMDb URLs would be useful when appropriate, but should remain optional.
- The collection is mostly trading cards today, but signed comic books are expected.

## Candidate Model

- `signerCredits[]`: ordered signer relationships for an item. Each credit links to a reusable signer record and may carry item-specific role/display context later if needed.
- `signers`: reusable people records with display name, optional Wikipedia URL, optional IMDb URL, optional official URL, and optional role metadata.
- `title`: item display title, which may be a character name, card title, comic title, or other collector-facing label.
- `format`: controlled single value such as `Trading Card`, `Comic Book`, `Photo`, `Poster`, `Book`, or `Art Print`.
- `origin`: controlled value `official` or `custom`; admin should present this as a simple `Custom item` checkbox defaulting to false.
- `franchises[]`: multi-value field for visitor-friendly franchise labels such as `Star Wars`, `Star Trek`, and `Monty Python`.
- `productLines[]`: multi-value field for specific games, card lines, or collectible lines such as `Young Jedi`, `Force Attax`, `Lorcana`, or `Magic: The Gathering`.
- `tags[]`: loose advanced extras only, for values that do not deserve first-class fields.

## Public UX Direction

Public filters should be natural collector facets generated from the model:

- Signer
- Franchise
- Format
- Origin
- Product Line
- Role, if populated enough to be useful
- Tags, as a secondary/advanced facet

Multi-signer items should appear once and match any selected signer. Cards can show one signer directly, two signers as `A + B`, and larger groups as `A, B + N more`. Detail pages should show a `Signers` list and include optional role/link metadata where useful.

## Admin UX Direction

Keep the common entry path fast:

- Identity: title plus repeatable signer rows.
- Signer row: existing/new signer name with optional role and optional profile links available without making them required.
- Classification: required format select, `Custom item` checkbox, franchise tokens, product-line tokens.
- Advanced: loose tags and less-common enrichment.

Avoid requiring a separate taxonomy-management screen for the first pass. Suggestions can be derived from existing values, and new values can be entered inline with canonical casing.

## Migration And Backfill Notes

The phase should produce a review report before applying live data changes. Suggested mappings:

- `Tr`, `Tra`, `Trading Card`, and similar category values -> `format = Trading Card`.
- `custom` tag -> `origin = custom`.
- missing origin -> `official`.
- `actor`, `artist`, `author`, `game designer`, `voice actor` -> role metadata.
- franchise-like tags -> `franchises[]`.
- product-line tags -> `productLines[]`.
- ambiguous leftovers remain in `tags[]` until reviewed.

Do not automatically merge duplicate records that might represent one multi-signer physical item. Flag likely duplicates for manual review.

## Open Questions

- Should character names become a first-class `characters[]` field, or should Phase 7 keep them as `title` until a stronger need appears?
- Should signer roles live primarily on the reusable signer record, on the item-signer credit, or both?
- Should Wikipedia/IMDb links be public on item detail pages immediately, or stored first for later signer profile pages?
- Which product-line values should be normalized during backfill versus left as loose tags?

## Review Target

Before implementation, turn this brief into a detailed phase plan with schema, migration/backfill, admin API, static publisher, public UI, verification, and live full-rebuild tasks.
