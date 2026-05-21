# Phase 3: Public Gallery MVP - Research

**Researched:** 2026-05-21
**Domain:** Next.js App Router public gallery over Oracle-backed catalog and app-mediated private images
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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
- Exact metadata section names and ordering, provided essential facts appear near the top and supporting metadata is grouped cleanly.
- Exact quote rotation implementation, provided quotes are short, attributed, stylized, and paired with recovery actions.

### Deferred Ideas (OUT OF SCOPE)
- Re-evaluate a hybrid filter model once there is enough real collection data: curated primary facets first, with an "all tags" or deeper-browse area for richer discovery.
- Replace the temporary SSH-tunnel operator data-entry procedure with the Phase 4 admin workflow.
- Dedicated lightbox/image route can be reconsidered after MVP if image browsing needs a richer standalone experience.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GALL-01 | Anonymous user can browse a public gallery of published autograph items. | Use `createCatalogService().list()` from Server Components and render `/collection` as an image-forward grid; repository defaults to `publication_status = 'published'` unless `includeUnpublished` is set. [VERIFIED: app/src/catalog/repository.ts] |
| GALL-02 | Anonymous user can filter or search the gallery by signer name, category, and tags. | Existing public list API and repository support `signer`, `category`, and `tag`; implement URL query-backed filters and derive curated facets from published items. [VERIFIED: app/app/api/catalog/route.ts] [VERIFIED: app/src/catalog/repository.ts] |
| GALL-03 | Anonymous user can open a detail page for a single autograph item and view its full stored metadata. | Existing detail route and service `getById` return the v1 metadata shape and hide missing/unpublished records as `null`/404. [VERIFIED: app/app/api/catalog/[id]/route.ts] [VERIFIED: app/src/catalog/types.ts] |
| GALL-04 | Anonymous user can view all published images attached to an autograph item, including a clear primary presentation. | Existing item shape includes ordered images and primary marker; image bytes are read through `/api/catalog/{itemId}/images/{imageId}` only. [VERIFIED: app/src/catalog/types.ts] [VERIFIED: app/app/api/catalog/[id]/images/[imageId]/route.ts] |
</phase_requirements>

## Summary

Phase 3 should be planned as a public App Router UI layer over the already-proven catalog service, not as a new data/media subsystem. Server-rendered pages should call `src/catalog` directly for initial data so Oracle credentials and object metadata remain server-side, while the existing read-only JSON routes stay available for client-side refreshes, smoke checks, or future API consumers. [CITED: https://nextjs.org/docs/app/getting-started/fetching-data] [VERIFIED: app/src/catalog/index.ts]

Use small Client Components only where browser interactivity is required: the curated filter menu/tag cloud, the landing-page Surprise Me button, context-menu suppression around image displays, and the detail-page thumbnail swapper. Next.js pages/layouts are Server Components by default, and Client Components are the documented fit for state and event handlers. [CITED: https://nextjs.org/docs/app/getting-started/server-and-client-components]

The planner should keep media privacy as a hard architectural invariant. Browser-visible image `src` values may contain app-level catalog IDs, but must not contain Object Storage namespaces, bucket names, object keys, signed URLs, or credentials. The current app-mediated image route already streams private media and sets `Content-Type`, `Cache-Control`, and `X-Content-Type-Options`. [VERIFIED: app/app/api/catalog/[id]/images/[imageId]/route.ts]

**Primary recommendation:** Build `/` as the branded overview, `/collection` as a server-rendered filtered grid, and `/collection/[id]` as a server-rendered detail page with a small client image viewer; keep all image URLs shaped as `/api/catalog/{itemId}/images/{imageId}`. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Landing page overview | Frontend Server (SSR) | Browser / Client | Static brand/editorial content belongs in the App Router page; Surprise Me needs client-side navigation/random selection unless implemented as a server redirect endpoint. [CITED: https://nextjs.org/docs/app/getting-started/server-and-client-components] |
| Published collection browsing | Frontend Server (SSR) | API / Backend | Initial grid should fetch published catalog data server-side through `createCatalogService`; existing API route remains a JSON integration surface. [CITED: https://nextjs.org/docs/app/getting-started/fetching-data] [VERIFIED: app/src/catalog/index.ts] |
| Filter/search controls | Browser / Client | Frontend Server (SSR) | The controls need event handlers and URL query updates; the server page should read `searchParams` to render the filtered result. [CITED: https://nextjs.org/docs/app/api-reference/functions/use-search-params] |
| Curated facets | Frontend Server (SSR) | API / Backend | Facets can be derived from published items for MVP; repository/API filtering already supports signer/category/tag. [VERIFIED: app/src/catalog/repository.ts] |
| Item detail | Frontend Server (SSR) | API / Backend | Dynamic segment `/collection/[id]` should fetch by id and call `notFound()` when unpublished/missing. [CITED: https://nextjs.org/docs/app/api-reference/file-conventions/dynamic-routes] [CITED: https://nextjs.org/docs/app/api-reference/functions/not-found] |
| Thumbnail swapper | Browser / Client | Frontend Server (SSR) | Thumbnail selection is local state and must not mutate URL in MVP. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md] |
| Image byte delivery | API / Backend | Database / Storage | Object Storage access stays server-side through `readPublishedImage`; browser receives app route responses only. [VERIFIED: app/src/catalog/service.ts] |
| Temporary production data entry docs | Operations docs | API / Backend | Operator mutation is transitional and token-guarded; Phase 3 should document SSH tunnel usage rather than expose operator endpoints publicly. [VERIFIED: docs/deployment-runbook.md] [VERIFIED: app/app/api/operator/catalog/route.ts] |

## Project Constraints (from AGENTS.md)

- Use a single `Next.js` full-stack application for v1. [VERIFIED: AGENTS.md]
- Prefer OCI Always Free services where feasible. [VERIFIED: AGENTS.md]
- Prefer Oracle Autonomous Database Free unless implementation friction forces a justified fallback. [VERIFIED: AGENTS.md]
- Keep autograph images private in OCI Object Storage and centralize access through the app. [VERIFIED: AGENTS.md]
- Auto-deploy from GitHub Actions on merge to `main`. [VERIFIED: AGENTS.md]
- Keep operations understandable for one developer; avoid multi-service sprawl. [VERIFIED: AGENTS.md]
- v1 scope excludes staging, bulk import, public accounts, and advanced search infrastructure; multi-image items and edit history remain in scope. [VERIFIED: AGENTS.md]
- Use least-privilege OCI access and explicit secret handling. [VERIFIED: AGENTS.md]
- Start file-changing work through a GSD command; this research was initialized with `gsd-sdk query init.phase-op "03-public-gallery-mvp"`. [VERIFIED: gsd-sdk init]
- If GitHub/network publishing commands fail due to connectivity or credentials, stop and report rather than inventing alternate remotes/protocols. [VERIFIED: AGENTS.md]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js | Installed `16.2.4`; latest npm `16.2.6`, modified 2026-05-21 | App Router pages, route handlers, build/runtime | Existing app already uses Next.js App Router; route handlers and Server Components are the native patterns for this phase. [VERIFIED: app/package.json] [VERIFIED: npm registry] |
| React | Installed `19.2.5`; latest npm `19.2.6`, modified 2026-05-08 | Component rendering and client interactivity | Existing Next.js app depends on React 19; use `useState` only in small Client Components for filters/image selection. [VERIFIED: app/package.json] [VERIFIED: npm registry] |
| TypeScript | Installed/latest `6.0.3`, modified 2026-04-16 | Type safety for catalog types and UI props | Existing app uses `tsc --noEmit`; reuse `AutographItem`/`AutographImage` types rather than duplicating payload shapes. [VERIFIED: app/package.json] [VERIFIED: npm registry] |
| Oracle catalog service | Local module | Published metadata reads | `createCatalogService` composes Oracle repository and private media store; it is the existing server-side domain boundary. [VERIFIED: app/src/catalog/index.ts] |
| App-mediated image route | Local route | Public image delivery without storage exposure | Existing route reads only published item images and returns bytes with security/cache headers. [VERIFIED: app/app/api/catalog/[id]/images/[imageId]/route.ts] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Next.js `Image` | Bundled with Next.js | Responsive image rendering | Use with internal app-mediated URLs only if it does not create unwanted optimizer proxy behavior; `unoptimized` is appropriate for SVG/local fixture images and authenticated/private image sources. [CITED: https://nextjs.org/docs/app/api-reference/components/image] |
| Playwright | Not installed; latest npm `1.60.0`, modified 2026-05-21 | Browser-level public route verification | Add only if planner wants automated UI behavior checks for filter interaction and thumbnail swapper; otherwise keep Phase 3 validation to lint/typecheck/build plus manual browser smoke. [VERIFIED: npm registry] [CITED: https://nextjs.org/docs/app/guides/testing/playwright] |
| ESLint | Installed `9.39.4`; latest npm `10.4.0`, modified 2026-05-15 | Static linting | Keep existing lint command; do not upgrade during Phase 3 unless the planner deliberately scopes dependency churn. [VERIFIED: app/package.json] [VERIFIED: npm registry] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Server Components calling catalog service | Client fetching `/api/catalog` for all initial UI | Client fetching adds loading complexity and exposes more API dependency to the browser; Server Components keep DB access and item shaping close to the source. [CITED: https://nextjs.org/docs/app/getting-started/fetching-data] |
| URL query-backed filters | Pure in-memory client filters only | Query-backed filters are shareable and refresh-safe; pure in-memory filters lose state on reload and make public route integration harder. [CITED: https://nextjs.org/docs/app/api-reference/functions/use-search-params] |
| Native CSS and small Client Components | UI framework dependency | Existing app uses global CSS only; adding a component framework would create dependency and design-system scope not required for MVP. [VERIFIED: app/app/globals.css] |
| Existing app image route | Direct OCI signed URLs or public bucket URLs | Direct storage URLs conflict with locked decisions and public direct-object-storage exclusion. [VERIFIED: .planning/REQUIREMENTS.md] |

**Installation:**

```bash
# No runtime dependency is required for the recommended MVP.
# Optional browser tests, if planned:
corepack pnpm --filter app add -D @playwright/test
```

## Architecture Patterns

### System Architecture Diagram

```text
Anonymous visitor
  |
  v
Next.js public pages
  |-- / ----------------------> Branded overview + View Collection + Surprise Me
  |                              |
  |                              v
  |                           Client navigation to /collection/{randomPublishedId}
  |
  |-- /collection?signer=&category=&tag=
  |        |
  |        v
  |   Server Component reads searchParams
  |        |
  |        v
  |   createCatalogService().list({ signer, category, tag })
  |        |
  |        v
  |   Oracle repository applies published-only default + filters
  |        |
  |        v
  |   Image-forward grid renders app image src values only
  |
  |-- /collection/[id]
           |
           v
      Server Component awaits params and calls getById(id)
           |                         |
           | item found               | missing/unpublished
           v                         v
      grouped metadata +       notFound() -> warm quote state
      client thumbnail viewer
           |
           v
      <img/Image src="/api/catalog/{itemId}/images/{imageId}">
           |
           v
      Route Handler readPublishedImage()
           |
           v
      Oracle metadata authorizes image -> private media store reads OCI/local object
```

### Recommended Project Structure

```text
app/
├── app/
│   ├── page.tsx                         # branded landing page
│   ├── not-found.tsx                    # warm public not-found fallback
│   ├── collection/
│   │   ├── page.tsx                     # filtered published gallery
│   │   ├── loading.tsx                  # optional meaningful gallery skeleton
│   │   └── [id]/
│   │       ├── page.tsx                 # item detail
│   │       └── not-found.tsx            # item-specific missing/unpublished state
│   ├── components/
│   │   ├── PublicHeader.tsx             # shared public nav/brand
│   │   ├── GalleryGrid.tsx              # server-rendered cards
│   │   ├── GalleryFilters.tsx           # small Client Component
│   │   ├── ImageViewer.tsx              # small Client Component thumbnail swapper
│   │   └── EmptyState.tsx               # quote + recovery actions
│   └── api/catalog/...                  # existing read-only JSON/media routes
├── src/
│   └── catalog/
│       ├── public-view-models.ts        # optional UI-safe shaping helpers
│       └── ...
└── docs/
    └── temporary-production-data-entry.md
```

### Pattern 1: Server-Rendered Catalog Pages

**What:** Fetch catalog items directly from the server-side service in `page.tsx`, passing only render-safe fields into components. [CITED: https://nextjs.org/docs/app/getting-started/fetching-data]

**When to use:** Use for `/collection` and `/collection/[id]` initial render because they depend on Oracle-backed data and publication filtering. [VERIFIED: app/src/catalog/repository.ts]

**Example:**

```typescript
// Source: Next.js Server Component data-fetching docs + local catalog service.
import { createCatalogService } from "../../src/catalog";

type CollectionPageProps = {
  searchParams: Promise<{
    signer?: string;
    category?: string;
    tag?: string;
  }>;
};

export default async function CollectionPage({ searchParams }: CollectionPageProps) {
  const filters = await searchParams;
  const service = createCatalogService();
  const items = await service.list({
    signer: filters.signer,
    category: filters.category,
    tag: filters.tag,
  });

  return <GalleryGrid items={items} />;
}
```

### Pattern 2: URL Query-Backed Filter Controls

**What:** Render the collection from server-side `searchParams`, then use a Client Component to update `signer`, `category`, and `tag` query values. [CITED: https://nextjs.org/docs/app/api-reference/functions/use-search-params]

**When to use:** Use for the dropdown/filterable tag-cloud menu so selected filters survive refreshes and direct links. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

**Example:**

```typescript
// Source: Next.js useSearchParams docs.
"use client";

import { useRouter, useSearchParams } from "next/navigation";

export function GalleryFilters({ tags }: { tags: string[] }) {
  const router = useRouter();
  const searchParams = useSearchParams();

  const setTag = (tag: string | null) => {
    const next = new URLSearchParams(searchParams);
    if (tag) next.set("tag", tag);
    else next.delete("tag");
    router.push(`/collection?${next.toString()}`);
  };

  return tags.map((tag) => (
    <button key={tag} type="button" onClick={() => setTag(tag)}>
      {tag}
    </button>
  ));
}
```

### Pattern 3: Detail Route With `notFound()`

**What:** Use a dynamic segment and `notFound()` when `createCatalogService().getById(id)` returns `null`. [CITED: https://nextjs.org/docs/app/api-reference/file-conventions/dynamic-routes] [CITED: https://nextjs.org/docs/app/api-reference/functions/not-found]

**When to use:** Use for missing or unpublished detail pages so users get the warm public 404 state and search engines receive noindex behavior. [CITED: https://nextjs.org/docs/app/api-reference/functions/not-found]

**Example:**

```typescript
// Source: Next.js dynamic route and notFound docs.
import { notFound } from "next/navigation";
import { createCatalogService } from "../../../src/catalog";

type DetailPageProps = {
  params: Promise<{ id: string }>;
};

export default async function DetailPage({ params }: DetailPageProps) {
  const { id } = await params;
  const item = await createCatalogService().getById(id);
  if (!item) notFound();

  return <AutographDetail item={item} />;
}
```

### Pattern 4: App-Mediated Image Viewer

**What:** Build image `src` values only from public item/image IDs, then let the existing route stream private bytes. [VERIFIED: app/app/api/catalog/[id]/images/[imageId]/route.ts]

**When to use:** Use for all gallery thumbnails, detail primary images, and supporting thumbnails. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

**Example:**

```typescript
// Source: local app-mediated image route contract.
const publicImageSrc = (itemId: string, imageId: string) =>
  `/api/catalog/${encodeURIComponent(itemId)}/images/${encodeURIComponent(imageId)}`;
```

### Pattern 5: Context-Menu Friction Is Best-Effort Only

**What:** Add `onContextMenu={(event) => event.preventDefault()}` to public image display wrappers as an anti-casual-saving measure. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Element/contextmenu_event]

**When to use:** Use on gallery card image areas and the detail viewer, paired with no direct links to standalone image URLs. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

**Example:**

```typescript
// Source: MDN contextmenu event docs.
<div className="image-frame" onContextMenu={(event) => event.preventDefault()}>
  <img src={publicImageSrc(item.id, image.id)} alt={image.altText ?? item.title} draggable={false} />
</div>
```

### Anti-Patterns to Avoid

- **Fetching the app's own API from Server Components:** Call `createCatalogService()` directly in server pages; keep `/api/catalog` for client/browser and integration use. [CITED: https://nextjs.org/docs/app/getting-started/fetching-data]
- **Exposing raw image object metadata in UI props:** `AutographImage` currently includes `storageNamespace`, `bucketName`, and `objectKey`; shape UI props so those fields are not rendered, serialized into client components unnecessarily, or included in DOM attributes. [VERIFIED: app/src/catalog/types.ts]
- **Direct links to image routes:** An `<a href="/api/catalog/.../images/...">` creates visible standalone image URLs; render images inline and provide navigation back to item/collection instead. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]
- **Every-tag-as-filter MVP:** Curated facets are locked; do not expose every raw tag by default. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]
- **Long movie quote passages:** The Copyright Office notes that fair use has no fixed word-count rule and depends on circumstances; keep quotes short, attributed, and incidental. [CITED: https://www.copyright.gov/help/faq/faq-fairuse.html]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Routing and dynamic detail pages | Custom router/state machine | Next.js App Router dynamic segments | Dynamic route params are a built-in file convention. [CITED: https://nextjs.org/docs/app/api-reference/file-conventions/dynamic-routes] |
| 404 handling | Manual "not found" booleans across pages | `notFound()` plus segment `not-found.tsx` | Next.js handles route termination and SEO noindex for 404 flows. [CITED: https://nextjs.org/docs/app/api-reference/functions/not-found] |
| Data access | New client-side catalog SDK | Existing `createCatalogService` and `CatalogRepository` | The service already enforces published reads and private media lookup. [VERIFIED: app/src/catalog/service.ts] |
| Image delivery | Signed URL generator, public bucket, or proxy rewrite | Existing `/api/catalog/{itemId}/images/{imageId}` route | It preserves app-controlled access and storage secrecy. [VERIFIED: app/app/api/catalog/[id]/images/[imageId]/route.ts] |
| Browser automation, if needed | Custom DOM smoke scripts | Playwright | Playwright is the documented Next.js E2E testing option for Chromium/Firefox/WebKit automation. [CITED: https://nextjs.org/docs/app/guides/testing/playwright] |
| Quote legality rules | Homemade "N words is always safe" rule | Short, attributed quotes plus explicit legal uncertainty | Official guidance says there is no fixed permitted word count. [CITED: https://www.copyright.gov/help/faq/faq-fairuse.html] |

**Key insight:** The deceptive complexity in this phase is not the grid; it is accidentally widening the public contract. Keep route/page work small, keep catalog IDs as the only public image identifiers, and keep object/storage metadata on the server side. [VERIFIED: app/src/catalog/types.ts]

## Common Pitfalls

### Pitfall 1: Leaking Storage Metadata Into Client Components

**What goes wrong:** Passing full `AutographItem` objects into `"use client"` components serializes image fields that include namespace, bucket, and object key. [VERIFIED: app/src/catalog/types.ts]

**Why it happens:** The existing domain model is server-safe but not UI-public-safe. [VERIFIED: app/src/catalog/types.ts]

**How to avoid:** Add view-model helpers that expose only `id`, display metadata, tags, and `imageId`/`altText`/primary flag to client components. [ASSUMED]

**Warning signs:** Rendered HTML, React payloads, or browser devtools show `objectKey`, `bucketName`, `storageNamespace`, or OCI-style paths. [VERIFIED: app/src/catalog/types.ts]

### Pitfall 2: Losing Published-Only Semantics

**What goes wrong:** UI code accidentally calls `includeUnpublished: true` or consumes operator routes, showing draft records publicly. [VERIFIED: app/src/catalog/repository.ts]

**Why it happens:** The service supports operator/admin reads for Phase 2 verification and later Phase 4 admin work. [VERIFIED: app/src/catalog/service.ts]

**How to avoid:** Public pages and public API routes must call default `list()`/`getById()` with no `includeUnpublished`. [VERIFIED: app/src/catalog/repository.ts]

**Warning signs:** The sample draft "Jordan Ellis" item appears in `/collection` or can be opened through public detail. [VERIFIED: app/db/seed/sample-autographs.ts]

### Pitfall 3: Client-Side Filtering Diverges From API Filtering

**What goes wrong:** The UI filters in-memory differently from `/api/catalog`, causing direct URLs, refreshes, and API smoke checks to disagree. [VERIFIED: app/app/api/catalog/route.ts]

**Why it happens:** The repository signer filter is case-insensitive substring, category is exact match, and tag is exact match. [VERIFIED: app/src/catalog/repository.ts]

**How to avoid:** Treat URL query parameters as canonical and pass them to the service/API using the same `signer`, `category`, and `tag` keys. [VERIFIED: app/app/api/catalog/route.ts]

**Warning signs:** `/collection?tag=baseball` and `/api/catalog?tag=baseball` show different item sets. [VERIFIED: app/app/api/catalog/route.ts]

### Pitfall 4: `next/image` Optimizer Changes the Media Contract

**What goes wrong:** The optimizer may request app-mediated image URLs through `/_next/image`, which changes what appears in browser network traffic and can complicate private-image debugging. [CITED: https://nextjs.org/docs/app/api-reference/components/image]

**Why it happens:** The Image component generates optimized `srcset`/`src` URLs by default. [CITED: https://nextjs.org/docs/app/api-reference/components/image]

**How to avoid:** Prefer plain `<img>` for maximum transparency, or use `<Image unoptimized>` after validating network output contains no storage metadata. [CITED: https://nextjs.org/docs/app/api-reference/components/image]

**Warning signs:** Gallery image network requests do not hit `/api/catalog/{itemId}/images/{imageId}` directly, or SVG fixtures behave unexpectedly. [CITED: https://nextjs.org/docs/app/api-reference/components/image]

### Pitfall 5: Treating Context-Menu Prevention as Security

**What goes wrong:** The implementation presents right-click suppression as protection rather than friction. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Element/contextmenu_event]

**Why it happens:** `preventDefault()` can cancel many context menu events, but browser behavior has exceptions and it cannot stop screenshots or network replay. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Element/contextmenu_event] [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

**How to avoid:** Document and test the real invariant: no direct OCI URLs/object keys/credentials in DOM, source, or network responses. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

**Warning signs:** Review language says "prevents downloads" or "protects images" instead of "discourages casual saving." [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

### Pitfall 6: Movie Quote Overreach

**What goes wrong:** Empty states become quote-heavy or use long, recognizable passages without commentary. [CITED: https://www.copyright.gov/help/faq/faq-fairuse.html]

**Why it happens:** Short quotes feel harmless, but fair use is contextual and has no bright-line word count. [CITED: https://www.copyright.gov/help/faq/faq-fairuse.html]

**How to avoid:** Use only short, attributed snippets already listed in context; keep the recovery action more prominent than the quote. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

**Warning signs:** Empty state text uses paragraphs of dialogue or quote-only UI without practical action. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]

## Code Examples

### UI-Safe View Model

```typescript
// Source: local catalog types; strips storage metadata before client components.
import type { AutographItem } from "../catalog";

export type PublicGalleryItem = {
  id: string;
  title: string;
  signer: string;
  category: string;
  tags: string[];
  primaryImage?: {
    id: string;
    altText: string;
  };
};

export const toPublicGalleryItem = (item: AutographItem): PublicGalleryItem => {
  const primary = item.images.find((image) => image.isPrimary) ?? item.images[0];

  return {
    id: item.id,
    title: item.title,
    signer: item.signer,
    category: item.category,
    tags: item.tags,
    primaryImage: primary
      ? { id: primary.id, altText: primary.altText ?? `${item.title} signed by ${item.signer}` }
      : undefined,
  };
};
```

### Curated Facet Derivation

```typescript
// Source: local repository/API filter contract.
const preferredTags = new Set(["baseball", "poster", "concert", "card", "spring-training"]);

export const buildFacets = (items: PublicGalleryItem[]) => ({
  categories: [...new Set(items.map((item) => item.category))].sort(),
  signers: [...new Set(items.map((item) => item.signer))].sort(),
  tags: [...new Set(items.flatMap((item) => item.tags))]
    .filter((tag) => preferredTags.has(tag))
    .sort(),
});
```

### Detail Metadata Groups

```typescript
// Source: Phase 3 detail grouping decisions + local catalog type.
const metadataGroups = (item: AutographItem) => [
  {
    heading: "Essential facts",
    rows: [
      ["Signer", item.signer],
      ["Item", item.title],
      ["Category", item.category],
      ["Estimated year", item.estimatedYear?.toString()],
    ],
  },
  {
    heading: "Provenance",
    rows: [
      ["Event", item.eventName],
      ["Location", item.eventLocation],
      ["Source", item.source],
      ["Object reference", item.objectReference],
    ],
  },
  {
    heading: "Certification",
    rows: [
      ["Company", item.certificationCompany],
      ["Certification ID", item.certificationId],
    ],
  },
  {
    heading: "Collection notes",
    rows: [
      ["Description", item.description],
      ["Inscription", item.inscription],
      ["Tags", item.tags.join(", ")],
    ],
  },
].map((group) => ({
  ...group,
  rows: group.rows.filter(([, value]) => value),
}));
```

### Temporary Production Data Entry Documentation Shape

```bash
# Source: Phase 3 context and existing operator token route.
ssh -L 4300:127.0.0.1:3000 opc@<runtime-public-ip>

curl --fail \
  -H "Authorization: Bearer <AUTOGRAPHS_OPERATOR_API_TOKEN>" \
  -H "Content-Type: application/json" \
  -X POST \
  --data @new-autograph.json \
  http://127.0.0.1:4300/api/operator/catalog
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Pages Router `getServerSideProps` for server data | App Router async Server Components | Next.js App Router era; current docs updated 2026-03-31 | Plan pages as async `page.tsx` functions, not Pages Router files. [CITED: https://nextjs.org/docs/app/getting-started/fetching-data] |
| API routes under `pages/api` | Route Handlers under `app/.../route.ts` | App Router convention; current docs updated 2026-03-31 | Existing catalog routes are already in the correct style. [CITED: https://nextjs.org/docs/app/api-reference/file-conventions/route] [VERIFIED: app/app/api/catalog/route.ts] |
| Manual 404 UI branching everywhere | Segment `not-found.tsx` plus `notFound()` | App Router convention | Plan warm empty/not-found states as reusable components and route-level `not-found.tsx`. [CITED: https://nextjs.org/docs/app/api-reference/file-conventions/not-found] |

**Deprecated/outdated:**
- `pages/` Router patterns such as `getStaticProps`/`getServerSideProps` are not the right fit for this codebase because the app already uses App Router under `app/app`. [VERIFIED: app/app/page.tsx] [CITED: https://nextjs.org/docs/app/getting-started/fetching-data]
- Public buckets or signed direct image URLs are out of scope and conflict with private media requirements. [VERIFIED: .planning/REQUIREMENTS.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Add view-model helpers that expose only UI-safe item/image fields to client components. | Common Pitfalls | If React Server Component serialization does not expose sensitive fields in this implementation path, the helper is still a low-cost defense; if omitted and serialization does expose them, Phase 3 violates image privacy constraints. |
| A2 | Plain `<img>` may be preferable to `next/image` for app-mediated private images. | Common Pitfalls | If planner prefers Next Image, they must validate generated network URLs and optimizer behavior against the no-storage-exposure rule. |

## Open Questions

1. **Should Phase 3 add Playwright now or leave browser behavior manual?**
   - What we know: `workflow.nyquist_validation` is explicitly `false`, and no test framework is installed. [VERIFIED: .planning/config.json] [VERIFIED: app/package.json]
   - What's unclear: Whether UI safety gate expectations require automated browser tests for this phase. [ASSUMED]
   - Recommendation: Keep required verification to lint/typecheck/build/API smoke plus manual UI smoke; add Playwright only if the planner wants automated thumbnail/filter behavior coverage. [CITED: https://nextjs.org/docs/app/guides/testing/playwright]

2. **Should the public detail route be `/collection/[id]` or `/autographs/[id]`?**
   - What we know: Phase context says new detail page is likely keyed by item id and collection thumbnails open details. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]
   - What's unclear: Final URL naming is not locked. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]
   - Recommendation: Use `/collection/[id]` for MVP because it keeps the public browse/detail area together. [ASSUMED]

3. **Should Surprise Me use client-side random choice or a server redirect route?**
   - What we know: Surprise Me appears only on the landing page and chooses from the full published collection. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md]
   - What's unclear: Whether the planner values no-JS operation for that action. [ASSUMED]
   - Recommendation: Implement a small client button using server-rendered published IDs for MVP; a server route can be added later if no-JS behavior matters. [ASSUMED]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js/pnpm via Corepack | Build, lint, typecheck, dev server | Yes | Existing `corepack pnpm` commands run | None needed. [VERIFIED: corepack pnpm list] |
| Next.js | Public App Router implementation | Yes | Installed `16.2.4` | None needed. [VERIFIED: app/package.json] |
| Oracle credentials | Live catalog data | Not available in this research context | N/A | Use local configured development database/media mode and existing dry-run/seed scripts where possible; live proof remains operator-run. [VERIFIED: .planning/phases/02-oracle-and-private-media-core/02-04-SUMMARY.md] |
| OCI Object Storage credentials | Live image delivery | Not available in this research context | N/A | Use local media provider and generated sample fixtures for UI iteration. [VERIFIED: app/src/media/local-store.ts] |
| Playwright browsers | Optional automated UI behavior tests | Not installed | N/A | Manual browser smoke plus lint/typecheck/build. [VERIFIED: app/package.json] |

**Missing dependencies with no fallback:**
- None for planning or local UI implementation; live production data/media proof requires operator credentials and is already documented as an operator gate. [VERIFIED: .planning/phases/02-oracle-and-private-media-core/02-04-SUMMARY.md]

**Missing dependencies with fallback:**
- Live Oracle/Object Storage credentials can be replaced during UI iteration by local provider/fixtures, provided production API contracts remain unchanged. [VERIFIED: app/src/media/local-store.ts] [VERIFIED: app/db/seed/sample-autographs.ts]
- Playwright can be skipped unless automated UI behavior tests are explicitly scoped. [VERIFIED: .planning/config.json]

## Verification Strategy

`workflow.nyquist_validation` is explicitly `false`, so this research omits the Nyquist Validation Architecture section. [VERIFIED: .planning/config.json]

| Check | Command / Action | Purpose |
|-------|------------------|---------|
| Typecheck | `corepack pnpm --filter app typecheck` | Catch App Router prop typing, Client Component boundary, and view-model typing errors. [VERIFIED: app/package.json] |
| Lint | `corepack pnpm --filter app lint` | Enforce existing ESLint rules. [VERIFIED: app/package.json] |
| Build | `corepack pnpm --filter app build` | Prove Next.js route/page compilation. [VERIFIED: app/package.json] |
| Seed dry run | `corepack pnpm --filter app db:seed:dry-run` | Ensure sample records still match catalog schema after UI-facing helper changes. [VERIFIED: app/package.json] |
| API smoke | `curl /api/catalog`, `curl /api/catalog?tag=baseball`, `curl /api/catalog/{publishedId}` | Verify public API list/filter/detail still returns published items only. [VERIFIED: app/app/api/catalog/route.ts] |
| Image route smoke | `curl -I /api/catalog/{itemId}/images/{imageId}` | Verify media route returns image content via app and no storage URL redirect. [VERIFIED: app/app/api/catalog/[id]/images/[imageId]/route.ts] |
| Manual UI smoke | Browser: `/`, `/collection`, filtered collection, detail page, thumbnail swap, missing item | Verify success criteria and editorial states. [VERIFIED: .planning/ROADMAP.md] |
| Privacy inspection | Browser devtools: HTML, React payload, network URLs, image requests | Confirm no `bucketName`, `storageNamespace`, `objectKey`, OCI URL, signed URL, or credential leaks. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md] |
| Optional E2E | Playwright test for filter URL, detail navigation, thumbnail swapper | Automate the highest-risk UI behavior if test dependency is added. [CITED: https://nextjs.org/docs/app/guides/testing/playwright] |

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V1 Encoding and Sanitization | Yes | React text rendering plus avoid `dangerouslySetInnerHTML`; quote text and metadata should be rendered as text. [CITED: https://owasp.org/www-project-application-security-verification-standard/] |
| V2 Authentication | No for public gallery; yes for operator docs | Public gallery is anonymous; operator endpoints remain bearer-token guarded and tunnel-only by procedure. [VERIFIED: app/app/api/operator/catalog/route.ts] |
| V3 Session Management | No | Phase 3 has no public accounts or sessions. [VERIFIED: .planning/REQUIREMENTS.md] |
| V4 Access Control | Yes | Public reads must use default published-only service/repository behavior; operator mutation routes are not public UX. [VERIFIED: app/src/catalog/repository.ts] |
| V5 Validation, Sanitization, Encoding | Yes | Treat `signer`, `category`, and `tag` query params as untrusted; repository uses binds for SQL filters. [VERIFIED: app/src/catalog/repository.ts] |
| V6 Cryptography / Secrets | Yes | Do not expose Object Storage credentials, API signing material, operator token, or direct URLs. [VERIFIED: docs/configuration-contract.md] |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Draft item disclosure | Information Disclosure | Never set `includeUnpublished` in public pages/routes; verify sample draft is absent. [VERIFIED: app/src/catalog/repository.ts] |
| Storage object-key disclosure | Information Disclosure | Strip storage fields before client components and inspect rendered/network output. [VERIFIED: app/src/catalog/types.ts] |
| SQL injection through filters | Tampering | Keep Oracle bind variables in repository filters; do not string-concatenate query params. [VERIFIED: app/src/catalog/repository.ts] |
| Operator endpoint exposure | Elevation of Privilege | Keep `/api/operator/...` token-guarded and reachable only through SSH tunnel procedure, not public Caddy routes. [VERIFIED: app/app/api/operator/catalog/route.ts] [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md] |
| XSS through metadata/quotes | Tampering | Render metadata and movie quotes as React text, not HTML. [ASSUMED] |
| Misstated image protection | Repudiation / Information Disclosure | Document anti-casual friction honestly; do not claim DRM. [VERIFIED: .planning/phases/03-public-gallery-mvp/03-CONTEXT.md] |

## Sources

### Primary (HIGH confidence)
- `AGENTS.md` - project constraints, GSD workflow, security and publishing rules. [VERIFIED: file read]
- `.planning/phases/03-public-gallery-mvp/03-CONTEXT.md` - locked Phase 3 user decisions and deferred ideas. [VERIFIED: file read]
- `.planning/REQUIREMENTS.md` - GALL-01 through GALL-04 and v1 exclusions. [VERIFIED: file read]
- `.planning/ROADMAP.md` - Phase 3 goal and success criteria. [VERIFIED: file read]
- `.planning/STATE.md` - Phase 3 readiness and Phase 2 completion state. [VERIFIED: file read]
- `.planning/phases/02-oracle-and-private-media-core/02-CONTEXT.md` - prior data/media decisions. [VERIFIED: file read]
- `.planning/phases/02-oracle-and-private-media-core/02-04-SUMMARY.md` - app-mediated delivery and live operator gate. [VERIFIED: file read]
- `app/app/api/catalog/route.ts` - public list route and query shape. [VERIFIED: file read]
- `app/app/api/catalog/[id]/route.ts` - public detail route behavior. [VERIFIED: file read]
- `app/app/api/catalog/[id]/images/[imageId]/route.ts` - image streaming route and headers. [VERIFIED: file read]
- `app/src/catalog/types.ts`, `app/src/catalog/service.ts`, `app/src/catalog/repository.ts`, `app/src/catalog/index.ts` - catalog model/service/repository behavior. [VERIFIED: file read]
- `app/db/seed/sample-autographs.ts` - representative published/draft sample data. [VERIFIED: file read]
- Next.js official docs - Server Components, data fetching, dynamic routes, route handlers, image component, `notFound`, `useSearchParams`, Playwright guide. [CITED: https://nextjs.org/docs]
- MDN contextmenu event docs - context-menu event and `preventDefault()` limitations. [CITED: https://developer.mozilla.org/en-US/docs/Web/API/Element/contextmenu_event]
- U.S. Copyright Office fair use FAQ - quote/fair-use uncertainty. [CITED: https://www.copyright.gov/help/faq/faq-fairuse.html]
- OWASP ASVS project page - security verification framing. [CITED: https://owasp.org/www-project-application-security-verification-standard/]
- npm registry - current package versions for Next.js, React, TypeScript, ESLint, Playwright, OCI SDK, oracledb. [VERIFIED: npm registry]

### Secondary (MEDIUM confidence)
- None used.

### Tertiary (LOW confidence)
- None used.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - package versions were verified through installed manifests and npm registry; no required new runtime dependency is recommended.
- Architecture: HIGH - App Router patterns were verified against current Next.js docs and mapped to existing service/route files.
- Pitfalls: HIGH for storage/publication/API concerns from code and context; MEDIUM for `next/image` optimizer risk until implementation validates exact network behavior.
- Legal quote guidance: MEDIUM - official Copyright Office guidance confirms uncertainty and no fixed word count, but this is not legal advice.

**Research date:** 2026-05-21
**Valid until:** 2026-06-20 for Next.js/package-version details; phase constraints remain valid until CONTEXT.md or REQUIREMENTS.md changes.
