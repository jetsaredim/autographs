# Phase 3: Public Gallery MVP - Context

**Gathered:** 2026-05-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 delivers the anonymous public gallery experience for Jared Greenwald's autograph collection. It includes a branded landing page, a public collection browsing page, curated public filtering/discovery, public item detail pages, multi-image viewing, and public empty/not-found/media-missing states.

This phase does not deliver admin workflows, public accounts, social/community features, direct Object Storage URLs, app-wide incident handling, advanced search infrastructure, or a collection-page "Surprise Me" feature.

</domain>

<decisions>
## Implementation Decisions

### Site Brand and Landing Page
- **D-03-01:** The public site name is **Jared Greenwald's Autograph Gallery**. Use this as the primary public-facing title/brand and replace the generic proof-of-life "Autographs" presentation on public UX surfaces.
- **D-03-02:** The main landing page is a basic overview of the collection, not the full gallery grid.
- **D-03-03:** The landing page should guide anonymous visitors toward two primary actions: **View Collection** and **Surprise Me**.
- **D-03-04:** "Surprise Me" appears only on the main landing page, selects from the full published collection, and is not available on the filtered collection page.

### Collection Browsing
- **D-03-05:** The collection page should be easy to navigate and image-forward, using a grid of smaller thumbnails rather than a dense list or table.
- **D-03-06:** Clicking a collection thumbnail opens that item's public detail page.
- **D-03-07:** The collection grid should emphasize visual browsing while still making enough metadata visible for orientation.

### Filtering and Discovery
- **D-03-08:** Filtering uses a dropdown/filterable tag-cloud style menu based on a curated set of public-facing metadata.
- **D-03-09:** MVP facets should be intentionally selected metadata such as card game, IP/category, and meaningful tags.
- **D-03-10:** Selecting filters should reduce or enlarge the visible collection grid.
- **D-03-11:** The MVP should not expose every raw database tag/category by default.

### Detail Page Presentation
- **D-03-12:** Detail pages should be clean and focused, with the primary image as the main visual focus.
- **D-03-13:** Important metadata should use grouped sections instead of one giant flat list.
- **D-03-14:** Essential facts should appear near the top. Supporting metadata can be grouped into sections such as provenance, certification, tags, and collection notes.
- **D-03-15:** Important metadata examples include signer, item/card title, card game/IP/category, rarity when applicable, certification, year, event, and source.
- **D-03-16:** Exact grouping, labels, and layout may be iterated during implementation once real sample data is visible.
- **D-03-42:** The image/image gallery should be the main focus and centered on initial detail-page view.
- **D-03-43:** Clicking the focused image should reduce it slightly and reveal the grouped information to the right of the image where viewport space allows. This interaction can be iterated during implementation.

### Image Viewing
- **D-03-17:** If an item has multiple images, show one focused primary image and show all available images as smaller thumbnails below it.
- **D-03-18:** Clicking a thumbnail swaps it into the focused image area.
- **D-03-19:** Thumbnail selection does not change the URL, hash, or query string in the MVP.
- **D-03-20:** No dedicated public lightbox route or image-browsing route is required in Phase 3.

### Public Empty, Not Found, and Missing States
- **D-03-21:** Public no-result, not-found, and media-missing states should use warm editorial content rather than dry system messages.
- **D-03-22:** These states apply to no published items, filters returning no matches, missing/unpublished detail pages, image/media fetch failures, and public catalog data fallback states.
- **D-03-23:** Empty states should use short movie-reference quotes about not finding things, such as "These aren't the droids you're looking for" and "X never, ever marks the spot."
- **D-03-24:** Movie quotes should be stylized as quote blocks with proper attribution rendered separately from the quote.
- **D-03-25:** Quote blocks should be paired with practical recovery actions such as Clear filters, Back to collection, View collection, or Surprise Me where appropriate.
- **D-03-26:** Keep quote usage short and tasteful; do not build the entire UX around long quoted passages.
- **D-03-44:** The examples in this context are directional examples only. Development should include researching similar short "not finding things" movie quotes for user review and approval.
- **D-03-45:** Approved quotes should be stored in the database or another durable approved list, then randomly selected for public empty/not-found/error states.
- **D-03-46:** State-specific messages should draw from the same approved quote source/list rather than hard-coding separate per-state quotes.

### Public Footer and Future Admin Access
- **D-03-47:** Public pages should include a very small footer with an `About` link.
- **D-03-48:** The `About` link should display the current architecture page for now; that architecture page needs more detail in a later phase or follow-up.
- **D-03-49:** The public UI should leave room for a future admin entry point, but it should be non-obvious so casual visitors do not click around on it.
- **D-03-50:** The future admin access affordance can be nearly hidden or unlocked by a keyboard combination. Phase 3 should not build the admin UI, but should avoid blocking that later access pattern.

### Image Access Friction
- **D-03-27:** Phase 3 should make casual image extraction structurally difficult while acknowledging that browser-viewable images cannot be made impossible to extract.
- **D-03-28:** Public pages must not expose direct Object Storage URLs, bucket paths, object keys, storage credentials, or browser-visible storage identifiers.
- **D-03-29:** Images must continue to flow through app-mediated routes only.
- **D-03-30:** Public image displays should discourage casual saving where practical, including preventing default context-menu behavior on image displays and avoiding visible standalone image links.
- **D-03-31:** The goal is anti-casual extraction and no direct storage exposure, not DRM. Determined users can still screenshot, inspect network traffic, or replay app-mediated image requests.

### Temporary Production Data Entry
- **D-03-32:** Phase 3 should not invest heavily in a dedicated production seeding workflow or polished import tool before the Phase 4 admin workflow exists.
- **D-03-33:** Document the procedural path for temporary production data entry: open an SSH tunnel from the operator machine to the runtime VM and forward a local port to `127.0.0.1:3000` on the VM.
- **D-03-34:** Local curl/script commands may call `http://127.0.0.1:<forwarded-port>/api/operator/...` through the tunnel and must include `Authorization: Bearer <AUTOGRAPHS_OPERATOR_API_TOKEN>`.
- **D-03-35:** The deployed app should write production Oracle metadata and private Object Storage images through the existing catalog service/operator API path. Do not manually hand-edit database rows or upload untracked objects.
- **D-03-36:** Do not expose operator endpoints through public Caddy routes for Phase 3. This is a transitional bridge until Phase 4 admin workflow replaces it.
- **D-03-37:** For Phase 3, documentation/procedure is enough unless implementation reveals a hard blocker.

### Carried Forward From Earlier Phases
- **D-03-38:** Public gallery surfaces must show only published items.
- **D-03-39:** Image delivery must remain app-mediated through `/api/catalog/{itemId}/images/{imageId}`.
- **D-03-40:** Do not expose direct Object Storage URLs, object keys, bucket credentials, or browser-visible storage credentials.
- **D-03-41:** Existing public APIs are read-only; existing temporary operator mutation routes are not part of the public gallery UX.

### the agent's Discretion
- Exact responsive grid breakpoints and card density, provided the collection remains image-forward and easy to navigate.
- Exact curated facet taxonomy for the first implementation, provided it starts with public-facing metadata such as card game, IP/category, and meaningful tags.
- Exact metadata section names and ordering, provided the image/gallery remains the first focus and supporting metadata is grouped cleanly.
- Exact quote rotation implementation, provided quotes come from an approved durable list, are short, attributed, stylized, and paired with recovery actions.
- Exact hidden future-admin affordance, provided it is not a visible public navigation target and does not implement Phase 4 admin behavior.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Product Scope and Phase Requirements
- `.planning/ROADMAP.md` — Defines Phase 3 goal, success criteria, dependency on Phase 2, and phase boundaries.
- `.planning/REQUIREMENTS.md` — Defines `GALL-01` through `GALL-04` plus out-of-scope constraints.
- `.planning/STATE.md` — Records current project state, prior decisions, and Phase 3 readiness.
- `.planning/PROJECT.md` — Defines core value, product constraints, and personal-collection scope.

### Prior Phase Decisions
- `.planning/phases/02-oracle-and-private-media-core/02-CONTEXT.md` — Locks private media, app-mediated image delivery, published-only public reads, and temporary operator route constraints.
- `.planning/phases/02-oracle-and-private-media-core/02-04-SUMMARY.md` — Records app-mediated private image delivery and the operator data/media smoke path.

### Existing App Surfaces
- `app/app/api/catalog/route.ts` — Public catalog list route with category/signer/tag query filters.
- `app/app/api/catalog/[id]/route.ts` — Public item detail route.
- `app/app/api/catalog/[id]/images/[imageId]/route.ts` — App-mediated published image route.
- `app/app/architecture/page.tsx` — Current architecture page that the public footer `About` link should expose.
- `app/src/catalog/types.ts` — Catalog item, image, and list option types.
- `app/src/catalog/service.ts` — Catalog service behavior for item reads, image reads, and image attachment.
- `app/db/seed/sample-autographs.ts` — Current representative sample metadata and image shape.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/app/api/catalog/route.ts`: Existing public list endpoint accepts `category`, `signer`, and `tag`; Phase 3 can build initial filters against this shape and extend only if the curated-facet UX needs it.
- `app/app/api/catalog/[id]/route.ts`: Existing public detail endpoint returns one published item by id and already hides missing/unpublished records behind 404 behavior.
- `app/app/api/catalog/[id]/images/[imageId]/route.ts`: Existing image endpoint streams private media through the app and sets cache/security headers. Public UI should use this route for all images.
- `app/app/architecture/page.tsx`: Existing architecture page can serve as the initial footer `About` destination.
- `app/src/catalog/types.ts`: The item model already includes publication status, category, tags, signer, title, description, certification, event/source fields, and ordered images.
- `app/db/seed/sample-autographs.ts`: Current sample records include published and draft items plus primary/supporting images; Phase 3 can use this to shape local gallery fixture expectations.

### Established Patterns
- The app uses Next.js App Router under `app/app`.
- Current public UI is simple global CSS in `app/app/globals.css`; Phase 3 may replace proof-of-life styling with a more complete public gallery style system.
- Public catalog APIs are dynamic server routes and are read-only.
- Media privacy is enforced by app-mediated read routes, not direct Object Storage access.

### Integration Points
- Main landing page: `app/app/page.tsx`.
- New collection page: likely a new public route such as `app/app/collection/page.tsx`.
- New detail page: likely a new public route keyed by item id, connected to `GET /api/catalog/{id}`.
- Image components must build URLs with `/api/catalog/{itemId}/images/{imageId}` and avoid exposing storage metadata.
- Footer/about integration should link to the current architecture page without turning it into a primary public navigation destination.
- Temporary production data-entry docs should reference the existing operator API shape without making those endpoints public through Caddy.

</code_context>

<specifics>
## Specific Ideas

- Landing page actions: **View Collection** and **Surprise Me**.
- Public site title: **Jared Greenwald's Autograph Gallery**.
- Collection page: grid of smaller thumbnails with a dropdown/filterable tag-cloud style menu.
- Filtering examples: card game, IP/category, and meaningful tags.
- Detail page: focused primary image, grouped metadata, and thumbnail selector below the primary image.
- Detail interaction: initial focus is the centered image/gallery; clicking the focused image reduces it slightly and reveals metadata to the right where layout allows.
- Empty/not-found states: short attributed movie quotes about not finding things, rendered as quote blocks with practical recovery actions. Candidate quotes should be researched, reviewed, approved, and stored in a durable approved list before random display.
- Footer: very small `About` link to the current architecture page.
- Future admin access: leave room for a non-obvious/near-hidden affordance or keyboard-unlocked entry point without building admin UI in Phase 3.
- Temporary production data entry command shape:
  - `ssh -L <local-port>:127.0.0.1:3000 opc@<runtime-public-ip>`
  - `curl -H "Authorization: Bearer <AUTOGRAPHS_OPERATOR_API_TOKEN>" http://127.0.0.1:<local-port>/api/operator/...`

</specifics>

<deferred>
## Deferred Ideas

- Re-evaluate a hybrid filter model once there is enough real collection data: curated primary facets first, with an "all tags" or deeper-browse area for richer discovery.
- Replace the temporary SSH-tunnel operator data-entry procedure with the Phase 4 admin workflow.
- Dedicated lightbox/image route can be reconsidered after MVP if image browsing needs a richer standalone experience.
- Add more detailed architecture page content after the footer `About` link exposes the current architecture page.
- Implement the actual admin UI and finalized hidden/admin-entry behavior in Phase 4.

</deferred>

---

*Phase: 3-Public Gallery MVP*
*Context gathered: 2026-05-21*
