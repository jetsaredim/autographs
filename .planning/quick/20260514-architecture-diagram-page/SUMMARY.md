---
status: complete
completed: 2026-05-14
---

# Architecture Diagram Page Summary

Added a static `/architecture` page backed by committed draw.io source and an SVG export. The diagram was simplified into GitHub and OCI zones with numbered workflows explained in a table below the image. The Ilograph experiment was removed because it did not improve local website review.

## Changed

- Added `docs/architecture.drawio` as the editable draw.io diagram source.
- Added `app/public/architecture-diagram.svg` as the website-rendered diagram export.
- Simplified the diagram into large GitHub and OCI boxes with repo/CI/deploy/GHCR, VCN/subnet/VM/container stack, ADB, Object Storage, AI metadata processing, and Let's Encrypt interactions.
- Added step 9 for Caddy's Let's Encrypt certificate obtain/renew flow, with Let's Encrypt shown outside the OCI boundary as an external gray service.
- Renumbered workflow steps so manual Terraform tenancy bootstrap is step 1, followed by code push, validation, deploy, OCI runtime provisioning, image publishing, public traffic, private data access, and certificate management.
- Routed the admin-to-GitHub workflow as a smooth left-side path to reduce overlap with GitHub title text.
- Moved ADB and Object Storage to the right of the VCN, stacked vertically, with curved left-to-right Next.js data-service connections.
- Removed descriptions from VCN and subnet boxes and moved that detail into the workflow table.
- Kept the VM label while leaving only the Caddy and Next container boxes inside the VM boundary.
- Retargeted deployment and bootstrap workflow arrows to the outer OCI boundary while keeping Terraform details in the workflow table.
- Removed the extra layer ownership card section and simplified the page heading to focus on the system diagram.
- Added step 10 for the site admin content workflow: upload images/data through the app, store media in Object Storage, and use AI processing for metadata suggestions.
- Adjusted workflow routing so admin-to-repo flares out, GHCR enters the VM near the VM label, and admin content management enters through Caddy.
- Replaced the canonical draw.io source with the manually edited `architecture-1.drawio` version and regenerated the website SVG from that layout.
- Preserved the intentional duplicate step 10 markers, ended the deploy-to-GHCR arrow on the GHCR box edge, and aligned the admin-to-OCI/admin-to-Caddy origins on the right side of the Admin user box.
- Added `app/app/architecture/page.tsx` with concise solution overview copy, diagram display, and workflow table.
- Updated `app/app/page.tsx` with a link to the architecture page.
- Updated `app/app/globals.css` for a monochrome black/gray theme and diagram/table presentation.

## Validation

- `corepack pnpm --filter app lint`
- `corepack pnpm --filter app typecheck`

## Review

Local dev server is running at `http://127.0.0.1:3000/architecture` for visual review.
