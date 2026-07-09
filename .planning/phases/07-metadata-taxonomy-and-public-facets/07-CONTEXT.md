# Phase 7: Metadata Taxonomy and Public Facets - Context

**Gathered:** 2026-07-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 7 upgrades the catalog model and generated public facets so the collection can represent real autograph items without duplicating physical items or overloading `signer`, `category`, and `tags`.

This phase covers reusable signer profiles, multi-signer item credits, optional signer enrichment links, first-class character/format/franchise/product-line/set/origin/language fields, reviewed migration/backfill from the current live tag/category drift, admin create/edit UI updates, public static JSON/facet changes, public collection/detail rendering, full rebuild verification, and legacy-field deprecation planning.

This phase does not add AI/OCR metadata suggestions, bulk import, public accounts, multi-admin roles, social features, direct Object Storage URLs, or a split public frontend/backend architecture. AI-assisted ingest moves to Phase 8 after the richer manual metadata model exists.

</domain>

<decisions>
## Implementation Decisions

### Signer Model
- **D-07-01:** Use reusable signer profiles and item-specific signer credits. A single signer profile can link to multiple items, and a single item can have multiple signer credits.
- **D-07-02:** Common roles live on the reusable signer profile, with optional item-specific role/context on the signer credit when needed.
- **D-07-03:** Store optional Wikipedia and IMDb URLs on signer profiles. Public item detail pages should show these as small site icons next to the signer name, not as full URLs. Collection cards should not show these links.
- **D-07-04:** Collection cards should render compact signer text: `A + B` for two signers and `A, B + N more` for larger groups. Detail pages and accessible labels should carry the full signer list.
- **D-07-05:** Admin signer entry should use inline create/select with typeahead reuse. Selecting an existing signer reuses it; typing a new name can create a signer record on save.
- **D-07-06:** The signer flow must include typo/duplicate guardrails: near-duplicate warnings on save and a minimal signer edit/merge repair path so typo-created duplicates can be consolidated without editing each item manually.

### Taxonomy Fields
- **D-07-07:** `title` remains the required flexible public display title.
- **D-07-08:** Add first-class `characters[]` because current and expected items have at least one character. Game/product line and set should not be crammed into `title` by default.
- **D-07-09:** Add optional first-class `set`/`setName`. It is useful for trading cards and can remain blank for comic books or formats where it does not apply.
- **D-07-10:** Use a single optional product line per item. Custom cards still use the relevant product line, with `Set: Custom` and `Origin: Custom`.
- **D-07-11:** Allow multiple franchises per item so crossovers or unusual items do not need hacks. The admin UI should still make the common one-franchise case easy.
- **D-07-12:** Keep `tags[]` for loose extras, but do not use tags as main collection-page filters. With first-class taxonomy fields, each item should need fewer loose tags.
- **D-07-13:** Add first-class single-value `language`, defaulting to `English`. Backfill `Japanese` tags to `language = Japanese`; future Chinese cards use `language = Chinese`.
- **D-07-14:** Keep `format` as a controlled single value, replacing public `Category` semantics. Expected values include `Trading Card`, `Comic Book`, and future physical formats such as photo, poster, book, or art print.
- **D-07-15:** Keep `origin` as a controlled official/custom value. Admin should present this as a simple `Custom item` checkbox defaulting to false.

### Migration Path
- **D-07-16:** Generate a conservative migration review/report first, with an operator-reviewable PL/SQL script for live data changes.
- **D-07-17:** The operator may run the generated PL/SQL manually through SQL Developer before merge/deploy when appropriate.
- **D-07-18:** Use a temporary committed mapping file for repeatable review/backfill, but treat it as a Phase 7 migration artifact rather than permanent runtime taxonomy config. Archive or summarize it after live migration and verification.
- **D-07-19:** Likely duplicate multi-signer workaround records should be report-only and handled manually. Current production likely has only one two-signer duplicated item; the new model should prevent future duplication.
- **D-07-20:** Production rollout should be schema/data first, then deploy the full application update, then full rebuild: generate/review report and PL/SQL, run PL/SQL manually, merge/deploy controller/admin/public code that reads/writes/renders the new model, run full static publish, and verify admin editing plus public facets/detail pages against migrated data.
- **D-07-21:** Keep legacy columns temporarily for safety/backward reference through Phase 7, but stop using them for new public facets. Include an explicit deprecation/cleanup step in the overall Phase 7 plan.

### Public and Admin Presentation
- **D-07-22:** Keep the current public collection filter hide/reveal UX. Phase 7 should add richer facets inside that existing collapsible pattern rather than redesigning the collection page interaction.
- **D-07-23:** Primary public collection filters should be Signer, Franchise, Product Line, Format, and Language.
- **D-07-24:** Secondary/advanced public filters should be Origin, Role, and Tags.
- **D-07-25:** Language facet values remain semantic text values such as `English`, `Japanese`, and `Chinese` for JSON, URLs, query params, and metadata. The dropdown UX can render country flags for compactness: English -> US flag, Japanese -> Japanese flag, Chinese -> Chinese flag, with accessible labels/title text.
- **D-07-26:** Public detail pages should hide boring defaults such as `Language: English` and `Origin: Official`, but show meaningful non-defaults such as `Language: Japanese`, `Language: Chinese`, and `Origin: Custom`.
- **D-07-27:** Existing optional detail metadata behavior remains: description, inscription, event/source/location/year/certification fields are shown only when present.
- **D-07-28:** Admin editor should use three sections: Identity (`Title`, `Characters`, `Signers`), Classification (`Format`, `Franchise`, `Product Line`, `Set`, `Language`, `Custom item`), and Details (`Description`, `Inscription`, `Event/source/location/year/certification`, `Loose tags`).
- **D-07-29:** Taxonomy fields should suggest existing values inline from current data while allowing new values to be typed in the item editor. Do not require a separate taxonomy-management screen for routine entry.

### the agent's Discretion
- Exact database table/column names, provided the model supports reusable signer profiles, item signer credits, first-class taxonomy fields, and temporary legacy-field safety.
- Exact near-duplicate matching approach for signer warnings, provided it catches likely typos without blocking deliberate new signer creation.
- Exact admin control implementation, provided the three-section editor, inline reuse/create flow, and common-path efficiency are preserved.
- Exact public facet layout inside the existing hide/reveal pattern, provided the primary/secondary facet split and accessible language rendering are preserved.
- Exact migration report format and PL/SQL generation mechanics, provided the operator can review mappings before live data changes and the phase records what was applied.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Decisions
- `.planning/ROADMAP.md` - Phase 7 goal, dependencies, and success criteria; Phase 8 AI ingest dependency.
- `.planning/REQUIREMENTS.md` - Existing data, gallery, admin, static, and AI requirement context that Phase 7 refines before AI ingest.
- `.planning/PROJECT.md` - Product value, constraints, and out-of-scope boundaries.
- `.planning/STATE.md` - Current project state, Phase 7 focus, and roadmap evolution.
- `.planning/phases/07-metadata-taxonomy-and-public-facets/07-BRIEF.md` - Phase 7 brief, current live taxonomy signals, candidate model, and open questions.

### Prior Phase Decisions
- `.planning/phases/06-admin-collection-workflow/06-CONTEXT.md` - Admin workflow, save/publish separation, edit history, security, diagnostics, and static admin constraints to preserve.
- `.planning/phases/05-static-runtime-migration-foundation/05-CONTEXT.md` - Static runtime, public JSON, private controller, publisher, media privacy, and full rebuild/repair decisions.

### Codebase Maps
- `.planning/codebase/STACK.md` - Current Rust/static runtime stack and validation expectations. Note: it predates the Phase 7 roadmap insertion, so its Phase 7 AI wording is stale.
- `.planning/codebase/ARCHITECTURE.md` - Current static-public/Rust-controller architecture and privacy boundaries. Note: it predates the Phase 7 roadmap insertion, so its Phase 7 AI wording is stale.
- `.planning/codebase/CONVENTIONS.md` - Rust/static/admin conventions, testing habits, privacy rules, and documentation habits. Note: it predates the Phase 7 roadmap insertion, so its Phase 7 AI wording is stale.
- `.planning/codebase/STRUCTURE.md` - Repository structure and likely locations for schema, controller, publisher, static admin, static public, and test changes. Note: it predates the Phase 7 roadmap insertion, so its Phase 7 AI wording is stale.

### Live Taxonomy Evidence
- `facets.json` - Downloaded live `/data/facets.json` used during discussion to identify overloaded tag/category values. This is an untracked local evidence file unless planning later chooses to commit a sanitized snapshot.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `controller/src/catalog.rs` contains the current item model, update model, edit history, pending-change behavior, and in-memory repository patterns that Phase 7 will extend.
- `controller/src/oracle_catalog.rs` and `controller/src/oracle_schema.rs` contain production persistence and schema validation that Phase 7 will extend for signer profiles, signer credits, taxonomy fields, and migration safety.
- `controller/db/schema.sql` and `controller/db/updates/` are the schema and update-script surfaces for Phase 7 persistence changes.
- `controller/src/contracts.rs` defines current public DTOs and facets; Phase 7 should introduce the new public schema shape deliberately.
- `controller/src/publisher.rs` generates public collection/detail/facet JSON and validates public-safe output.
- `controller/static-public/assets/browse.js` renders the current public collection filters inside the hide/reveal UX that Phase 7 should preserve.
- `controller/static-admin/index.html`, `controller/static-admin/admin.js`, and `controller/static-admin/admin.css` implement the current admin editor and should evolve within the plain static HTML/CSS/JS pattern.
- `controller/tests/admin_workflow.rs`, `controller/tests/publisher.rs`, `controller/tests/static_admin.rs`, `controller/tests/static_contract.rs`, and `controller/tests/live_static_publish_smoke.rs` provide current verification patterns.

### Established Patterns
- Public static output must be generated inside the OCI/runtime boundary and fail closed if validation finds incomplete or privacy-leaking artifacts.
- Public artifacts must not expose private Object Storage URLs, bucket names, namespaces, object keys, Oracle internals, image UUIDs, credentials, or unpublished records.
- Admin collection management remains single-admin, same-origin, session-cookie authenticated, and framework-free.
- Saving private metadata and publishing public static releases remain separate operations.
- Full rebuild remains explicit and appropriate for structural schema/template changes.
- Live Oracle/Object Storage verification remains operator-run because it requires real credentials and tenancy state.

### Integration Points
- Admin API item create/update/list/detail routes need to read/write the new metadata model without leaking private internals.
- Static publisher needs schema-versioned public DTO/facet generation for signer credits, characters, franchises, product line, set, format, origin, language, and loose tags.
- Public collection JS needs to filter multi-value fields by "item has value" while combining different facet groups with AND.
- Public detail generation needs compact default-hiding rules and signer link icon rendering.
- Admin editor needs three-section layout, inline suggestions/reuse, near-duplicate signer warnings, and signer merge/edit repair.
- Migration/backfill tooling needs a temporary mapping artifact, conservative report generation, PL/SQL generation, and a documented manual SQL Developer/live rollout path.

</code_context>

<specifics>
## Specific Ideas

- Current live tag examples should move into first-class fields: franchise-like values (`Star Wars`, `Star Trek`, `Monty Python`), product-line values (`Young Jedi`, `Force Attax`, `Lorcana`, `Magic: The Gathering`), role values (`actor`, `artist`, `author`, `game designer`, `voice actor`), and language values (`Japanese`).
- `custom` means self-designed/self-printed origin, not a card-specific format. A custom card can still be `Format: Trading Card`, `Product Line: Star Wars CCG`, `Set: Custom`, and `Origin: Custom`.
- The current category values `Tr`, `Tra`, and `Trading Card` should map to `Format: Trading Card`.
- Expected language values include `English`, `Japanese`, and `Chinese`.
- Expected format values include at least `Trading Card` and `Comic Book`.
- Product line should be single-value and optional; franchise should be multi-value.
- Item title should stay the public display title even when `characters[]` carries the character data.
- The current public collection filter hide/reveal UX should remain.

</specifics>

<deferred>
## Deferred Ideas

- AI/OCR metadata suggestions remain Phase 8.
- Bulk import remains out of scope.
- Public signer profile pages such as `/signers/{slug}/` are not required for Phase 7; signer profile links can appear on item detail pages first.
- Automatic item merging for likely duplicate physical items is deferred; Phase 7 should report likely duplicates for manual cleanup.
- Removing legacy columns immediately is deferred; Phase 7 should include deprecation/cleanup planning while keeping legacy fields temporarily.

</deferred>

---

*Phase: 7-Metadata Taxonomy and Public Facets*
*Context gathered: 2026-07-05*
