# Phase 3: Public Gallery MVP - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-21
**Phase:** 3-Public Gallery MVP
**Areas discussed:** Gallery browsing shape, Filtering and discovery, Item detail presentation, Image viewing behavior, Empty/loading/error states, Site brand, Image access friction, Temporary production data entry

---

## Gallery Browsing Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Catalog grid | Image-forward cards in a responsive grid, optimized for browsing a collection quickly. | ✓ |
| Editorial showcase | Larger featured presentation with fewer items visible at once, more like a curated portfolio. | |
| Compact list | Denser rows with thumbnails and metadata, optimized for scanning/search over visual browsing. | |

**User's choice:** The landing page should be an overview with guided actions. The collection page should use a grid of smaller thumbnails that is easy to navigate.
**Notes:** The landing page should offer **View Collection** and **Surprise Me**. Clicking collection images should open a clean detail page.

---

## Filtering and Discovery

| Option | Description | Selected |
|--------|-------------|----------|
| Curated facets | Only show selected useful filters like card game, IP/category, and meaningful tags. | ✓ |
| All public tags/categories | Expose whatever published items have in the database, sorted/grouped cleanly. | |
| Hybrid | Curated primary facets first, with an all-tags area for deeper browsing. | Deferred |

**User's choice:** Curated facets are the MVP. Hybrid browsing can be reconsidered later once there is enough real data to experiment with.
**Notes:** Filtering should use a dropdown/filterable tag-cloud style menu. Selecting filters should reduce or enlarge the visible collection grid.

---

## Item Detail Presentation

| Option | Description | Selected |
|--------|-------------|----------|
| Essential facts only | Show signer, title, category/IP/card game, rarity, certification, year/event/source. | |
| Full catalog metadata | Show nearly everything stored. | |
| Grouped metadata sections | Essential facts up top, then supporting sections like provenance, certification, tags, and collection notes. | ✓ |

**User's choice:** Use grouped metadata sections, with room to iterate once implementation reaches real data.
**Notes:** Important fields include signer, item/card title, card game/IP/category, rarity when applicable, certification, year, event, and source.

---

## Image Viewing Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| No URL change | Thumbnail selection only swaps the focused image in-place. | ✓ |
| URL hash/query update | Selected image is reflected in the URL. | |
| Dedicated image route/lightbox | Each image can open into a larger focused view. | |

**User's choice:** Thumbnail selection swaps the focused image in-place without URL/hash/query changes.
**Notes:** If more than one image is available, show thumbnails below the primary image; clicking a thumbnail replaces the focused image.

---

## Empty, Loading, and Error States

| Option | Description | Selected |
|--------|-------------|----------|
| Quiet and practical | Short messages with clear recovery actions. | |
| Warm/editorial | Personable copy that fits the collection's tone. | ✓ |
| Minimal | Sparse system-like messages. | |

**User's choice:** Use warm editorial content for public no-result/not-found/media-missing states.
**Notes:** Use short movie-reference quotes about not finding things, such as "These aren't the droids you're looking for" and "X never, ever marks the spot." Quotes should be stylized as quote blocks with proper attribution and paired with practical actions.

---

## Site Brand

| Option | Description | Selected |
|--------|-------------|----------|
| Generic Autographs | Keep the proof-of-life naming. | |
| Jared Greenwald's Autograph Gallery | Use the collector/site name as the public brand. | ✓ |

**User's choice:** The public site should be named **Jared Greenwald's Autograph Gallery**.
**Notes:** Use correct capitalization and replace generic proof-of-life presentation on public UX surfaces.

---

## Surprise Me

| Option | Description | Selected |
|--------|-------------|----------|
| Respect filters | Collection-page control chooses from the active filtered subset. | |
| All published items | Choose from the whole published collection. | ✓ |
| Contextual | Landing page uses all items; collection page respects filters. | |

**User's choice:** "Surprise Me" should only ever be triggered from the main page and should select from all published items.
**Notes:** The collection page does not need a Surprise Me control.

---

## Image Access Friction

| Option | Description | Selected |
|--------|-------------|----------|
| Direct browser images | Standard image rendering with no extra friction. | |
| Anti-casual extraction | Avoid direct storage exposure and make casual saving harder. | ✓ |
| DRM-like protection | Attempt to prevent all extraction. | |

**User's choice:** Make casual image extraction structurally difficult, while preserving the realistic limitation that browser-viewable images can still be extracted by determined users.
**Notes:** Do not expose Object Storage URLs, bucket paths, object keys, or storage credentials. Disable/default-prevent context menus on image displays where practical and avoid standalone image links.

---

## Temporary Production Data Entry

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub workflow seeding | Manual workflow creates a small fixture set. | |
| SSH tunnel to operator endpoints | Operator forwards local port to deployed app loopback and calls token-guarded endpoints. | ✓ |
| Build admin/import UI now | Pull Phase 4 capability into Phase 3. | |

**User's choice:** Document a lightweight SSH-tunnel procedure instead of investing heavily in a one-time production seeding workflow.
**Notes:** Commands should run from the operator machine against `http://127.0.0.1:<forwarded-port>/api/operator/...` with `Authorization: Bearer <AUTOGRAPHS_OPERATOR_API_TOKEN>`. Do not expose operator endpoints through public Caddy routes.

---

## the agent's Discretion

- Exact responsive grid breakpoints and visual card density.
- Exact metadata section labels/order once real data is visible.
- Exact implementation of curated facets and quote rotation, as long as the locked behavior is preserved.

## Deferred Ideas

- Revisit hybrid filtering once the collection has enough real data.
- Replace SSH-tunnel operator data entry with Phase 4 admin workflow.
- Consider a richer lightbox or dedicated image experience after the MVP if needed.
