# Temporary Production Data Entry

This document is retained as a retired historical note for the pre-cutover Node
operator bridge. It is not the current data-entry runbook.

Routine production collection management now uses the Rust private controller
and static admin shell:

- Open `/admin` on the deployed hostname.
- Log in through `/admin/api/login` with the single-admin credential.
- Create and edit item metadata through the Phase 6 admin workflow.
- Upload, replace, remove, and select primary images through same-origin
  `/admin/api/*` requests.
- Review edit history, pending changes, cleanup warnings, and publish status in
  the admin shell.
- Publish through incremental or full static release actions.

The retired bridge used a Node runtime, SSH tunnel, bearer token, and
`/api/operator/catalog` routes. Those paths are historical only. Caddy must keep
`/api/operator/*` blocked, and production deploys no longer start the Node app.
Do not use the old bridge for current create, edit, upload, delete, or publish
work.

Keep `AUTOGRAPHS_OPERATOR_API_TOKEN` in the operator shell or secret manager
only when a non-management diagnostic still needs it. Do not paste real tokens,
admin passwords, Oracle credentials, Object Storage coordinates, object keys, or
Terraform state into repository files, public docs, chat logs, or browser-visible
pages.

For current verification, publish through the Rust controller/publisher path and
open the generated public pages through the deployed static site:

- `/collection`
- `/items/<item-slug>/`
- `/data/items/<item-slug>.json`
- generated derivative URLs under `/media/...`

The browser should never need a direct Object Storage URL, and retired
`/api/catalog/*` image streams should not appear in current public pages.
