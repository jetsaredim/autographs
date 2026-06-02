# Static Runtime Foundation Runbook

## Local Controller Seed Path

The Phase 5 static admin shell is served at `/admin` by the private controller
routing path once Caddy wiring is deployed. It is a minimal seed/publish tool,
not the polished Phase 6 admin workflow. Keep `/admin` and `/admin/api/*`
behind the authenticated private-controller boundary; the browser shell relies
on the HTTP-only session cookie and same-origin mutation checks.

The staged deploy starts the controller with local persistence adapters in
`/opt/autographs/env/controller.env`. For the live static publish proof, update
that protected VM-local file to:

```text
AUTOGRAPHS_CONTROLLER_DB_PROVIDER=oracle
AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER=oci-s3
OCI_S3_ENDPOINT=https://replace-with-namespace.compat.objectstorage.us-ashburn-1.oraclecloud.com
OCI_S3_ACCESS_KEY=replace-with-customer-secret-access-key
OCI_S3_SECRET_KEY=replace-with-customer-secret-secret-key
```

Then restart `autographs-controller.service`. Keep the Customer Secret values
on the VM/operator secret path; do not add them to GitHub-hosted workflows.

Start the controller with local-only values:

```bash
export AUTOGRAPHS_ADMIN_PASSWORD=local-only-password
export AUTOGRAPHS_ADMIN_SECURE_COOKIES=false
export AUTOGRAPHS_OPERATOR_API_TOKEN=local-operator-token
cargo run --manifest-path controller/Cargo.toml
```

Create a draft item through the private bearer-token boundary:

```bash
curl -fsS http://127.0.0.1:8080/admin/api/items \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  --data '{"title":"Signed card","signer":"Example Signer","category":"Cards","tags":["fixture"]}'
```

Upload one private original using the returned item ID:

```bash
curl -fsS "http://127.0.0.1:8080/admin/api/items/${ITEM_ID}/images" \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -F "image=@./example.jpg;type=image/jpeg"
```

Update publication status:

```bash
curl -fsS "http://127.0.0.1:8080/admin/api/items/${ITEM_ID}/publication" \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  -H "Content-Type: application/json" \
  --data '{"publicationStatus":"published"}'
```

Publish and inspect the generated static release:

```bash
curl -fsS http://127.0.0.1:8080/admin/api/publish/incremental \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}" \
  --request POST

curl -fsS http://127.0.0.1:8080/admin/api/publish/status \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}"
```

Use `POST /admin/api/publish/full` for an explicit full rebuild. Successful
publishes write candidates under `${AUTOGRAPHS_STATIC_RELEASE_ROOT}/releases/`
and atomically update `${AUTOGRAPHS_STATIC_RELEASE_ROOT}/current` only after
validation passes.

Before the public static cutover, validate a promoted release from the runtime
VM through Caddy's localhost-only preview listener. Use `/current/` to inspect
the promoted candidate or `/releases/${RELEASE_ID}/` to inspect a specific
release:

```bash
curl --fail --silent \
  "http://127.0.0.1:8081/current/collection/"

curl --fail --silent \
  "http://127.0.0.1:8081/releases/${RELEASE_ID}/collection/"
```

Responses expose item IDs and public-safe status only. They do not return
Object Storage namespace, bucket name, original object key, or direct Object
Storage URLs. Private original keys are generated as:

```text
originals/{item-uuid}/{image-uuid}
```

## Required Live Persistence Smoke

The Oracle Autonomous Database and OCI Object Storage persistence smoke is
mandatory before Phase 5 verification passes, even though ordinary CI skips it.
Supply the runtime wallet/connect variables and OCI S3 compatibility Customer
Secret credentials through the operator environment, then run:

```bash
AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE=true \
  cargo test --manifest-path controller/Cargo.toml \
  --features live-persistence live_persistence_smoke -- --ignored --nocapture
```

The smoke must create one draft item, upload one private original with a
UUID-only object key, read both records back, and clean up the smoke item and
object. Do not mark Phase 5 verified until this command has passed against the
live OCI tenancy.

Apply the committed app migrations before running the smoke. The probe performs
a read-only schema preflight and stops before inserting an item or uploading an
object when `002_static_runtime_foundation.sql` has not been applied.

The native Oracle probe uses Oracle Instant Client and the same wallet alias as
the deployed app. It requires `ORACLE_DB_CONNECT_STRING`, `ORACLE_DB_USER`, and
`ORACLE_DB_PASSWORD`; the smoke container sets `TNS_ADMIN` to the mounted wallet
directory. OCI S3 compatibility requires `OCI_S3_ENDPOINT`,
`OCI_S3_ACCESS_KEY`, `OCI_S3_SECRET_KEY`, `OCI_MEDIA_NAMESPACE`, and
`OCI_MEDIA_BUCKET_NAME`.

### Run the Smoke as a Temporary VM Container

To prove the runtime VM network path without installing Rust on the VM, build
and export the one-shot smoke image on a trusted Linux `amd64` workstation:

```bash
docker build \
  --file controller/Containerfile.smoke \
  --tag localhost/autographs-live-persistence-smoke:phase-05 \
  .

docker save \
  --output /tmp/autographs-live-persistence-smoke.tar \
  localhost/autographs-live-persistence-smoke:phase-05

scp /tmp/autographs-live-persistence-smoke.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-live-persistence-smoke.tar
```

On the runtime VM, create `/opt/autographs/env/live-persistence-smoke.env` with
mode `0600`. Do not commit this file. It must contain:

```text
AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE=true
ORACLE_DB_USER=ADMIN
ORACLE_DB_PASSWORD=replace-with-runtime-db-password
ORACLE_DB_CONNECT_STRING=autographsdb_medium
ORACLE_DB_WALLET_DIR=/opt/autographs/wallet
OCI_REGION=us-ashburn-1
OCI_S3_ENDPOINT=https://replace-with-namespace.compat.objectstorage.us-ashburn-1.oraclecloud.com
OCI_S3_ACCESS_KEY=replace-with-customer-secret-access-key
OCI_S3_SECRET_KEY=replace-with-customer-secret-secret-key
OCI_MEDIA_NAMESPACE=replace-with-object-storage-namespace
OCI_MEDIA_BUCKET_NAME=autographs-media-prod
```

Load and run the image with Podman:

```bash
sudo install -d -m 0700 -o opc -g opc /opt/autographs/env
chmod 0600 /opt/autographs/env/live-persistence-smoke.env

sudo podman load --input /tmp/autographs-live-persistence-smoke.tar
sudo podman run --rm \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --volume /opt/autographs/wallet:/opt/autographs/wallet:ro \
  localhost/autographs-live-persistence-smoke:phase-05
```

The image contains the compiled smoke-test executable, CA certificates, and
Oracle Instant Client. It does not contain the Oracle wallet, database password,
or OCI Customer Secret credentials.

## Live Static Publish Smoke

The final Phase 5 checkpoint is a second credential-gated smoke that exercises
the deployed controller and Caddy preview as black boxes. It creates a uniquely
named draft through `/admin/api/*`, uploads a valid private image, verifies the
Oracle row and OCI Object Storage object, publishes a static release, and
fetches the browse page, item HTML, item JSON, facets, and generated WebP
derivatives through Caddy. It then unpublishes the item, runs an incremental
publish, confirms that stale public files return `404`, and removes the
temporary Oracle row and private original.

Build and export the temporary image on a trusted Linux `amd64` workstation:

```bash
docker build \
  --file controller/Containerfile.static-smoke \
  --tag localhost/autographs-live-static-publish-smoke:phase-05 \
  .

docker save \
  --output /tmp/autographs-live-static-publish-smoke.tar \
  localhost/autographs-live-static-publish-smoke:phase-05

scp /tmp/autographs-live-static-publish-smoke.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-live-static-publish-smoke.tar
```

On the VM, extend the protected smoke environment file with:

```text
AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true
AUTOGRAPHS_CONTROLLER_BASE_URL=http://autographs-controller:8080
AUTOGRAPHS_STATIC_PREVIEW_BASE_URL=http://autographs-caddy:8081/current
AUTOGRAPHS_STATIC_RELEASE_ROOT=/opt/autographs/static
AUTOGRAPHS_OPERATOR_API_TOKEN=replace-with-runtime-operator-token
```

Keep the Oracle and OCI S3 compatibility values from the live persistence smoke
in the same protected file. Load and run the one-shot image on the private
Podman network:

```bash
sudo podman load --input /tmp/autographs-live-static-publish-smoke.tar
sudo podman run --rm \
  --network autographs \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --volume /opt/autographs/wallet:/opt/autographs/wallet:ro \
  localhost/autographs-live-static-publish-smoke:phase-05
```

The smoke must pass before the public hostname is switched to generated output.
If a failed run stops before cleanup, search Oracle for a title beginning with
`Live Static Smoke`, remove that temporary draft through the available
operator-maintenance path, and delete its logged `originals/{item-id}/{image-id}`
object from Object Storage.

## Candidate Validation

After any seed or metadata change, trigger an incremental publish and inspect
the promoted candidate privately:

```bash
curl --fail --silent http://127.0.0.1:8081/current/collection/
curl --fail --silent http://127.0.0.1:8081/current/data/collection.json
curl --fail --silent http://127.0.0.1:8081/current/data/facets.json
```

Check `/opt/autographs/static/failed/` when a candidate fails. The publisher
retains only the latest failed candidate for diagnosis and leaves `current`
pointing at the last validated release.

## Full Rebuild

Use a full rebuild after schema-contract changes, recovery from an uncertain
release state, or before the planned public cutover:

```bash
curl --fail --silent --request POST \
  http://127.0.0.1:8080/admin/api/publish/full \
  -H "Authorization: Bearer ${AUTOGRAPHS_OPERATOR_API_TOKEN}"
```

Incremental publishing is conservative in Phase 5: it regenerates the complete
public surface after copying the prior release because durable per-item change
events do not exist yet.

## Cutover

Planned downtime is acceptable for the first static-runtime cutover. Before
editing Caddy's public root:

1. Deploy the controller/static-volume shape and switch the protected
   `controller.env` file to Oracle plus `oci-s3`.
2. Run the live persistence smoke and live static publish smoke.
3. Run an explicit full rebuild and inspect `/current/` through port `8081`.
4. Update Caddy so the public root serves `/srv/autographs/static/current`.
5. Verify `/`, `/collection/`, one `/items/{slug}/` page, JSON, facets, and
   generated media from the public hostname.

Recovery is roll-forward oriented: fix the controller or source data, run a
full rebuild, validate the candidate privately, and promote the corrected
release. Keep the Next.js app available until the static hostname verification
passes.

## Retirement Checks

After public static cutover verification passes:

- Return `404` for `/api/catalog/*`; static JSON and generated derivatives
  replace public catalog APIs and app-mediated image streaming.
- Return `404` for `/api/operator/*`; the Rust `/admin/api/*` boundary replaces
  the temporary Node operator bridge.
- Retire `.github/workflows/data-smoke.yml` only after the live static publish
  smoke is the documented production verification path.
- Remove the public Next.js runtime from Caddy and later from deploy wiring only
  after static browse, detail, filtering, generated media, publish, and
  unpublish checks pass on the public hostname.
- Revoke temporary OCI S3 Customer Secret credentials after the checkpoint and
  replace them with the intended runtime secret handling before routine use.
