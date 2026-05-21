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
