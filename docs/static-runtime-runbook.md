# Static Runtime Foundation Runbook

## Local Controller Seed Path

The Phase 5 static admin shell is served at `/admin` by the private controller
routing path once Caddy wiring is deployed. It is a minimal seed/publish tool,
not the polished Phase 6 admin workflow. Keep `/admin` and `/admin/api/*`
behind the authenticated private-controller boundary; the browser shell relies
on the HTTP-only session cookie and same-origin mutation checks.

The GitHub production deploy starts the controller with persistent Oracle and
OCI instance-principal media adapters. Configure these repo-level values before
deploying so Ansible renders `/opt/autographs/env/controller.env`
intentionally:

```text
AUTOGRAPHS_CONTROLLER_DB_PROVIDER=oracle
AUTOGRAPHS_CONTROLLER_MEDIA_STORAGE_PROVIDER=oci-instance-principal
OCI_MEDIA_NAMESPACE=replace-with-object-storage-namespace
OCI_MEDIA_BUCKET_NAME=autographs-media-prod
```

Then restart or redeploy `autographs-controller.service`. Do not hand-edit
`controller.env` as the durable live switch; the next Ansible deploy owns that
file and will render values from deploy variables. The controller-specific file
sets `OCI_AUTH_MODE=instance_principal`, and the runtime dynamic group grants
media-bucket-scoped Object Storage access.

Start the controller with local-only values after loading the local admin
credential values from an untracked environment file:

```bash
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

Validate a promoted release from the runtime VM through Caddy's localhost-only
static listener. The listener serves `${AUTOGRAPHS_STATIC_RELEASE_ROOT}/current`
as its web root so these checks use the same paths as the public hostname:

```bash
curl --fail --silent \
  "http://127.0.0.1:8081/collection/"
```

Responses expose item IDs and public-safe status only. They do not return
Object Storage namespace, bucket name, original object key, or direct Object
Storage URLs. Private original keys are generated as:

```text
originals/{item-uuid}/{image-uuid}
```

## Live Persistence Smoke

The Oracle Autonomous Database and OCI Object Storage persistence smoke was
required for Phase 5 closeout and remains the operator-run verification path for
future controller persistence or private media changes. Ordinary CI skips it
because it needs live tenancy credentials. Supply the runtime
wallet/connect variables and instance-principal media coordinates through the
operator environment, then run:

```bash
AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE=true \
  cargo test --manifest-path controller/Cargo.toml \
  --features live-persistence live_persistence_smoke -- --ignored --nocapture
```

The smoke creates one draft item, uploads one private original with a UUID-only
object key, reads both records back, and cleans up the smoke item and object.
Phase 5 closeout recorded this proof against the live OCI tenancy; rerun it when
controller persistence, Oracle connectivity, OCI instance-principal media
access, or cleanup behavior changes.

Before running the smoke, confirm the database has been initialized from the
canonical controller schema end state in `controller/db/schema.sql`. The retired
Node app migrations have already been applied to the live environment and are no
longer part of the active repository tree; fresh ADB bootstrap or recovery
should start from that end-state schema rather than replaying the retired
migration chain. The probe performs a read-only schema preflight and stops
before inserting an item or uploading an object when the static-runtime schema
is absent.

The native Oracle probe uses Oracle Instant Client and the same wallet alias as
the deployed controller. It requires `ORACLE_DB_CONNECT_STRING`,
`ORACLE_DB_USER`, and the matching Oracle credential; the smoke container sets
`TNS_ADMIN` to the mounted wallet directory. Instance-principal media access
requires `OCI_AUTH_MODE`, `OCI_MEDIA_NAMESPACE`, `OCI_MEDIA_BUCKET_NAME`, and
the runtime dynamic-group policy for the media bucket.

### Run the Smoke as a Temporary VM Container

To prove the runtime VM network path without installing Rust on the VM, build
and export the one-shot smoke image on a trusted Linux `amd64` workstation:

```bash
SMOKE_VERSION="$(git rev-parse --short HEAD)"
SMOKE_IMAGE="localhost/autographs-live-persistence-smoke:${SMOKE_VERSION}"

docker build \
  --file controller/Dockerfile.smoke \
  --tag "${SMOKE_IMAGE}" \
  .

docker save \
  --output /tmp/autographs-live-persistence-smoke.tar \
  "${SMOKE_IMAGE}"

scp /tmp/autographs-live-persistence-smoke.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-live-persistence-smoke.tar
```

On the runtime VM, create `/opt/autographs/env/live-persistence-smoke.env` with
mode `0600`. Do not commit this file. It must contain the Oracle connection,
wallet, and OCI media coordinates used by the deployed controller, including:

```text
AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE=true
ORACLE_DB_USER=ADMIN
ORACLE_DB_CONNECT_STRING=autographsdb_medium
ORACLE_DB_WALLET_DIR=/opt/autographs/wallet
OCI_REGION=us-ashburn-1
OCI_AUTH_MODE=instance_principal
OCI_MEDIA_NAMESPACE=replace-with-object-storage-namespace
OCI_MEDIA_BUCKET_NAME=autographs-media-prod
```

The smoke must run on an OCI instance that can reach the instance metadata
service and belongs to the runtime dynamic group with media-bucket object
permissions.

Load and run the image with Podman:

```bash
SMOKE_VERSION="<git-short-sha-used-during-build>"
SMOKE_IMAGE="localhost/autographs-live-persistence-smoke:${SMOKE_VERSION}"
SMOKE_WALLET_DIR="/tmp/autographs-smoke-wallet"

sudo install -d -m 0700 -o opc -g opc /opt/autographs/env
chmod 0600 /opt/autographs/env/live-persistence-smoke.env
sudo rm -rf "${SMOKE_WALLET_DIR}"
sudo cp -a /opt/autographs/wallet "${SMOKE_WALLET_DIR}"

sudo podman load --input /tmp/autographs-live-persistence-smoke.tar
sudo podman run --rm \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --volume "${SMOKE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
  "${SMOKE_IMAGE}"
```

The image contains the compiled smoke-test executable, CA certificates, and
Oracle Instant Client. It does not contain the Oracle wallet, database
credential, or Object Storage credentials.

Use a copied wallet directory for one-shot smoke containers instead of mounting
the controller's live wallet path. The deployed controller owns
`/opt/autographs/wallet` with a private SELinux label; giving each smoke run its
own copied wallet lets Podman apply a separate private label without relabeling
the controller's mounted secret directory.

### Clean Up Interrupted Live Smoke Data

If a live persistence smoke is killed before its `Drop` cleanup runs, use the
same one-shot image and protected VM env file to remove leftover Oracle rows and
Object Storage objects. Set one or both cleanup variables; values can be comma
or newline separated:

```text
AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_ITEM_IDS=3f14e408-d4a7-4ef7-91fe-4ec10b3ea745
AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_OBJECT_KEYS=originals/3f14e408-d4a7-4ef7-91fe-4ec10b3ea745/949a003f-ba09-4fa2-bf7e-285ffdc187b4
```

Then run the persistence smoke image normally:

```bash
SMOKE_VERSION="<git-short-sha-used-during-build>"
SMOKE_IMAGE="localhost/autographs-live-persistence-smoke:${SMOKE_VERSION}"
SMOKE_WALLET_DIR="/tmp/autographs-smoke-wallet"

sudo rm -rf "${SMOKE_WALLET_DIR}"
sudo cp -a /opt/autographs/wallet "${SMOKE_WALLET_DIR}"

sudo podman run --rm \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --volume "${SMOKE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
  "${SMOKE_IMAGE}"
```

Cleanup mode runs before the normal smoke gate, deletes matching
`autograph_images`, `autograph_item_tags`, and `autograph_items` rows, deletes
the listed Object Storage objects through instance principal auth, and verifies
the database counts are zero. Remove the cleanup variables from the env file
before running the normal smoke again.

## Live Static Publish Smoke

### Prerequisite: Deploy the Staged Controller and Caddy Wiring

This smoke does not deploy the Rust controller. It assumes the Phase 5 runtime
wiring from PR 94 has already been deployed to the VM. That deployment installs:

- `autographs-controller.service`, running the Rust controller on the private
  `autographs` Podman network.
- Caddy `/admin/api/*` reverse proxying to `autographs-controller:8080`.
- Caddy `/admin/*` serving the static admin shell.
- Caddy `127.0.0.1:8081` host binding for the generated `current` static root.
- The shared `autographs-static.volume` mounted into the controller and Caddy.

Until that staged deployment is present, the smoke cannot reach
`http://autographs-controller:8080` or `http://autographs-caddy:8081`.
Deploy PR 94 through the normal deployment workflow, or manually install the
same controller quadlet, Caddyfile, static volume, and admin-shell files on the
VM before running this checkpoint.

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
SMOKE_VERSION="$(git rev-parse --short HEAD)"
SMOKE_IMAGE="localhost/autographs-live-static-publish-smoke:${SMOKE_VERSION}"

docker build \
  --file controller/Dockerfile.static-smoke \
  --build-arg AUTOGRAPHS_SMOKE_IMAGE_VERSION="${SMOKE_VERSION}" \
  --tag "${SMOKE_IMAGE}" \
  .

docker save \
  --output /tmp/autographs-live-static-publish-smoke.tar \
  "${SMOKE_IMAGE}"

scp /tmp/autographs-live-static-publish-smoke.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-live-static-publish-smoke.tar
```

On the VM, extend the protected smoke environment file with:

```text
AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true
AUTOGRAPHS_CONTROLLER_BASE_URL=http://autographs-controller:8080
AUTOGRAPHS_STATIC_PREVIEW_BASE_URL=http://autographs-caddy:8081
AUTOGRAPHS_STATIC_RELEASE_ROOT=/var/lib/autographs/static
AUTOGRAPHS_OPERATOR_API_TOKEN=replace-with-runtime-operator-token
```

Keep the Oracle and instance-principal media values from the live persistence
smoke in the same protected file. Load and run the one-shot image on the
private Podman network:

```bash
SMOKE_VERSION="<git-short-sha-used-during-build>"
SMOKE_IMAGE="localhost/autographs-live-static-publish-smoke:${SMOKE_VERSION}"
SMOKE_WALLET_DIR="/tmp/autographs-smoke-wallet"

sudo rm -rf "${SMOKE_WALLET_DIR}"
sudo cp -a /opt/autographs/wallet "${SMOKE_WALLET_DIR}"

sudo podman load --input /tmp/autographs-live-static-publish-smoke.tar
sudo podman run --rm \
  --network autographs \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --volume "${SMOKE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
  "${SMOKE_IMAGE}"
```

The static smoke result was recorded for Phase 5 closeout. The public hostname
now serves generated output through Caddy; rerunning the smoke proves that the
deployed Rust/static path can still publish a fresh item end to end and remove
it again.
If a failed run stops before cleanup, search Oracle for a title beginning with
`Live Static Smoke`, remove that temporary draft through the available
operator-maintenance path, and delete its logged `originals/{item-id}/{image-id}`
object from Object Storage. If the static smoke passes but logs a timeout while
deleting the private original, use the live persistence smoke cleanup mode with
the logged item ID and object key to confirm the database rows and Object Storage
object are absent.

When debugging Object Storage cleanup, use the VM-installed OCI CLI to verify
that instance-principal policy allows deletes independently from the Rust media
client:

```bash
oci os object delete \
  --auth instance_principal \
  --namespace-name "${OCI_MEDIA_NAMESPACE}" \
  --bucket-name "${OCI_MEDIA_BUCKET_NAME}" \
  --object-name "originals/<item-id>/<image-id>" \
  --force
```

If the CLI delete is unauthorized, check that the runtime dynamic group has
`manage objects` on the media bucket. If the CLI delete succeeds but the Rust
cleanup path does not, investigate the controller media client or smoke cleanup
image rather than OCI IAM.

### Controller Logs and Verbosity

The controller emits structured operation logs to container stdout/stderr. Normal
`info` logs include admin catalog create/update calls, image uploads,
publication status changes, static publish starts/completions, release IDs,
artifact counts, and elapsed times. Route failures log the underlying repository,
media, or publisher error before returning the public HTTP status.

The deployed env file sets:

```text
RUST_LOG=autographs_controller=info,tower_http=info
```

For a debugging session, temporarily raise the controller verbosity in
`/opt/autographs/env/app.env`, restart `autographs-controller`, and inspect
`sudo podman logs -f autographs-controller`:

```text
RUST_LOG=autographs_controller=debug,tower_http=debug
```

Use `autographs_controller=trace` only for short sessions; it is intended for
live diagnosis and can produce noisy logs.

## Candidate Validation

After any seed or metadata change, trigger an incremental publish and inspect
the promoted candidate privately:

```bash
curl --fail --silent http://127.0.0.1:8081/collection/
curl --fail --silent http://127.0.0.1:8081/data/collection.json
curl --fail --silent http://127.0.0.1:8081/data/facets.json
```

Check `/var/lib/autographs/static/failed/` inside the controller container when a candidate fails. The publisher
retains only the latest failed candidate for diagnosis and leaves `current`
pointing at the last validated release.

## Full Rebuild
