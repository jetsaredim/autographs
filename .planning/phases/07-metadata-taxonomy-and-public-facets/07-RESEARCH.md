# Phase 07: Metadata Taxonomy and Public Facets - Research

**Researched:** 2026-07-09
**Domain:** Rust controller, Oracle catalog schema migration, generated static public facets, static admin taxonomy UI
**Confidence:** HIGH for codebase scope and locked decisions; MEDIUM for external documentation patterns

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
## Deferred Ideas

- AI/OCR metadata suggestions remain Phase 8.
- Bulk import remains out of scope.
- Public signer profile pages such as `/signers/{slug}/` are not required for Phase 7; signer profile links can appear on item detail pages first.
- Automatic item merging for likely duplicate physical items is deferred; Phase 7 should report likely duplicates for manual cleanup.
- Removing legacy columns immediately is deferred; Phase 7 should include deprecation/cleanup planning while keeping legacy fields temporarily.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DATA-03 | Application records edit history for autograph items so the admin can see what changed over time in v1. | Extend existing field-level edit history to cover signer credits, signer profile changes, taxonomy arrays, and merge events rather than replacing the Phase 6 history model. [VERIFIED: codebase grep] |
| ADMIN-02 | Admin can create a new autograph item by uploading images and reviewing/editing metadata in one workflow before publish. | Keep Phase 6 save/publish/image workflow and replace the old signer/category/tags primary path with Identity, Classification, and Details sections from `07-UI-SPEC.md`. [VERIFIED: codebase grep] |
| ADMIN-03 | Admin can edit an existing autograph item, including metadata and associated images. | Update private item DTOs, filters, edit form hydration, and existing item summary rows for repeatable signers and first-class facets while preserving image and publish behavior. [VERIFIED: codebase grep] |
</phase_requirements>

## Summary

Phase 7 is primarily a catalog data-model and generated-static contract migration, with admin/public UI work layered on top. The existing implementation still stores one `signer`, one `category`, and loose `tags` on `autograph_items`/`autograph_item_tags`, and public contracts expose only signer/category/tag facets at schema version `1`. [VERIFIED: codebase grep] The planner should treat this as a deliberate schema + domain + public JSON version change, not a cosmetic rename. [VERIFIED: codebase grep]

The recommended path is to add normalized signer profile and signer-credit tables, item taxonomy columns/join tables for the locked first-class fields, migration/report tooling, then update Rust DTOs, Oracle adapter, publisher, static public JS, and static admin JS in that order. [VERIFIED: codebase grep] Oracle supports primary, unique, foreign key, check, and not-null constraints; foreign keys model relationships to primary/unique parent keys; indexes provide fast row access with normal B-tree indexes by default. [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/constraint.html] [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/CREATE-INDEX.html]

**Primary recommendation:** Model reusable signers and item signer credits in Oracle first, keep legacy `signer`/`category` columns through Phase 7 as read-only migration references, publish only new schema-versioned public facets, and require a full static rebuild after reviewed live backfill. [VERIFIED: codebase grep]

## Project Constraints (from AGENTS.md)

- Use generated static public artifacts plus one Rust private controller for v1; do not introduce a public split-service platform. [VERIFIED: AGENTS.md]
- Prefer OCI Always Free, Oracle Autonomous Database Free, private OCI Object Storage originals, and generated public-safe derivatives. [VERIFIED: AGENTS.md]
- Auto-deploy from GitHub Actions on merge to `main`; deployment and validation are part of the bootstrap. [VERIFIED: AGENTS.md]
- Keep v1 narrow: no staging environment, no bulk import, no public accounts, no advanced search platform, and no multi-admin roles. [VERIFIED: AGENTS.md]
- Use least-privilege OCI access and explicit secret handling; routine deploy workflows must not require tenancy-wide admin power. [VERIFIED: AGENTS.md]
- Rust is the active implementation language; public/admin surfaces are plain static HTML/CSS/JavaScript unless a later phase intentionally changes that constraint. [VERIFIED: AGENTS.md]
- Keep public static artifacts free of private storage identifiers and unpublished records. [VERIFIED: AGENTS.md]
- Keep persistence/media details in controller adapters and service modules, not route handlers or static assets. [VERIFIED: AGENTS.md]
- Public static output should fail closed during generation/validation rather than publish incomplete or privacy-leaking artifacts. [VERIFIED: AGENTS.md]
- Use Cargo checks for runtime code: `cargo fmt`, `cargo test`, `cargo check --features production-persistence`, and `cargo clippy`. [VERIFIED: AGENTS.md]
- Never commit directly to `main` or `master`; this research was created on `phase-7-metadata-taxonomy-planning`. [VERIFIED: git status]
- `07-UI-SPEC.md` is currently untracked user/workflow output and should be preserved as input, not overwritten by implementation planning. [VERIFIED: git status]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Reusable signer profiles | Database / Storage | API / Backend | Oracle should own signer identity and relationships; Rust validates and exposes safe DTOs. [VERIFIED: codebase grep] |
| Item signer credits | Database / Storage | API / Backend | The many-to-many item/signer relationship needs durable join rows and item-specific context. [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/constraint.html] |
| Taxonomy fields | Database / Storage | API / Backend | Format/origin/language/product line/franchise/characters must persist before admin and publisher can render them. [VERIFIED: codebase grep] |
| Migration/backfill review | API / Backend | Database / Storage | Rust tooling should generate reports/scripts; operator applies PL/SQL to live Oracle per locked rollout. [VERIFIED: 07-CONTEXT.md] |
| Admin editor taxonomy UX | Browser / Client | API / Backend | Static admin JS renders controls and warnings; backend enforces validation and persistence. [VERIFIED: codebase grep] |
| Public facet JSON | API / Backend | CDN / Static | Publisher generates `data/collection.json` and `data/facets.json`; Caddy serves static artifacts. [VERIFIED: codebase grep] |
| Public filter behavior | Browser / Client | CDN / Static | Existing `browse.js` reads static JSON and URL query params in the browser. [VERIFIED: codebase grep] |
| Privacy/fail-closed validation | API / Backend | CDN / Static | Publisher validation already rejects missing derivatives and private terms before promotion. [VERIFIED: codebase grep] |

## Standard Stack

### Core

| Library / Surface | Version | Purpose | Why Standard |
|-------------------|---------|---------|--------------|
| Rust controller crate | local `0.1.0`, Rust edition 2024 | Domain model, routes, publisher, Oracle/media adapters | Existing runtime implementation and project constraint. [VERIFIED: cargo metadata] |
| `axum` | `0.8.9` | Private admin API routing and JSON responses/extractors | Already installed; official docs cover `Json<T>` request/response behavior. [VERIFIED: Cargo.toml] [CITED: https://docs.rs/axum/0.8.9/axum/struct.Json.html] |
| `serde` / `serde_json` | `1.0.228` / `1.0.150` | Camel-case DTO serialization, optional fields, public JSON schema | Already installed; Serde docs support `rename_all`, `default`, `alias`, and `skip_serializing_if`. [VERIFIED: Cargo.toml] [CITED: https://serde.rs/container-attrs.html] [CITED: https://serde.rs/field-attrs.html] |
| Oracle Autonomous Database schema | existing SQL in `controller/db/schema.sql` | Durable catalog, signer, taxonomy, history, publish state | Project-selected production metadata store; existing schema uses Oracle constraints and indexes. [VERIFIED: codebase grep] |
| Static public JS | existing `controller/static-public/assets/browse.js` | URL-backed public filters over generated JSON | Existing public behavior; MDN supports `URLSearchParams` for query-state handling. [VERIFIED: codebase grep] [CITED: https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams] |
| Static admin JS | existing `controller/static-admin/admin.js` | Admin form, item list, save/publish, diagnostics | Phase 6 private admin shell; `07-UI-SPEC.md` requires no frontend framework. [VERIFIED: codebase grep] |

### Supporting

| Surface | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `uuid` | `1.23.4` | IDs for items, images, edit events, new signer/profile rows | Continue for new signer and credit IDs. [VERIFIED: Cargo.toml] |
| `time` | `0.3.51` | Timestamps in controller/publisher output | Continue for generated/public manifest and history timing. [VERIFIED: Cargo.toml] |
| `image` | `0.25.10` | Derivative generation | No taxonomy change needed; keep privacy tests around derivatives. [VERIFIED: Cargo.toml] |
| Oracle update scripts | existing `controller/db/updates/06-03-*.sql`, `06-04-*.sql` | Incremental production schema changes | Add Phase 7 schema/data scripts here, while keeping `schema.sql` as fresh end state. [VERIFIED: codebase grep] |
| Existing tests | `controller/tests/*.rs` | Static contract, publisher, admin workflow, live smoke | Extend these rather than adding a parallel test harness. [VERIFIED: codebase grep] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Normalized signer tables | JSON blob on `autograph_items` | JSON would make duplicate signer repair, merge, and reusable profile links harder and less enforceable. [ASSUMED] |
| First-class taxonomy fields | Continue overloaded tags/category | The phase goal explicitly requires first-class fields and current drift backfill. [VERIFIED: 07-CONTEXT.md] |
| Client-side generated-static facets | Public API filtering | Public APIs/split backend are out of scope; current runtime is Caddy-served static JSON. [VERIFIED: AGENTS.md] |
| Native datalist/light JS controls | React/Vite/shadcn | `07-UI-SPEC.md` forbids new frontend frameworks/build pipeline. [VERIFIED: 07-UI-SPEC.md] |

**Installation:** No new external packages are recommended for Phase 7. [VERIFIED: Cargo.toml] Keep `Cargo.toml` unchanged unless implementation proves an unavoidable need. [ASSUMED]

## Package Legitimacy Audit

No new external packages are recommended or required by this research, so the Package Legitimacy Gate is not triggered. [VERIFIED: Cargo.toml] Existing Rust dependencies are already present in `controller/Cargo.toml`; the planner should add a package-legitimacy checkpoint if it later introduces any new crate. [VERIFIED: cargo metadata]

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| None new | crates.io | — | — | — | — | No install planned |

**Packages removed due to [SLOP] verdict:** none. [VERIFIED: Cargo.toml]
**Packages flagged as suspicious [SUS]:** none. [VERIFIED: Cargo.toml]

## Architecture Patterns

### System Architecture Diagram

```text
Admin browser
  -> /admin/api/items create/edit JSON
  -> Rust route authorization + DTO validation
  -> Catalog service
  -> Oracle schema:
       autograph_items
       autograph_signers
       autograph_item_signers
       item taxonomy columns/join tables
       edit events
  -> Save private metadata only
  -> Explicit publish/full rebuild
  -> Publisher reads published items + signer credits + taxonomy
  -> Candidate static release:
       data/collection.json
       data/facets.json
       data/items/*.json
       items/*/index.html
       media derivatives
  -> Privacy/completeness validation
  -> Promote current release
  -> Caddy serves static public catalog
  -> Public browser filters generated JSON with URL query state
```

### Recommended Project Structure

```text
controller/
├── db/
│   ├── schema.sql                 # Phase 7 end-state schema
│   └── updates/
│       └── 07-*.sql               # additive schema + reviewed migration/backfill scripts
├── src/
│   ├── catalog.rs                 # domain structs, validation, edit history diffs
│   ├── catalog_admin.rs           # admin filters and item summaries
│   ├── oracle_catalog.rs          # production persistence joins/backfill queries
│   ├── oracle_schema.rs           # schema validation
│   ├── contracts.rs               # public schema v2 DTOs/facet ids
│   ├── publisher.rs               # static JSON/detail/facet generation + validation
│   └── routes/admin_items.rs      # admin item/list/history DTOs
├── static-admin/
│   ├── index.html                 # Identity/Classification/Details form sections
│   ├── admin.js                   # signer rows, suggestions, duplicate warnings, merge repair
│   └── admin.css                  # UI-SPEC-constrained styling
└── static-public/
    └── assets/browse.js           # richer URL-backed facets over static JSON
```

### Pattern 1: Normalized signer profile plus item credit

**What:** Add one reusable `autograph_signers` table and one `autograph_item_signers` join/credit table, with a uniqueness rule that prevents the same signer from being credited twice for the same item/role/order combination. [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/constraint.html]

**When to use:** Always for signer data in Phase 7; legacy `autograph_items.signer` remains temporary migration reference only. [VERIFIED: 07-CONTEXT.md]

**Example:**

```sql
-- Source: Oracle SQL Reference constraint docs + current schema style.
create table autograph_signers (
  id varchar2(36) primary key,
  display_name varchar2(255) not null,
  normalized_name varchar2(255) not null,
  role varchar2(80),
  wikipedia_url varchar2(1000),
  imdb_url varchar2(1000),
  created_at timestamp default current_timestamp not null,
  updated_at timestamp default current_timestamp not null,
  constraint autograph_signers_name_uq unique (normalized_name)
);

create table autograph_item_signers (
  item_id varchar2(36) not null,
  signer_id varchar2(36) not null,
  sort_order number(10) default 0 not null,
  item_role varchar2(80),
  item_context varchar2(255),
  created_at timestamp default current_timestamp not null,
  constraint autograph_item_signers_pk primary key (item_id, signer_id, sort_order),
  constraint autograph_item_signers_item_fk
    foreign key (item_id) references autograph_items(id) on delete cascade,
  constraint autograph_item_signers_signer_fk
    foreign key (signer_id) references autograph_signers(id)
);
```

### Pattern 2: Schema-versioned public DTOs

**What:** Bump `PUBLIC_SCHEMA_VERSION` from `1` to `2`, add new DTO fields explicitly, and preserve camelCase JSON naming through Serde. [VERIFIED: codebase grep] [CITED: https://serde.rs/container-attrs.html]

**When to use:** Required when `collection.json`, `facets.json`, item detail JSON, and public detail HTML semantics change. [VERIFIED: codebase grep]

**Example:**

```rust
// Source: Serde attributes docs + current controller/src/contracts.rs pattern.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicGalleryItem {
    pub slug: String,
    pub title: String,
    pub signer_text: String,
    pub signer_names: Vec<String>,
    pub franchises: Vec<String>,
    pub product_line: Option<String>,
    pub format: String,
    pub origin: String,
    pub language: String,
    pub tags: Vec<String>,
    pub primary_image: Option<PublicImage>,
}
```

### Pattern 3: Public facet groups from precomputed values

**What:** Generate facet options from published item DTO fields in Rust, using single-value fields as one option per item and multi-value fields as flattened values. [VERIFIED: codebase grep]

**When to use:** For signer, franchise, product line, format, language, origin, role, and tag groups. [VERIFIED: 07-CONTEXT.md]

**Example:**

```rust
// Source: Existing public_facets pattern in controller/src/publisher.rs.
fn public_facets(items: &[PublicSourceItem]) -> PublicFacets {
    PublicFacets::new(vec![
        public_facet_group(FacetId::Signer, "Signer", items.iter().flat_map(|item| item.signer_names())),
        public_facet_group(FacetId::Franchise, "Franchise", items.iter().flat_map(|item| item.franchises())),
        public_facet_group(FacetId::ProductLine, "Product Line", items.iter().filter_map(|item| item.product_line())),
        public_facet_group(FacetId::Format, "Format", items.iter().map(|item| item.format())),
        public_facet_group(FacetId::Language, "Language", items.iter().map(|item| item.language())),
    ])
}
```

### Pattern 4: URL-backed static filters

**What:** Continue using `URLSearchParams`, `history.pushState`, DOM node creation, and `textContent` for public filter state and labels. [VERIFIED: codebase grep] [CITED: https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams] [CITED: https://developer.mozilla.org/en-US/docs/Web/API/History/pushState] [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent]

**When to use:** For all public facet controls and selected-filter chips. [VERIFIED: 07-UI-SPEC.md]

### Anti-Patterns to Avoid

- **Encoding multiple signers in one string:** Blocks reuse, profile links, and merge repair. [VERIFIED: 07-CONTEXT.md]
- **Keeping `category` as the public format/product-line substitute:** Phase 7 explicitly replaces public Category semantics with controlled `format` and richer taxonomy. [VERIFIED: 07-CONTEXT.md]
- **Publishing direct profile URLs on collection cards:** Locked decision says collection cards show compact signer text only. [VERIFIED: 07-CONTEXT.md]
- **Using loose tags as primary facets:** Primary filters must be signer, franchise, product line, format, and language. [VERIFIED: 07-CONTEXT.md]
- **Adding React/Vite/Next.js/shadcn:** UI-SPEC and AGENTS forbid a frontend framework/build pipeline for Phase 7. [VERIFIED: 07-UI-SPEC.md]
- **Auto-merging duplicate physical items:** Likely duplicate item records are report-only and manual. [VERIFIED: 07-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Referential integrity | Ad hoc Rust-only relationship checks | Oracle primary/foreign key constraints | Database constraints enforce parent/child relationships consistently. [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/constraint.html] |
| Public JSON casing | Manual key renaming | Serde `rename_all = "camelCase"` | Existing contracts already use Serde; docs support container casing attributes. [VERIFIED: codebase grep] [CITED: https://serde.rs/container-attrs.html] |
| Optional public fields | Manual JSON string assembly | Serde `Option` plus `skip_serializing_if` where compatible | Reduces missing/null compatibility mistakes. [CITED: https://serde.rs/field-attrs.html] |
| Admin/public dynamic text | `innerHTML` for taxonomy values | DOM creation plus `textContent` | Existing Phase 6 pattern avoids operator-controlled HTML injection. [VERIFIED: STATE.md] [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent] |
| URL query parsing | String splitting and concatenation | `URLSearchParams` | Browser API handles query read/write/serialization. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams] |
| Static privacy scan | One-off spot checks | Existing publisher validation and privacy deny-list tests | Current tests already reject private object keys, filenames, bucket terms, and missing derivatives. [VERIFIED: codebase grep] |

**Key insight:** The hard part is preserving relationships and public privacy through migration and publish, not inventing a new UI stack. [VERIFIED: codebase grep]

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | Oracle live data currently uses legacy `autograph_items.signer`, `autograph_items.category`, and `autograph_item_tags`; `facets.json` live evidence file referenced by CONTEXT.md is not present in the current workspace. [VERIFIED: codebase grep] [VERIFIED: shell ls] | Generate migration report from live Oracle/static export, produce PL/SQL backfill, run operator review before deploy. |
| Live service config | No known external service config stores signer/category/tag values; Caddy/Podman/GitHub deploy config serves generated releases and controller routes. [VERIFIED: codebase grep] | None for taxonomy names; verify no runtime env vars reference legacy `category` before deploy. |
| OS-registered state | No OS-level registrations embed signer/category/tag taxonomy values; Podman/Caddy units are service/runtime names. [VERIFIED: codebase grep] | None. |
| Secrets/env vars | Existing secrets/env vars cover Oracle, OCI, GHCR, deploy/admin tokens, not metadata taxonomy values. [VERIFIED: AGENTS.md] | None for taxonomy; do not put migration contents in secrets. |
| Build artifacts | Generated static release directories may contain legacy schema v1 `collection.json`, `facets.json`, and detail JSON until full rebuild. [VERIFIED: codebase grep] | Require full publish/rebuild after schema/data migration and validate no stale v1 facets remain in current release. |

**Nothing found in category:** OS-registered taxonomy state is none, verified by codebase docs and no project instructions naming such registrations. [VERIFIED: AGENTS.md]

## Common Pitfalls

### Pitfall 1: Half-migrated public contracts

**What goes wrong:** `collection.json`, `facets.json`, detail JSON, HTML, and `browse.js` disagree about field names or schema version. [VERIFIED: codebase grep]
**Why it happens:** Current contracts are schema v1 and centered on `signer`, `category`, and `tag`. [VERIFIED: codebase grep]
**How to avoid:** Bump schema version, update all public DTOs/tests together, and add contract assertions for every new facet group. [VERIFIED: codebase grep]
**Warning signs:** Browser filter panel shows empty options, URL params do nothing, or `PublicCatalog` deserialization tests fail. [ASSUMED]

### Pitfall 2: Losing edit-history usefulness

**What goes wrong:** Complex taxonomy changes serialize as opaque blobs and the admin cannot see what changed. [VERIFIED: REQUIREMENTS.md]
**Why it happens:** Existing history diffs are field-based; arrays/credits/merge actions need deliberate summaries. [VERIFIED: codebase grep]
**How to avoid:** Add diff labels for `signers`, `characters`, `franchises`, `productLine`, `format`, `origin`, `language`, `setName`, and signer merge events. [ASSUMED]
**Warning signs:** History displays raw JSON arrays, missing before/after values, or no event for signer merge. [ASSUMED]

### Pitfall 3: Duplicate signer profile creation

**What goes wrong:** Typos create duplicate signer profiles and future items link to different rows. [VERIFIED: 07-CONTEXT.md]
**Why it happens:** Inline create/select is intentionally fast and allows new names. [VERIFIED: 07-CONTEXT.md]
**How to avoid:** Normalize signer names, warn on near duplicates, allow deliberate creation, and add a minimal merge repair path. [VERIFIED: 07-CONTEXT.md]
**Warning signs:** Same normalized display name appears in multiple profile rows or typeahead shows near-identical options. [ASSUMED]

### Pitfall 4: Treating `custom` as a format

**What goes wrong:** Custom card items end up with inconsistent format/facet values. [VERIFIED: 07-CONTEXT.md]
**Why it happens:** Current live drift includes `custom` as a tag/category-like signal. [VERIFIED: 07-CONTEXT.md]
**How to avoid:** Backfill `custom` to `Origin: Custom`; keep `Format: Trading Card`, product line, and `Set: Custom` as separate fields. [VERIFIED: 07-CONTEXT.md]
**Warning signs:** Public format facet includes `custom`, or custom items disappear from `Trading Card`. [ASSUMED]

### Pitfall 5: Privacy scan misses new taxonomy fields

**What goes wrong:** New public fields accidentally expose private filenames, object keys, or migration internals. [VERIFIED: codebase grep]
**Why it happens:** Publisher privacy tests currently target known source item/image fields; new strings expand the rendered surface. [VERIFIED: codebase grep]
**How to avoid:** Include new field values in candidate privacy scanning and add regression fixtures with private-looking taxonomy text. [VERIFIED: codebase grep]
**Warning signs:** New DTO fields bypass `validate_private_source_absence` or generated report filenames appear in public JSON. [ASSUMED]

## Code Examples

### Admin API response should remain redacted and camelCase

```rust
// Source: current controller/src/routes/admin_items.rs + Serde docs.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminItemSummaryResponse {
    id: Uuid,
    title: String,
    signer_text: String,
    signer_names: Vec<String>,
    format: String,
    franchises: Vec<String>,
    product_line: Option<String>,
    language: String,
    publication_status: PublicationStatus,
    image_count: usize,
    has_pending_changes: bool,
    updated_at_epoch_seconds: i64,
}
```

### Public browser filter should match multi-value fields by containment

```javascript
// Source: current controller/static-public/assets/browse.js + MDN URLSearchParams.
const matches = (item) =>
  (!state.signer || item.signerNames.includes(state.signer)) &&
  (!state.franchise || item.franchises.includes(state.franchise)) &&
  (!state.productLine || item.productLine === state.productLine) &&
  (!state.format || item.format === state.format) &&
  (!state.language || item.language === state.language) &&
  (!state.origin || item.origin === state.origin) &&
  (!state.role || item.signerRoles.includes(state.role)) &&
  (!state.tag || item.tags.includes(state.tag));
```

### Migration report classifications

```text
Mapped: legacy value has a deterministic target, such as Tr -> Format: Trading Card.
Needs review: legacy value is ambiguous, such as a tag that could be franchise or product line.
Report only: likely duplicate physical item records that should not be auto-merged.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single `signer` string on item | Reusable signer profile plus item-specific signer credit | Phase 7 locked decision, 2026-07-05 | Enables multi-signer physical items and reusable profile links. [VERIFIED: 07-CONTEXT.md] |
| Public `Category` facet | Controlled `Format` plus richer primary/secondary facets | Phase 7 locked decision, 2026-07-05 | Prevents product line/origin/franchise drift inside category/tags. [VERIFIED: 07-CONTEXT.md] |
| Loose tags as main filters | Loose tags are advanced extras only | Phase 7 locked decision, 2026-07-05 | Keeps collector facets natural and reduces tag overload. [VERIFIED: 07-CONTEXT.md] |
| AI ingest before taxonomy | Taxonomy before AI ingest | Roadmap insertion before Phase 8 | AI suggestions will target stable manual fields later. [VERIFIED: ROADMAP.md] |

**Deprecated/outdated:**
- Codebase maps that call Phase 7 “AI-assisted ingest” are stale relative to `ROADMAP.md` and `07-CONTEXT.md`. [VERIFIED: codebase docs]
- New public UI should not use `Category` copy except migration/backfill notes. [VERIFIED: 07-UI-SPEC.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Normalized signer tables are better than JSON blobs for merge/repair and referential integrity. | Standard Stack / Alternatives | If wrong, schema could be more complex than needed; however locked decisions require reusable profiles and credits. |
| A2 | No new crates are needed unless implementation discovers a hard gap. | Standard Stack / Installation | Planner may miss a legitimate utility crate; require checkpoint if adding one. |
| A3 | Diff labels for new taxonomy fields should be human-readable and not raw JSON. | Common Pitfalls | History may be less useful if planner omits rendering work. |
| A4 | Public filter warning signs such as empty facet options imply schema/JS mismatch. | Common Pitfalls | Debug path may differ, but test coverage should catch contract mismatch. |

## Open Questions

1. **Where should the temporary mapping artifact live?**
   - What we know: The mapping file should be committed temporarily and archived or summarized after live migration. [VERIFIED: 07-CONTEXT.md]
   - What's unclear: Exact path and lifetime convention are discretionary. [VERIFIED: 07-CONTEXT.md]
   - Recommendation: Put it under `.planning/phases/07-metadata-taxonomy-and-public-facets/` or `controller/db/updates/` depending on whether it is operator evidence or executable SQL. [ASSUMED]

2. **How strict should signer near-duplicate detection be?**
   - What we know: It must warn without blocking deliberate creation. [VERIFIED: 07-CONTEXT.md]
   - What's unclear: Exact algorithm is delegated to the agent. [VERIFIED: 07-CONTEXT.md]
   - Recommendation: Start with normalized case/spacing/punctuation comparison plus substring/token similarity; avoid adding a crate unless tests show unacceptable misses. [ASSUMED]

3. **Should signer profile merge record one event or per-item events?**
   - What we know: DATA-03 requires useful edit history and D-07-06 requires merge repair. [VERIFIED: REQUIREMENTS.md] [VERIFIED: 07-CONTEXT.md]
   - What's unclear: Existing history is item-scoped, while signer merge is profile-scoped. [VERIFIED: codebase grep]
   - Recommendation: Record a signer merge event plus item metadata-updated events for affected items if item credits change. [ASSUMED]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust/Cargo | Controller, tests | yes | `cargo 1.96.0`, `rustc 1.96.0` | None needed. [VERIFIED: shell] |
| Node.js | GSD tooling/static script sanity checks | yes | `v22.22.2` | None needed. [VERIFIED: shell] |
| Terraform | Infra validation if touched | yes | `1.15.7` | Avoid infra changes for Phase 7 unless deploy config changes. [VERIFIED: shell] |
| Ansible | Deploy/runtime validation if touched | yes with temp override | `ansible-core 2.19.0`; default probe failed under read-only home temp, `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local` works. [VERIFIED: shell] | Set Ansible local temp to `/tmp` in sandboxed validation. |
| SQL Developer / SQL*Plus | Manual live PL/SQL application | no local `sqlplus` found | — | Operator runs SQL Developer manually per D-07-17. [VERIFIED: shell] [VERIFIED: 07-CONTEXT.md] |
| Live Oracle/Object Storage credentials | Live migration/smoke | not available in repo | — | Use local/mock tests; operator-run live smoke/runbook for production. [VERIFIED: AGENTS.md] |

**Missing dependencies with no fallback:**
- Local SQL*Plus is not available, but this does not block planning because D-07-17 explicitly allows operator SQL Developer manual application. [VERIFIED: shell] [VERIFIED: 07-CONTEXT.md]

**Missing dependencies with fallback:**
- Ansible default temp path fails in this sandbox; use `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local` for Ansible validation. [VERIFIED: shell]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | yes | Preserve existing single session-cookie admin auth; do not add public accounts or roles. [VERIFIED: AGENTS.md] |
| V3 Session Management | yes | Preserve Phase 6 cookie/session/logout/lockout behavior. [VERIFIED: 06-CONTEXT.md] |
| V4 Access Control | yes | Keep `/admin/api/*` admin-only and public output generated/static only. [VERIFIED: codebase grep] |
| V5 Input Validation | yes | Validate taxonomy fields and external URLs in Rust before Oracle persistence; render browser text with `textContent`. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent] |
| V6 Cryptography | no new crypto | Do not change existing auth/media signing primitives for taxonomy. [VERIFIED: Cargo.toml] |
| V8 Data Protection | yes | Public artifacts must exclude private Object Storage URLs, object keys, bucket names, namespaces, image UUIDs, credentials, Oracle internals, unpublished records, and migration internals. [VERIFIED: AGENTS.md] |
| V12 File and Resources | yes indirectly | Keep generated derivative and static candidate validation; no direct original URLs. [VERIFIED: codebase grep] |

### Known Threat Patterns for Rust/static taxonomy changes

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Operator-controlled taxonomy HTML injection | Tampering / XSS | Use server-side HTML escaping in publisher and `textContent`/DOM creation in JS. [VERIFIED: codebase grep] [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Node/textContent] |
| Private metadata leakage through new public fields | Information Disclosure | Extend publisher privacy scan and static contract tests to all new fields. [VERIFIED: codebase grep] |
| Unauthorized admin taxonomy edits | Elevation of Privilege | Preserve `authorize_admin_session` checks on new/changed routes. [VERIFIED: codebase grep] |
| Unsafe signer profile merge | Tampering | Require explicit confirmation naming source and target; log merge/edit history. [VERIFIED: 07-UI-SPEC.md] |
| Broken public release after migration | Denial of Service | Run full static rebuild, candidate validation, and rollback-capable release promotion. [VERIFIED: codebase grep] |

## Sources

### Primary (HIGH confidence)
- `AGENTS.md` - project constraints, stack, security/privacy rules, branch guardrails. [VERIFIED: AGENTS.md]
- `.planning/phases/07-metadata-taxonomy-and-public-facets/07-CONTEXT.md` - locked Phase 7 decisions. [VERIFIED: 07-CONTEXT.md]
- `.planning/phases/07-metadata-taxonomy-and-public-facets/07-UI-SPEC.md` - public/admin UI contract. [VERIFIED: 07-UI-SPEC.md]
- `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md` - phase goal, requirements, roadmap history. [VERIFIED: planning docs]
- `controller/src/catalog.rs`, `contracts.rs`, `publisher.rs`, `catalog_admin.rs`, `oracle_catalog.rs`, `routes/admin_items.rs`, `controller/db/schema.sql`, and tests - current implementation surfaces. [VERIFIED: codebase grep]

### Secondary (MEDIUM confidence)
- Oracle Database 26 SQL Reference: constraints, `CREATE TABLE`, `CREATE INDEX`, `ALTER TABLE`. [CITED: https://docs.oracle.com/en/database/oracle/oracle-database/26/sqlrf/constraint.html]
- Serde docs: container and field attributes. [CITED: https://serde.rs/container-attrs.html] [CITED: https://serde.rs/field-attrs.html]
- Axum 0.8.9 docs: `Json<T>` extractor/response behavior. [CITED: https://docs.rs/axum/0.8.9/axum/struct.Json.html]
- MDN Web Docs: `URLSearchParams`, `History.pushState`, `createElement`, `textContent`. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams]

### Tertiary (LOW confidence)
- No tertiary sources used for recommendations; `[ASSUMED]` entries are local engineering judgment. [ASSUMED]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - existing stack and versions verified from `Cargo.toml`/`cargo metadata`; no new packages recommended. [VERIFIED: cargo metadata]
- Architecture: HIGH - codebase and planning docs agree on Rust controller + generated static public output, with stale Phase 7 wording noted in codebase maps. [VERIFIED: codebase docs]
- Migration strategy: MEDIUM - locked rollout is clear, but exact live data values require operator/live Oracle evidence not present in this workspace. [VERIFIED: 07-CONTEXT.md]
- UI patterns: HIGH - `07-UI-SPEC.md` is explicit and existing static admin/public code matches the plain JS approach. [VERIFIED: 07-UI-SPEC.md]
- Pitfalls: MEDIUM - most pitfalls are inferred from current v1 schema and migration risk, then grounded in code/tests. [VERIFIED: codebase grep]

**Research date:** 2026-07-09
**Valid until:** 2026-08-08 for codebase-local planning; re-check crate/docs versions before package changes or long-delayed implementation. [ASSUMED]
