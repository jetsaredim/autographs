# Feature Landscape

**Domain:** Autograph catalog and private-photo-backed public gallery
**Researched:** 2026-04-18

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Public gallery grid/list | Browsers need an obvious way to explore the collection without signing in | Low | This is the front door of the product and should work anonymously from day one |
| Item detail page with full metadata | A collectible catalog is only useful if each piece has a canonical detail view | Low | Must include image, title, signer, category, tags, event/source, and authentication details when known |
| Metadata search and filtering | The prompt explicitly requires discovery by signer, category, and tags | Medium | Keep v1 to structured metadata filters plus simple text search; do not build advanced search infrastructure yet |
| Private image delivery through the app | The core trust model depends on images staying private in object storage while remaining publicly viewable through the site | Medium | This is table stakes for this product because privacy of storage is a stated requirement, not an enhancement |
| Single-admin authenticated upload path | The catalog cannot grow without a reliable way to add new items | Medium | One admin account only; avoid multi-user user-management scope entirely |
| Image upload plus metadata form | The admin needs one place to attach the photo and the descriptive record together | Medium | Treat image object reference and metadata persistence as a single workflow to avoid orphaned records/files |
| Searchable metadata model | Collectors expect more than a title and caption | Medium | Persist signer, category, tags, event/location, inscription, certification company/ID, estimated year, and description |
| Basic validation and review before publish | Catalog quality matters more than upload speed in v1 | Medium | Require explicit admin confirmation before a new item becomes visible in the public gallery |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| AI-assisted metadata suggestions during upload | Speeds up cataloging while keeping the admin in control | Medium | Suggestions should prefill likely signer, item type, inscription hints, and other metadata, but never auto-publish |
| Hybrid OCR plus AI extraction | Makes autograph-specific uploads more useful than a generic image uploader | High | OCR can surface text on inscriptions, certificates, or labels before AI organizes it into suggested fields |
| Structured collectible-specific metadata | Makes the gallery feel like a real catalog rather than a generic photo feed | Medium | Certification details, event/source, inscription text, and estimated year are meaningful domain differentiators |
| App-mediated private media model | Balances public browsing with private storage hygiene | Medium | This is also an architectural choice, but it creates user trust and operator control that many basic galleries skip |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Public user accounts | Directly out of scope for the intentionally anonymous v1 | Keep all public access anonymous and read-only |
| Likes, comments, follows, or social activity | Turns a catalog into a community product and explodes moderation scope | Focus on browse, search, and detail views only |
| Multiple admin accounts or roles | Adds auth and permissions complexity before the single-operator workflow is proven | Use exactly one admin login path |
| Bulk import pipelines | Increases edge cases, file handling complexity, and validation burden too early | Optimize the single-item upload and review loop first |
| Multiple images per autograph item | Adds gallery, storage, and editorial complexity without validating the core catalog model | Keep one primary image per item in v1 |
| Advanced search stack or vector search | The prompt only needs metadata-based search/filtering and the dataset will be small initially | Use relational filtering and simple text search over stored fields |
| Edit history and versioning | Useful later, but not necessary to validate the catalog experience | Allow straightforward edits later without a revision system in v1 |
| Public direct object storage URLs | Conflicts with the private-storage requirement and weakens control over access patterns | Route image access through the application |

## Feature Dependencies

```text
Private object storage setup -> Admin image upload
Private object storage setup -> App-mediated image delivery
Metadata schema -> Gallery listing
Metadata schema -> Item detail page
Metadata schema -> Search and filtering
Single-admin authentication -> Admin upload/review workflow
Image upload -> AI/OCR suggestion pipeline
AI/OCR suggestion pipeline -> Review-and-confirm publish step
Review-and-confirm publish step -> Public gallery visibility
Gallery listing -> Item detail navigation
```

## MVP Recommendation

Prioritize:
1. Public gallery with anonymous browsing
2. Item detail page with collectible-specific metadata
3. Single-admin upload flow with image attach, AI-assisted suggestions, and mandatory human review

Defer: Multi-image items: the first release should prove that one clean image plus rich metadata is enough to make the catalog useful.

Defer: Social/community features: they do not support the narrow validation goal of a browsable autograph catalog.

Defer: Bulk import and advanced search: both are scale optimizations that should wait until the single-item workflow and base discovery model are working.

## MVP vs Later Scope

### MVP

- Anonymous public gallery browsing
- Search/filter by signer, category, and tags
- Detail page for a single autograph item
- One admin authentication path
- Single-image upload per item
- AI-assisted metadata suggestions with OCR help where useful
- Admin review/edit/confirm before save
- Private object storage with app-mediated image access
- Rich but manageable metadata schema oriented to autograph collecting

### Later

- Edit/delete ergonomics beyond the initial admin form
- Multi-image support per item
- Bulk import tooling
- Richer search facets, saved filters, or relevance tuning
- Public submissions, user accounts, or social features
- Moderation workflows, audit history, and versioning
- Collection analytics or trend views

## Sources

- `.planning/PROJECT.md` - project scope, active requirements, and explicit out-of-scope boundaries
- `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` - canonical implementation brief and v1 success criteria
- `.planning/codebase/CONCERNS.md` - current repository risks and reminders to keep the first slice thin and end-to-end
