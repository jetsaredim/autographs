# Static Runtime Foundation Runbook

## Local Controller Admin Path

The static admin shell is served at `/admin` by the private controller routing
path once Caddy wiring is deployed. It is the current Phase 6 collection
workflow. Keep `/admin` and `/admin/api/*` behind the authenticated
private-controller boundary; collection-management calls rely on the HTTP-only
session cookie and same-origin mutation checks.

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
export AUTOGRAPHS_PUBLIC_ORIGIN=http://127.0.0.1:8080
export AUTOGRAPHS_ADMIN_PASSWORD=replace-with-local-password
cargo run --manifest-path controller/Cargo.toml
```

Create a session cookie through the same login path the browser shell uses:

```bash
curl -fsS -c /tmp/autographs-admin.cookies \
  http://127.0.0.1:8080/admin/api/login \
  -H "Content-Type: application/json" \
  --data "{\"password\":\"${AUTOGRAPHS_ADMIN_PASSWORD}\"}"
```

Create a draft item through the private session boundary:

```bash
curl -fsS http://127.0.0.1:8080/admin/api/items \
  -b /tmp/autographs-admin.cookies \
  -H "Origin: http://127.0.0.1:8080" \
  -H "Content-Type: application/json" \
  --data '{"title":"Signed card","signer":"Example Signer","category":"Trading Card","signerCredits":[{"displayName":"Example Signer","itemRole":"actor"}],"format":"Trading Card","origin":"Official","language":"English","franchises":["Example Franchise"],"tags":["fixture"]}'
```

Upload one private original using the returned item ID:

```bash
curl -fsS "http://127.0.0.1:8080/admin/api/items/${ITEM_ID}/images" \
  -b /tmp/autographs-admin.cookies \
  -H "Origin: http://127.0.0.1:8080" \
  -F "image=@./example.jpg;type=image/jpeg"
```

Update publication status:

```bash
curl -fsS "http://127.0.0.1:8080/admin/api/items/${ITEM_ID}/publication" \
  -b /tmp/autographs-admin.cookies \
  -H "Origin: http://127.0.0.1:8080" \
  -H "Content-Type: application/json" \
  --data '{"publicationStatus":"published"}'
```

Publish and inspect the generated static release:

```bash
curl -fsS http://127.0.0.1:8080/admin/api/publish/incremental \
  -b /tmp/autographs-admin.cookies \
  -H "Origin: http://127.0.0.1:8080" \
  --request POST

curl -fsS http://127.0.0.1:8080/admin/api/publish/status \
  -b /tmp/autographs-admin.cookies \
  -H "Origin: http://127.0.0.1:8080"
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

## Phase 7 Taxonomy Rollout

Use this sequence when applying the Phase 7 metadata taxonomy to live data. The
goal is to review the migration, keep the operator in control of live Oracle
changes, then publish a fresh schema version 2 static release from the runtime
boundary.

1. Generate the taxonomy migration report from the committed
   `.planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json`
   mapping and the current legacy export. Review the `Mapped`, `Needs review`,
   and `Report only` sections before changing live data.
2. Generate the operator-reviewable PL/SQL and read it alongside
   `controller/db/updates/07-02-taxonomy-backfill-review.sql`. Confirm that
   `custom` maps to `Origin: Custom`, `Tr`/`Tra`/`Trading Card` map to
   `Format: Trading Card`, duplicate physical items remain report-only, and no
   private Object Storage identifiers or credentials appear in the artifacts.
3. Optionally run the reviewed PL/SQL manually through SQL Developer against
   live Oracle per D-07-17. Do not treat generated SQL as an automatic deploy
   step; the operator review is part of the safety boundary.
4. Deploy the controller/admin/public code and configuration through the normal
   GitHub/Ansible path after the schema and reviewed data migration are ready.
5. Run a full static publish with `POST /admin/api/publish/full`. A full static
   publish is required after the schema/taxonomy migration so
   `collection.json`, `facets.json`, item detail JSON, and rendered item pages
   all use `schemaVersion: 2`.
6. Verify admin editing through `/admin`: Identity, Classification, and Details
   sections should load migrated data; signer suggestions should show reusable
   profiles; `Possible duplicate signer` warnings and `Merge signer` repair
   should work before save/publish.
7. Verify public facets and detail pages through Caddy: `/data/collection.json`
   and `/data/facets.json` should report `schemaVersion: 2`, facets should
   include signer/franchise/productLine/format/language/origin/role/tag, and no
   category facet should be present.
8. Keep legacy `signer`, `category`, and `autograph_item_tags` fields through
   Phase 7 for rollback/reference. Cleanup is a later planned step after live
   schema version 2 verification, not part of the initial rollout.

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

The Oracle probe uses the same pure-Rust `oracledb` driver and wallet alias as
the deployed controller. It requires `ORACLE_DB_CONNECT_STRING`,
`ORACLE_DB_USER`, the matching Oracle credential, `ORACLE_DB_WALLET_DIR`, and
`ORACLE_DB_WALLET_PASSWORD`; the password decrypts `ewallet.pem` after the
wallet is unpacked. Instance-principal media access requires `OCI_AUTH_MODE`,
`OCI_MEDIA_NAMESPACE`, `OCI_MEDIA_BUCKET_NAME`, and the runtime dynamic-group
policy for the media bucket.

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
ORACLE_DB_PASSWORD=replace-with-runtime-db-password
ORACLE_DB_CONNECT_STRING=autographsdb_medium
ORACLE_DB_WALLET_DIR=/opt/autographs/wallet
ORACLE_DB_WALLET_PASSWORD=replace-with-wallet-download-password
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

The image contains the compiled smoke-test executable and CA certificates. It
does not contain Oracle Instant Client, the Oracle wallet, database credential,
or Object Storage credentials. Oracle connectivity is provided by the
Oracle-maintained pure-Rust `oracledb` crate compiled into the executable.

Rerun the live persistence and static publish smokes after `oracledb` updates,
especially when connection, wallet, catalog encoding, or value-conversion
behavior changes.

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
same controller quadlet, Caddyfile, static volume, and promoted static release on the
VM before running this checkpoint.

For Phase 5 closeout, the final checkpoint was a second credential-gated smoke
that exercised the deployed controller and Caddy preview as black boxes. Rerun
this smoke for future controller or publisher changes that need live
end-to-end proof. It creates a uniquely named draft through `/admin/api/*`,
uploads a valid private image, verifies the Oracle row and OCI Object Storage
object, publishes a static release, and fetches the browse page, item HTML,
item JSON, facets, and generated WebP derivatives through Caddy. It then
unpublishes the item, runs an incremental publish, confirms that stale public
files return `404`, and removes the
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
AUTOGRAPHS_ADMIN_PASSWORD=replace-with-runtime-admin-password
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

### Run a Candidate Controller Before Merge

When a controller persistence driver or runtime dependency changes, do not make
the automatic post-merge deployment its first full repository exercise. Build
and transfer the candidate controller from the exact reviewed commit alongside
the static-publish smoke image:

```bash
CANDIDATE_VERSION="$(git rev-parse --short HEAD)"
CANDIDATE_IMAGE="localhost/autographs-controller-candidate:${CANDIDATE_VERSION}"

docker build \
  --file controller/Dockerfile \
  --tag "${CANDIDATE_IMAGE}" \
  .
docker save \
  --output /tmp/autographs-controller-candidate.tar \
  "${CANDIDATE_IMAGE}"
scp /tmp/autographs-controller-candidate.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-controller-candidate.tar
```

On the VM, start the candidate beside the deployed controller on the private
Podman network. Use copied wallet and secret directories so applying private
SELinux labels cannot relabel the deployed controller's live mounts. The
candidate shares the static volume because the smoke verifies its promoted
release through the existing Caddy preview. Do not run a separate operator
publish concurrently with this gate.

```bash
run_candidate_gate() (
  set -Eeuo pipefail

  CANDIDATE_VERSION="<git-short-sha-used-during-build>"
  CANDIDATE_IMAGE="localhost/autographs-controller-candidate:${CANDIDATE_VERSION}"
  CANDIDATE_NAME="autographs-controller-candidate"
  CANDIDATE_WALLET_DIR="/tmp/autographs-controller-candidate-wallet"
  CANDIDATE_SECRETS_DIR="/tmp/autographs-controller-candidate-secrets"
  SMOKE_IMAGE="localhost/autographs-live-static-publish-smoke:${CANDIDATE_VERSION}"
  SMOKE_WALLET_DIR="/tmp/autographs-smoke-wallet"

  cleanup_candidate_gate() {
    sudo podman rm --force "${CANDIDATE_NAME}" >/dev/null 2>&1 || true
    sudo rm -rf \
      "${CANDIDATE_WALLET_DIR}" \
      "${CANDIDATE_SECRETS_DIR}" \
      "${SMOKE_WALLET_DIR}" || true
  }
  trap cleanup_candidate_gate EXIT INT TERM
  cleanup_candidate_gate

  sudo cp -a /opt/autographs/wallet "${CANDIDATE_WALLET_DIR}"
  sudo cp -a /opt/autographs/secrets "${CANDIDATE_SECRETS_DIR}"
  sudo podman load --input /tmp/autographs-controller-candidate.tar
  sudo podman run --replace --detach \
    --name "${CANDIDATE_NAME}" \
    --network autographs \
    --env-file /opt/autographs/env/app.env \
    --env-file /opt/autographs/env/controller.env \
    --volume "${CANDIDATE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
    --volume "${CANDIDATE_SECRETS_DIR}":/opt/autographs/secrets:ro,Z \
    --volume autographs-static:/var/lib/autographs/static \
    "${CANDIDATE_IMAGE}"

  for attempt in {1..30}; do
    sudo podman logs "${CANDIDATE_NAME}" 2>&1 \
      | grep -q "Oracle catalog schema preflight passed" && break
    sleep 2
  done
  sudo podman logs "${CANDIDATE_NAME}" 2>&1 \
    | grep "Oracle catalog schema preflight passed"

  sudo cp -a /opt/autographs/wallet "${SMOKE_WALLET_DIR}"
  sudo podman load --input /tmp/autographs-live-static-publish-smoke.tar
  sudo podman run --rm \
    --network autographs \
    --env-file /opt/autographs/env/live-persistence-smoke.env \
    --env AUTOGRAPHS_CONTROLLER_BASE_URL="http://${CANDIDATE_NAME}:8080" \
    --volume "${SMOKE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
    "${SMOKE_IMAGE}"
)

run_candidate_gate
```

The gate passes only when the smoke creates and updates the item through the
candidate controller, publishes and verifies the generated static release,
unpublishes it, republishes the removal, and cleans up the Oracle rows and OCI
original. Record the exact candidate commit and complete result on the PR before
merge. The subshell's exit/signal trap removes the candidate container and all
copied wallet/API-key material after success, failure, or interruption.

The static smoke result was recorded for Phase 5 closeout. The public hostname
now serves generated output through Caddy; rerunning the smoke proves that the
deployed Rust/static path can still publish a fresh item end to end and remove
it again. For `oracledb` updates, include non-English catalog metadata in the
smoke evidence so the real controller persistence path continues to prove the
app's UTF-8 catalog behavior.
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

## Phase 6 Public Artifact Size Review

The Phase 6 optimization pass measured the checked-in generated public sample
before changing derivative bounds. Largest current sample artifacts were:

| Artifact | Bytes | Notes |
|----------|-------|-------|
| `controller/static-public/media/ahsoka-tano/image-1-detail-eaa984e2fa19.webp` | 2,615,114 | Lossless WebP detail derivative, previously generated at about 1600x1200. |
| `controller/static-public/media/ahsoka-tano/image-2-detail-66e63732ffc3.webp` | 578,396 | Lossless WebP detail derivative. |
| `controller/static-public/media/ahsoka-tano/image-1-thumbnail-8781173feb10.webp` | 485,934 | Lossless WebP thumbnail derivative. |
| `controller/static-public/assets/site.css` | 17,417 | Largest static text asset. |
| `controller/static-public/assets/browse.js` | 8,092 | Largest public JavaScript asset. |

The active Rust `image` crate WebP encoder is lossless-only in this repository,
so Phase 6 avoided adding a new image encoder dependency. Instead, detail
derivatives are capped to the current public UI need: thumbnails remain bounded
at `480x640`, and detail derivatives are bounded at `960x1280`. A regression
uses the large public sample above and the production `generate_derivative`
function:

```text
detail derivative sample before=2615114 after=1777658 width=960 height=1276
```

That is a 837,456 byte reduction for the large sample detail derivative while
preserving the sanitized `/media/...-detail-<fingerprint>.webp` path contract
and WebP content type. Public artifact privacy scans and manifest byte-size
validation remain mandatory for derivative changes.

## Cache and CDN Verification

After deploy, verify Caddy's origin cache posture from the public hostname:

```bash
curl -I "https://${AUTOGRAPHS_DOMAIN}/media/<item-slug>/<image-slug>-detail-<fingerprint>.webp"
curl -I "https://${AUTOGRAPHS_DOMAIN}/data/collection.json"
curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/"
curl -I "https://${AUTOGRAPHS_DOMAIN}/admin/api/health"
```

Expected `Cache-Control` behavior:

- Public `/media/*`: `public, max-age=86400`.
- Public `/assets/*`: `public, max-age=3600`.
- Public HTML, JSON, and `manifest.json`: `public, max-age=60, must-revalidate`.
- `/admin`, `/admin/*`, and `/admin/api/*`: `no-store`.

If Cloudflare or another CDN is enabled later, keep admin shell/API routes out
of CDN caching, and preserve rollback by keeping HTML/JSON short-lived. Routine
image replacement publishes a new fingerprinted `/media/*` URL; reserve CDN
purges for emergency takedown, accidental public exposure, or CDN incident
response. See `docs/dns-runbook.md` for the deferred Cloudflare checklist and
purge guidance.

## Phase 8 CDN/cache contract

The Phase 8 CDN/cache rule, purge, rollback, and production verification
contract is maintained in [cdn-cache-contract.md](cdn-cache-contract.md). Keep
the Caddy origin headers above aligned with that contract before implementing
or verifying admin image adjustment cache behavior.

## Phase 6 Admin Live Smoke

Use this operator-run smoke when an admin workflow, publisher, retention, or
cleanup change needs live Oracle/Object Storage proof. Local and CI runs only
prove that the ignored smoke remains gated by
`AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`.

1. Log in through `/admin/api/login` and save the HTTP-only session cookie.
2. Create or edit a private smoke item through `/admin/api/items`.
3. Upload at least one private original, then replace or remove one image when
   validating cleanup behavior.
4. Read item history and confirm metadata, image, cleanup, and publication
   events are visible without private Object Storage details.
5. Confirm pending-change status reflects saved private edits before publish.
6. Run incremental publish, then verify `/collection/`, item HTML, item JSON,
   facets, and generated WebP derivatives through the static Caddy preview.
7. Confirm publish status reports promoted-release and failed-candidate
   retention counts without exposing filesystem internals.
8. Unpublish or delete the smoke item, publish again, and verify stale public
   item pages and media derivatives no longer resolve.
9. Check Oracle rows and Object Storage objects for cleanup residue. Use the
   live persistence cleanup mode only for interrupted smoke residue.

The bundled ignored test implements the same black-box shape:

```bash
AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true \
  cargo test --manifest-path controller/Cargo.toml \
  --features live-persistence live_static_publish_smoke -- --ignored --nocapture
```

Do not record this as passed unless it ran with real Oracle, private Object
Storage, deployed controller, Caddy static preview, and a runtime admin password.

## Full Rebuild
