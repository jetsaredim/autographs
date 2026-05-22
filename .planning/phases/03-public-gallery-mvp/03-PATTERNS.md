# Phase 03: Public Gallery MVP - Pattern Map

**Mapped:** 2026-05-21
**Files analyzed:** 18 likely new/modified files
**Analogs found:** 14 / 18

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `app/app/page.tsx` | route/page | request-response | `app/app/page.tsx` | exact-modify |
| `app/app/layout.tsx` | provider/layout | request-response | `app/app/layout.tsx` | exact-modify |
| `app/app/globals.css` | config/style | transform | `app/app/globals.css` | exact-modify |
| `app/app/collection/page.tsx` | route/page | request-response | `app/app/api/catalog/route.ts` + `app/app/page.tsx` | role-match |
| `app/app/collection/[id]/page.tsx` | route/page | request-response | `app/app/api/catalog/[id]/route.ts` | role-match |
| `app/app/collection/[id]/not-found.tsx` | route/page | request-response | `app/app/api/catalog/[id]/route.ts` | partial |
| `app/app/not-found.tsx` | route/page | request-response | `app/app/architecture/page.tsx` | partial |
| `app/app/admin/page.tsx` | route/page | request-response | `app/app/architecture/page.tsx` | role-match |
| `app/app/components/PublicHeader.tsx` | component | request-response | `app/app/architecture/page.tsx` | partial |
| `app/app/components/PublicFooter.tsx` | component | request-response | `app/app/page.tsx` | partial |
| `app/app/components/GalleryGrid.tsx` | component | request-response | `app/app/architecture/page.tsx` | partial |
| `app/app/components/GalleryFilters.tsx` | component | request-response | no local analog | none |
| `app/app/components/ImageViewer.tsx` | component | request-response | no local analog | none |
| `app/app/components/EmptyState.tsx` | component | request-response | no local analog | none |
| `app/app/components/AdminUnlock.tsx` | component | event-driven | no local analog | none |
| `app/src/catalog/public-view-models.ts` | utility | transform | `app/src/catalog/repository.ts` + `app/src/catalog/types.ts` | role-match |
| `app/src/gallery/approved-quotes.ts` | fixture/utility | transform | `app/db/seed/sample-autographs.ts` | role-match |
| `docs/temporary-production-data-entry.md` or `docs/deployment-runbook.md` | docs/runbook | file-I/O | `docs/deployment-runbook.md` | exact |

## Pattern Assignments

### `app/app/page.tsx` (route/page, request-response)

**Analog:** `app/app/page.tsx`

**Imports pattern** (lines 1-1):
```typescript
import Link from "next/link";
```

**Server component pattern** (lines 5-41):
```typescript
export default function HomePage() {
  return (
    <main className="proof-shell">
      <section className="proof-card" aria-labelledby="proof-title">
        <p className="eyebrow">Phase 1 proof-of-life</p>
        <h1 id="proof-title">Autographs is up and ready for the delivery spine.</h1>
        ...
      </section>
    </main>
  );
}
```

**Apply:** Replace proof-of-life content with the branded overview. Keep App Router page as a server component unless the implementation chooses a tiny child Client Component for `Surprise Me`.

---

### `app/app/layout.tsx` (provider/layout, request-response)

**Analog:** `app/app/layout.tsx`

**Imports and metadata pattern** (lines 1-10):
```typescript
import type { Metadata } from "next";
import type { ReactNode } from "react";

import "./globals.css";

export const metadata: Metadata = {
  title: "Autographs | Proof of Life",
  description:
    "Phase 1 proof-of-life surface for the Autographs application scaffold.",
};
```

**Layout shell pattern** (lines 12-22):
```typescript
type RootLayoutProps = Readonly<{
  children: ReactNode;
}>;

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
```

**Apply:** Update metadata to `Jared Greenwald's Autograph Gallery`. Keep `globals.css` imported only once from the root layout.

---

### `app/app/globals.css` (config/style, transform)

**Analog:** `app/app/globals.css`

**Global reset/font pattern** (lines 1-25):
```css
:root {
  color-scheme: light;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
  background:
    radial-gradient(circle at top, #ffffff 0, #f4f4f5 42%, #e5e7eb 100%);
  color: #111111;
}

* {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  min-height: 100%;
}

a {
  color: inherit;
}
```

**Responsive media-query pattern** (lines 271-308):
```css
@media (min-width: 720px) {
  .proof-shell {
    padding: 5rem 2rem;
  }

  .status-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}
```

**Apply:** Replace proof-of-life visual styling with Phase 3 native CSS tokens from `03-UI-SPEC.md`: `#f8f7f4`, `#ffffff`, `#1f6f68`, `#1f2937`, `#4b5563`, `#d7d2c8`, `#f1efea`. Avoid the existing gradient background and large rounded cards for public gallery surfaces.

---

### `app/app/collection/page.tsx` (route/page, request-response)

**Analogs:** `app/app/api/catalog/route.ts`, `app/src/catalog/index.ts`

**Catalog service import pattern** (`app/app/api/catalog/route.ts` lines 1-3):
```typescript
import { createCatalogService } from "../../../src/catalog";

export const dynamic = "force-dynamic";
```

**Public filter contract** (`app/app/api/catalog/route.ts` lines 5-15):
```typescript
export async function GET(request: Request) {
  const url = new URL(request.url);
  const service = createCatalogService();
  const items = await service.list({
    category: url.searchParams.get("category") ?? undefined,
    signer: url.searchParams.get("signer") ?? undefined,
    tag: url.searchParams.get("tag") ?? undefined,
  });

  return Response.json({ items });
}
```

**Service factory boundary** (`app/src/catalog/index.ts` lines 1-10):
```typescript
import { createOracleExecutor } from "../db/oracle";
import { createPrivateMediaStore } from "../media";
import { OracleCatalogRepository } from "./repository";
import { DefaultCatalogService } from "./service";

export const createCatalogRepository = (): OracleCatalogRepository =>
  new OracleCatalogRepository(createOracleExecutor());

export const createCatalogService = (): DefaultCatalogService =>
  new DefaultCatalogService(createCatalogRepository(), createPrivateMediaStore());
```

**Apply:** Server-render `/collection` by calling `createCatalogService().list({ signer, category, tag })` directly. Do not call the app's own `/api/catalog` route from a Server Component.

---

### `app/app/collection/[id]/page.tsx` (route/page, request-response)

**Analog:** `app/app/api/catalog/[id]/route.ts`

**Dynamic params pattern** (lines 5-12):
```typescript
type RouteContext = {
  params: Promise<{ id: string }> | { id: string };
};

export async function GET(_request: Request, context: RouteContext) {
  const { id } = await context.params;
  const service = createCatalogService();
  const item = await service.getById(id);
```

**Missing item handling pattern** (lines 14-18):
```typescript
if (!item) {
  return Response.json({ error: "Not found" }, { status: 404 });
}

return Response.json({ item });
```

**Apply:** For the page route, use `params: Promise<{ id: string }>` and `notFound()` instead of returning JSON. Keep default public `getById(id)` semantics so drafts and missing records are hidden.

---

### `app/app/collection/[id]/not-found.tsx` and `app/app/not-found.tsx` (route/page, request-response)

**Analog:** `app/app/architecture/page.tsx`

**Page metadata pattern** (lines 5-9):
```typescript
export const metadata: Metadata = {
  title: "Autographs | Architecture",
  description:
    "End-to-end architecture for the Autographs deployment path from GitHub through OCI and Caddy.",
};
```

**Semantic section pattern** (lines 74-91):
```typescript
export default function ArchitecturePage() {
  return (
    <main className="architecture-shell">
      <section className="architecture-hero" aria-labelledby="architecture-title">
        <Link className="back-link" href="/">
          ← Proof of life
        </Link>
        <p className="eyebrow">System architecture</p>
        <h1 id="architecture-title">Autographs system overview</h1>
        <p className="lede">...</p>
      </section>
    </main>
  );
}
```

**Apply:** Render warm quote states as normal page content with recovery links. Do not implement quote/error states as toasts.

---

### `app/app/admin/page.tsx` (route/page, request-response)

**Analog:** `app/app/architecture/page.tsx`

**Simple static page pattern** (lines 74-91):
```typescript
export default function ArchitecturePage() {
  return (
    <main className="architecture-shell">
      <section className="architecture-hero" aria-labelledby="architecture-title">
        ...
      </section>
    </main>
  );
}
```

**Apply:** Use a static App Router page for the placeholder admin route. It must communicate that collection management is coming later and must not include login, mutation, upload, edit, publish, or operator workflow behavior.

---

### `app/app/components/PublicHeader.tsx`, `PublicFooter.tsx`, `GalleryGrid.tsx` (components, request-response)

**Analog:** `app/app/architecture/page.tsx`

**Local data array pattern** (lines 11-72):
```typescript
const workflowSteps = [
  {
    number: 1,
    name: "Bootstrap tenancy with Terraform",
    description:
      "The admin user runs the manual Terraform bootstrap that creates the compartment, deploy identity, policies, and state bucket.",
  },
  ...
];
```

**Mapped render pattern** (lines 124-134):
```typescript
<tbody>
  {workflowSteps.map((step) => (
    <tr key={step.number}>
      <td>
        <span className="step-badge">{step.number}</span>
      </td>
      <td>{step.name}</td>
      <td>{step.description}</td>
    </tr>
  ))}
</tbody>
```

**Link pattern** (`app/app/page.tsx` lines 17-20):
```typescript
<p className="lede lede-action">
  Review the <Link href="/architecture">end-to-end architecture diagram</Link>{" "}
  for the current delivery path from GitHub Actions to Caddy on OCI.
</p>
```

**Apply:** Keep repeated display components simple and typed. Use `Link` for collection/detail/about navigation. `GalleryGrid` cards should link to `/collection/{itemId}` and render only UI-safe view-model fields.

---

### `app/src/catalog/public-view-models.ts` (utility, transform)

**Analogs:** `app/src/catalog/types.ts`, `app/src/catalog/repository.ts`

**Server-safe domain model showing sensitive image fields** (`app/src/catalog/types.ts` lines 3-21):
```typescript
export type AutographImageInput = {
  storageNamespace: string;
  bucketName: string;
  objectKey: string;
  contentType: string;
  byteSize?: number | null;
  checksum?: string | null;
  etag?: string | null;
  isPrimary: boolean;
  sortOrder: number;
  altText?: string | null;
};

export type AutographImage = AutographImageInput & {
  id: string;
  itemId: string;
  createdAt: Date;
  updatedAt: Date;
};
```

**Public metadata shape** (`app/src/catalog/types.ts` lines 23-46):
```typescript
export type AutographItem = Omit<AutographItemInput, "images"> & {
  id: string;
  createdAt: Date;
  updatedAt: Date;
  images: AutographImage[];
};
```

**Transform helper pattern** (`app/src/catalog/repository.ts` lines 64-87):
```typescript
const toItem = (
  row: ItemRow,
  tags: string[],
  images: AutographImage[],
): AutographItem => ({
  id: row.ID,
  title: row.TITLE,
  signer: row.SIGNER,
  ...
  images,
});
```

**Apply:** Add UI-safe types that strip `storageNamespace`, `bucketName`, `objectKey`, `checksum`, and `etag` before passing data to Client Components. Keep app-mediated image URLs derived only from `item.id` and `image.id`.

---

### `app/src/gallery/approved-quotes.ts` (fixture/utility, transform)

**Analog:** `app/db/seed/sample-autographs.ts`

**Typed durable list pattern** (lines 1-4):
```typescript
import type { AutographItemInput } from "../../src/catalog/types";

export const sampleAutographs: AutographItemInput[] = [
```

**Representative fixture object pattern** (lines 4-20):
```typescript
{
  title: "Signed baseball from spring training",
  signer: "Maya Thompson",
  description:
    "Baseball signed in blue ink after a spring training exhibition game.",
  category: "Sports",
  tags: ["baseball", "spring-training", "ball"],
  ...
  publicationStatus: "published",
  images: [
```

**Apply:** Store quote inventory as a typed durable approved list unless the planner chooses a DB-backed quote table. Include quote text and attribution as separate fields, keep entries short, and make random state selection draw from this single source.

---

### `docs/temporary-production-data-entry.md` or `docs/deployment-runbook.md` (docs/runbook, file-I/O)

**Analog:** `docs/deployment-runbook.md`

**Runbook structure pattern** (lines 1-15):
```markdown
# Deployment Runbook

This runbook gets the app from a clean checkout to an OCI VM running systemd-managed Podman quadlets...

## Preconditions

- OCI tenancy exists.
- An OCI user or deploy identity has API signing keys for Phase 1.
```

**Command block pattern** (lines 155-176):
````markdown
## Manual Smoke Path

After deployment, verify from your workstation:

```bash
curl --fail --silent "http://${VM_PUBLIC_IP}/health"
```
````

**Existing data/media note to reference** (lines 118-126):
```markdown
## Data and Media Smoke

...
Published images are served through app-mediated URLs shaped as `/api/catalog/{itemId}/images/{imageId}`.
```

**Apply:** Add a concise temporary production data-entry procedure using SSH local port forwarding and `Authorization: Bearer <AUTOGRAPHS_OPERATOR_API_TOKEN>`. Keep it framed as transitional until Phase 4 admin, and do not suggest public Caddy exposure for operator routes.

## Shared Patterns

### Public Catalog Reads

**Source:** `app/src/catalog/repository.ts`
**Apply to:** `/collection`, `/collection/[id]`, `GalleryGrid`, `GalleryFilters`

```typescript
if (!options.includeUnpublished) {
  clauses.push("publication_status = 'published'");
}
if (options.signer) {
  clauses.push("lower(signer) like :signer");
  binds.signer = `%${options.signer.toLowerCase()}%`;
}
if (options.category) {
  clauses.push("category = :category");
  binds.category = options.category;
}
if (options.tag) {
  clauses.push(
    "exists (select 1 from autograph_item_tags t where t.item_id = autograph_items.id and t.tag = :tag)",
  );
  binds.tag = options.tag;
}
```

Lines: `app/src/catalog/repository.ts` 214-230.

### App-Mediated Image Delivery

**Source:** `app/app/api/catalog/[id]/images/[imageId]/route.ts`
**Apply to:** gallery cards, detail viewer, media-missing handling

```typescript
const image = await service.readPublishedImage(id, imageId);

if (!image) {
  return Response.json({ error: "Not found" }, { status: 404 });
}

return new Response(body, {
  headers: {
    "Content-Type": image.contentType,
    "Cache-Control": "public, max-age=300, stale-while-revalidate=3600",
    "X-Content-Type-Options": "nosniff",
  },
});
```

Lines: `app/app/api/catalog/[id]/images/[imageId]/route.ts` 14-30.

### Storage Metadata Must Stay Server-Side

**Source:** `app/src/catalog/types.ts`
**Apply to:** all Client Components and public DOM/rendered props

```typescript
export type AutographImageInput = {
  storageNamespace: string;
  bucketName: string;
  objectKey: string;
  contentType: string;
  byteSize?: number | null;
  checksum?: string | null;
  etag?: string | null;
  isPrimary: boolean;
  sortOrder: number;
  altText?: string | null;
};
```

Lines: `app/src/catalog/types.ts` 3-14.

### Operator Token Guard

**Source:** `app/app/api/operator/catalog/route.ts`
**Apply to:** docs only; do not expose in public UX

```typescript
const authorizeOperator = (request: Request): Response | null => {
  const token = process.env.AUTOGRAPHS_OPERATOR_API_TOKEN;
  if (!token) {
    return Response.json({ error: "Operator API is disabled" }, { status: 404 });
  }

  const providedToken = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "");
  if (providedToken !== token) {
    return Response.json({ error: "Unauthorized" }, { status: 401 });
  }

  return null;
};
```

Lines: `app/app/api/operator/catalog/route.ts` 38-50.

### Script Error Handling

**Source:** `app/scripts/smoke-data.ts`
**Apply to:** any new smoke/quote validation script, if added

```typescript
main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
```

Lines: `app/scripts/smoke-data.ts` 72-75.

## No Analog Found

Files with no close match in the codebase. Planner should use `03-RESEARCH.md` and `03-UI-SPEC.md` patterns for these.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `app/app/components/GalleryFilters.tsx` | component | request-response | No existing Client Component, `useRouter`, or `useSearchParams` usage exists. |
| `app/app/components/ImageViewer.tsx` | component | request-response | No existing local state/thumbnail selector component exists. |
| `app/app/components/EmptyState.tsx` | component | request-response | No existing warm public empty/not-found quote component exists. |
| `app/app/components/AdminUnlock.tsx` | component | event-driven | No keyboard/hidden affordance component exists. |

## Metadata

**Analog search scope:** `app/app`, `app/src/catalog`, `app/db/seed`, `app/scripts`, `docs`
**Files scanned:** 35 implementation/docs files from `rg --files` plus phase artifacts
**Pattern extraction date:** 2026-05-21
