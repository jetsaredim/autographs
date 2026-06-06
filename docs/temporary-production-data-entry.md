# Temporary Production Data Entry

## Purpose

Production data entry remains intentionally procedural until the Phase 5 static
runtime/private controller cutover checkpoint passes. This is the
current-until-cutover Node bridge: it lets the operator create, update, attach
image media to, and remove image media from catalog records through the
deployed app's operator API while keeping private media and Oracle metadata
connected through the catalog service.

## Security Boundary

Operator endpoints must not be exposed through public Caddy routes. Treat this as an operator-only bridge reached from your workstation through an SSH tunnel to the app container listener on the runtime VM.

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
  http://127.0.0.1:<forwarded-port>/api/operator/...
```

Create requests go to `POST /api/operator/catalog`.

Metadata updates and image attachments go to `PATCH /api/operator/catalog/{id}`.

Image deletion goes to `DELETE /api/operator/catalog/{id}/images/{imageId}`.

Full item deletion goes to `DELETE /api/operator/catalog/{id}`.

For a normal operator session, set these shell variables once:

```bash
export AUTOGRAPHS_OPERATOR_API_TOKEN="<real-token-from-secret-store>"
export AUTOGRAPHS_OPERATOR_BASE_URL="http://127.0.0.1:<forwarded-port>"
```

Do not commit a file containing the real token.

## Create A Published Item

Use `POST /api/operator/catalog` to create the metadata record. The API returns the created item, including its generated `id`.

The recommended temporary workflow is:

1. Create the item metadata with JSON.
2. Save the returned item `id`.
3. Attach one or more images with the multipart upload workflow.

This avoids giant base64 JSON payloads during normal operator work.

```bash
curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -X POST \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog" \
  --data-binary @- <<JSON
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
  "publicationStatus": "published"
}
JSON
```

Capture the returned item id:

```bash
export AUTOGRAPH_ITEM_ID="<id-returned-from-create>"
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
  --data-binary @- <<JSON
{
  "item": {
    "description": "Updated public description.",
    "tags": ["star wars", "trading cards", "featured"],
    "publicationStatus": "published"
  }
}
JSON
```

Use the response body to confirm the saved fields:

```bash
curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -X PATCH \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}" \
  --data-binary @update-metadata.json \
  | jq '.item | {id,title,description,category,tags,objectReference,eventName,eventLocation,source,estimatedYear,images}'
```

## Attach Images With Multipart Upload

Use `PATCH /api/operator/catalog/{id}` with multipart form data to attach one or more images to an existing item. This is the recommended manual operator path because it avoids base64-in-JSON payloads.

Do not include a manual `Content-Type` header for multipart uploads. Let `curl -F` set the multipart boundary.

```bash
export AUTOGRAPH_ITEM_ID="<id-returned-from-create>"

curl -sS -D /tmp/autographs-upload.headers \
  -o /tmp/autographs-upload.body \
  -w '\nHTTP %{http_code}\nContent-Type: %{content_type}\nDownloaded: %{size_download} bytes\nUploaded: %{size_upload} bytes\n' \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -X PATCH \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}" \
  -F "image=@./front.jpg;type=image/jpeg" \
  -F "altText=Signed Example Card front"

cat /tmp/autographs-upload.body | jq '.item.id, .item.images'
```

The route accepts either `image` or `images` as the file field name. Use the matching MIME type for the file, such as `image/jpeg`, `image/png`, or `image/webp`.

To attach multiple files in one request:

```bash
curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -X PATCH \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}" \
  -F "images=@./front.jpg;type=image/jpeg" \
  -F "images=@./back.jpg;type=image/jpeg" \
  | jq '.item.images'
```

If an attached image should become the primary collection-grid image, attach that image last or use the legacy JSON/base64 `imageUploads` path with `"isPrimary": true`. The service normalizes primary image selection so only one image remains primary.

## Legacy JSON/Base64 Image Uploads

The create and patch endpoints still accept image bytes as base64 inside JSON. This is useful for scripts, but it is not recommended for manual shell use because large base64 strings are hard to inspect and easy to quote incorrectly.

If you use this path, build the JSON dynamically. Do not put literal shell variables such as `${FRONT_IMAGE_BASE64}` into a static JSON file and expect `curl --data @file.json` to expand them.

```bash
export FRONT_IMAGE_BASE64="$(base64 -w 0 ./front.jpg)"

curl -sS \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -X PATCH \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}" \
  --data-binary @- <<JSON
{
  "imageUploads": [
    {
      "filename": "front.jpg",
      "contentType": "image/jpeg",
      "bodyBase64": "${FRONT_IMAGE_BASE64}",
      "isPrimary": true,
      "sortOrder": 0,
      "altText": "Signed Example Card front"
    }
  ]
}
JSON
```

## Delete An Image

Use `DELETE /api/operator/catalog/{id}/images/{imageId}` to remove a single image from an item and delete the backing private media object.

```bash
export AUTOGRAPH_ITEM_ID="<item-id>"
export AUTOGRAPH_IMAGE_ID="<image-id>"

curl -sS -D /tmp/autographs-delete-image.headers \
  -o /tmp/autographs-delete-image.body \
  -w '\nHTTP %{http_code}\nContent-Type: %{content_type}\nDownloaded: %{size_download} bytes\n' \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -X DELETE \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}/images/${AUTOGRAPH_IMAGE_ID}"

cat /tmp/autographs-delete-image.body | jq '.item.id, .item.images'
```

If the deleted image was the only image, the item remains published but has no public image until another image is attached.

## Delete An Item

Use `DELETE /api/operator/catalog/{id}` to remove a catalog item and delete its backing private media objects.

```bash
export AUTOGRAPH_ITEM_ID="<item-id>"

curl -sS -D /tmp/autographs-delete-item.headers \
  -o /tmp/autographs-delete-item.body \
  -w '\nHTTP %{http_code}\nContent-Type: %{content_type}\nDownloaded: %{size_download} bytes\n' \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -X DELETE \
  "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/operator/catalog/${AUTOGRAPH_ITEM_ID}"

cat /tmp/autographs-delete-item.body | jq .
```

A successful delete response is shaped like:

```json
{
  "deleted": {
    "id": "<item-id>",
    "imageCount": 2
  }
}
```

If you only want to remove an item from the public collection without deleting its metadata or media, update `publicationStatus` to `draft` or `archived` instead.

## Verify Public Read Paths

After a create, update, attach, or delete operation, verify through the public read path, not by inspecting Oracle or Object Storage directly:

```bash
curl -sS "${AUTOGRAPHS_OPERATOR_BASE_URL}/api/catalog/${AUTOGRAPH_ITEM_ID}" | jq .
```

Then open the public pages through the deployed site:

- `/collection`
- `/collection/<item-id>`

Images should load through app-mediated URLs shaped like `/api/catalog/{itemId}/images/{imageId}`. The browser should never need a direct Object Storage URL.

## Media and Metadata Path

Use the operator API so the deployed app writes Oracle metadata and private Object Storage images through the same catalog service used by the rest of the system. Published public pages should then read records through the public catalog service and display images only through `/api/catalog/{itemId}/images/{imageId}`.

Delete images and full catalog items through the operator API as well so metadata and Object Storage cleanup stay connected.

## What Not To Do

- Do not hand-edit Oracle rows.
- Do not upload untracked Object Storage objects.
- Do not expose operator endpoints through public Caddy routes.
- Do not bypass the catalog service with ad hoc scripts that leave image objects and metadata disconnected.
- Do not store the real bearer token in committed examples.
- Do not include a manual `Content-Type: application/json` header on multipart `curl -F` requests.

## Retirement Path

Phase 5 replaces this bridge with the Rust private controller and minimal
static admin seed/publish path after the live static publish smoke and public
hostname checks pass. At retirement, Caddy must continue returning `404` for
`/api/operator/*`; normal seed and publish operations move to `/admin` and
`/admin/api/*`. Keep the Node tunnel only as an explicitly documented
break-glass procedure while the Next.js container still exists. Phase 6 turns
the Rust foundation into the polished single-admin collection workflow.
