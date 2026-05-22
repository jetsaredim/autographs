# Temporary Production Data Entry

## Purpose

Phase 3 keeps production data entry intentionally procedural until the Phase 4 admin workflow exists. The temporary path lets the operator create and update catalog records through the deployed app's existing operator API while keeping private media and Oracle metadata connected through the catalog service.

## Security Boundary

Operator endpoints must not be exposed through public Caddy routes in Phase 3. Treat this as an operator-only bridge reached from your workstation through an SSH tunnel to the app container listener on the runtime VM.

Keep `AUTOGRAPHS_OPERATOR_API_TOKEN` in the operator shell or secret manager. Do not paste it into browser-visible pages, public docs with real values, chat logs, or repository files.

## SSH Tunnel

Open a tunnel from the operator machine to the runtime VM:

```bash
ssh -L <local-port>:127.0.0.1:3000 opc@<runtime-public-ip>
```

Use a local port that is free on your workstation. Keep the SSH session open while sending operator requests.

## Token-Guarded Calls

Send temporary data-entry requests through the forwarded local port and include the operator token:

```bash
curl \
  -H "Authorization: Bearer <AUTOGRAPHS_OPERATOR_API_TOKEN>" \
  -H "Content-Type: application/json" \
  http://127.0.0.1:<forwarded-port>/api/operator/...
```

Create requests go to `POST /api/operator/catalog`. Update and image-attachment requests go to `PATCH /api/operator/catalog/{id}`. Image bodies are base64 encoded in the existing operator API request shape.

For a normal operator session, set these shell variables once:

```bash
export AUTOGRAPHS_OPERATOR_API_TOKEN="<real-token-from-secret-store>"
export AUTOGRAPHS_OPERATOR_BASE_URL="http://127.0.0.1:<forwarded-port>"
```

Do not commit a file containing the real token.

## Prepare Image Payloads

The temporary API accepts image bytes as base64 inside JSON. From the operator workstation, prepare one or more local image variables:

```bash
export FRONT_IMAGE_BASE64="$(base64 -w 0 ./front.jpg)"
export BACK_IMAGE_BASE64="$(base64 -w 0 ./back.jpg)"
```

Use the matching `contentType` for the file you are uploading, such as `image/jpeg`, `image/png`, or `image/webp`.

## Create A Published Item With Images

Use `POST /api/operator/catalog` to create the metadata record and upload initial images in one call. The API returns the created item, including its generated `id`.

```bash
curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -X POST \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog" \
  --data @- <<JSON
{
  "title": "Signed Example Card",
  "signer": "Example Signer",
  "description": "Short public description of the signed item.",
  "category": "Star Wars CCG",
  "tags": ["star wars", "trading cards"],
  "objectReference": "Premiere example card",
  "eventName": "Example Signing",
  "eventLocation": "Orlando, FL",
  "source": "Convention signing",
  "inscription": "To Jared",
  "certificationCompany": "PSA",
  "certificationId": "EXAMPLE123",
  "estimatedYear": 2026,
  "publicationStatus": "published",
  "imageUploads": [
    {
      "filename": "front.jpg",
      "contentType": "image/jpeg",
      "bodyBase64": "${FRONT_IMAGE_BASE64}",
      "isPrimary": true,
      "sortOrder": 0,
      "altText": "Signed Example Card front"
    },
    {
      "filename": "back.jpg",
      "contentType": "image/jpeg",
      "bodyBase64": "${BACK_IMAGE_BASE64}",
      "isPrimary": false,
      "sortOrder": 1,
      "altText": "Signed Example Card back"
    }
  ]
}
JSON
```

Recommended fields for the first few real uploads:

- `title`: item/card or object title shown on the detail page.
- `signer`: signer name used for browsing and filtering.
- `description`: concise public note.
- `category`: currently shown as the `Game` filter.
- `tags`: currently shown as `IP / Genre` filter options and detail chips.
- `objectReference`: card set, product, or object identifier.
- `eventName`, `eventLocation`, `source`: provenance details.
- `certificationCompany`, `certificationId`: omit or set to `null` if not certified.
- `estimatedYear`: omit or set to `null` if unknown.
- `publicationStatus`: use `published` when the item should appear publicly, otherwise `draft`.

## Update Metadata

Use `PATCH /api/operator/catalog/{id}` with an `item` object to update metadata. Only include fields you want to change.

```bash
export AUTOGRAPH_ITEM_ID="<id-returned-from-create>"

curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -X PATCH \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}" \
  --data @- <<JSON
{
  "item": {
    "description": "Updated public description.",
    "tags": ["star wars", "trading cards", "featured"],
    "publicationStatus": "published"
  }
}
JSON
```

## Attach More Images

Use the same `PATCH` endpoint with `imageUploads` to attach additional images to an existing item.

```bash
export EXTRA_IMAGE_BASE64="$(base64 -w 0 ./detail.jpg)"

curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -X PATCH \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}" \
  --data @- <<JSON
{
  "imageUploads": [
    {
      "filename": "detail.jpg",
      "contentType": "image/jpeg",
      "bodyBase64": "${EXTRA_IMAGE_BASE64}",
      "sortOrder": 2,
      "altText": "Signed Example Card inscription detail"
    }
  ]
}
JSON
```

If an attached image should become the primary collection-grid image, include `"isPrimary": true`. The service normalizes primary image selection so only one image remains primary.

## Verify The Upload

After a create or update, verify through the public read path, not by inspecting Oracle or Object Storage directly:

```bash
curl -sS "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/catalog/${AUTOGRAPH_ITEM_ID}"
```

Then open the public pages through the deployed site:

- `/collection`
- `/collection/<item-id>`

Images should load through app-mediated URLs shaped like `/api/catalog/{itemId}/images/{imageId}`. The browser should never need a direct Object Storage URL.

## Media and Metadata Path

Use the operator API so the deployed app writes Oracle metadata and private Object Storage images through the same catalog service used by the rest of the system. Published public pages should then read records through the public catalog service and display images only through `/api/catalog/{itemId}/images/{imageId}`.

## What Not To Do

- Do not hand-edit Oracle rows.
- Do not upload untracked Object Storage objects.
- Do not expose operator endpoints through public Caddy routes.
- Do not bypass the catalog service with ad hoc scripts that leave image objects and metadata disconnected.
- Do not store the real bearer token in committed examples.

## Retirement Path

Phase 4 replaces this bridge with the real single-admin collection workflow. When that workflow exists, retire the SSH-tunnel data-entry path or keep it only as a documented break-glass operator procedure.
