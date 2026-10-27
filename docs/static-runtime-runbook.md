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
Object Storage objects. Prefer the printed item ID: cleanup discovers current
image keys and cleanup-event targets from Oracle before deleting any rows. Set
one or both cleanup variables; values can be comma or newline separated:

```text
AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_ITEM_IDS=3f14e408-d4a7-4ef7-91fe-4ec10b3ea745
AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_OBJECT_KEYS=<private-candidate-key-from-protected-inventory>
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

Cleanup mode runs before the normal smoke gate. It first inventories current
`autograph_images.object_key` and cleanup-event targets, merges them with any
privately supplied candidates, deletes and verifies every Object Storage
object through instance-principal auth, and only then deletes the Oracle item
and cascading discovery rows. If inventory, deletion, or absence verification
cannot be proven, it retains the Oracle rows for a retry. Candidate-key input
is private data that the tooling consumes but never echoes; captured output
contains item IDs, counts, safe status categories, and domain-separated object
fingerprints only. Remove the cleanup variables from the env file before
running the normal smoke again.

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
Podman network. Use a copied wallet directory so applying a private SELinux
label cannot relabel the deployed controller's live mount. The candidate uses
the VM instance principal for OCI media access and does not receive a deploy API
key or secrets-directory mount. It shares the static volume because the smoke
verifies its promoted release through the existing Caddy preview. Do not run a
separate operator publish concurrently with this gate.

```bash
run_candidate_gate() (
  set -Eeuo pipefail

  CANDIDATE_VERSION="<git-short-sha-used-during-build>"
  CANDIDATE_IMAGE="localhost/autographs-controller-candidate:${CANDIDATE_VERSION}"
  CANDIDATE_NAME="autographs-controller-candidate"
  CANDIDATE_WALLET_DIR="/tmp/autographs-controller-candidate-wallet"
  SMOKE_IMAGE="localhost/autographs-live-static-publish-smoke:${CANDIDATE_VERSION}"
  SMOKE_WALLET_DIR="/tmp/autographs-smoke-wallet"

  cleanup_candidate_gate() {
    sudo podman rm --force "${CANDIDATE_NAME}" >/dev/null 2>&1 || true
    sudo rm -rf \
      "${CANDIDATE_WALLET_DIR}" \
      "${SMOKE_WALLET_DIR}" || true
  }
  trap cleanup_candidate_gate EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  cleanup_candidate_gate

  sudo cp -a /opt/autographs/wallet "${CANDIDATE_WALLET_DIR}"
  sudo podman load --input /tmp/autographs-controller-candidate.tar
  sudo podman run --replace --detach \
    --name "${CANDIDATE_NAME}" \
    --network autographs \
    --env-file /opt/autographs/env/app.env \
    --env-file /opt/autographs/env/controller.env \
    --volume "${CANDIDATE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
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
operator-maintenance path, and use protected operator-side inventory for any
residual private object. If the static smoke passes but reports a timeout while
deleting the private original, use the live persistence smoke cleanup mode with
the logged item ID. Let cleanup discover current and pending candidates from
Oracle; supply an additional key only through the protected environment when
inventory proves it is not represented there. The cleanup transcript reports
object fingerprints, never the private candidate itself.

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
image replacement publishes a new fingerprinted `/media/*` URL. Caddy serves a
shared media namespace with an active `current/media` fallback. The publisher
copies only active media before pointer promotion; the atomic pointer change
commits candidate visibility, and shared copy follows it. A postcommit copy
failure leaves the new release active with a cleanup warning. Repair its shared
copy before a manual rollback. Confirm old and new URLs return 200 after an adjustment and rollback,
including an origin cache miss. A routine unpublish removes page and JSON but
keeps known old media URLs for at least 24 hours and while retained releases
reference them. For urgent privacy takedown, publish the unpublish when
possible, create a durable exact URL marker, prove dual-origin 404, then remove
copies and purge the exact CDN URL. See [cdn-cache-contract.md](cdn-cache-contract.md) for the
retention contract and `docs/dns-runbook.md` for the Cloudflare checklist.

### Phase 8 publisher and Caddy rollout freeze

Run `bash scripts/verify-caddy-media-origin.sh`, the publisher/Caddy contract
tests, and the reviewed Ansible syntax/lint checks before touching the VM.
Back up the installed host Caddyfile and freeze publication **before** replacing
either the running Caddy or controller process. Use the deployed host path and
edit its inode in place: the current quadlet bind-mounts that single file, so
an atomic host rename could leave the running container reading an old inode.
The two exact paths are `/admin/api/publish/full` and
`/admin/api/publish/incremental`; `/admin/api/publish/status` stays proxied.

```bash
CADDYFILE=/opt/autographs/caddy/Caddyfile
sudo cp -p "$CADDYFILE" "$CADDYFILE.pre-media-freeze.$(date -u +%Y%m%dT%H%M%SZ)"
sudo python3 - "$CADDYFILE" <<'PY'
import sys

needle = "\thandle /admin/api/* {"
gate = '''\thandle /admin/api/publish/full {
\t\theader Cache-Control "no-store"
\t\trespond "Publishing temporarily unavailable" 503
\t}
\thandle /admin/api/publish/incremental {
\t\theader Cache-Control "no-store"
\t\trespond "Publishing temporarily unavailable" 503
\t}
\n'''
with open(sys.argv[1], "r+", encoding="utf-8") as caddyfile:
    content = caddyfile.read()
    if content.count(needle) != 1 or "AUTOGRAPHS PUBLISH FREEZE" in content:
        raise SystemExit("Caddyfile shape or freeze state differs; stop rollout")
    caddyfile.seek(0)
    caddyfile.write(content.replace(needle, gate + needle, 1))
    caddyfile.truncate()
PY
sudo podman exec autographs-caddy caddy validate --config /etc/caddy/Caddyfile
sudo podman exec autographs-caddy caddy reload --config /etc/caddy/Caddyfile
```

Through the public admin origin, check that unauthenticated `POST` requests to
both exact publish paths return `503` and `Cache-Control: no-store`; check an
authenticated GET of `/admin/api/publish/status` still works and public current
pages/media still return `200`. Keep session cookies in a protected file and do
not print them. Stop automated smoke/publish jobs and direct private-network
calls to `autographs-controller:8080`; gate any other publisher ingress or abort.
The route gate blocks new ingress; publish status alone cannot prove that an
already accepted request is not waiting on the old controller's mutex. Record
the unchanged `current` pointer, then merge the Release PR or dispatch
`workflow_dispatch operation=retry`. The production workflow explicitly passes
`autographs_deploy_publish_frozen=true` and runs
`publish-freeze-preflight.yml` before the full deploy. That preflight keeps the
old gated Caddyfile, removes Caddy's controller service dependency, restarts
Caddy under the old gate, and checks publish `503` plus current static `200`.
It then stops the old controller and waits for its unit, captured container,
and PID to exit before any new shared-media route can restart. A surviving
process or lost static response stops the deploy with publishing frozen.

For the **first** cutover, preflight runs a read-only inventory before and
after verified old-controller exit; the latter is authoritative. A retained
release needs its manifest, successful production publish-job evidence or a
trusted promotion receipt, and verified shared WebP bytes. A pruned release
needs a trusted archived manifest or promotion receipt naming the exact path;
an Oracle success row alone cannot prove its media. Supply reviewed evidence
in a root-protected JSON file outside the static volume using
`autographs_legacy_media_evidence_path`. The current helper accepts only
`archived_receipts` keyed by release ID, each with `promotion_receipt_path`,
`promotion_receipt_sha256`, `manifest_sha256`, and, for pruned releases,
`manifest_path` plus `media_sha256`. The receipt file must bind release ID,
manifest digest, and successful pointer promotion. A bare success-ID list is
never proof. Verify the archive source and digest independently before using
it; do not manufacture an operator export or a promotion receipt. A direct
Oracle job-row verification adapter is not installed in this deployment path,
so hosts without trusted archived receipts remain at the frozen checkpoint.

If a fresh shared path could belong to a pruned committed release and no
archive exists, the report blocks cutover **without moving shared bytes**.
Keep the old frozen current-only route and controller stopped. That old route
may already return 404 for the pruned URL; deferral makes no new availability
claim. The protected `media-legacy-wait.json` records exit time, boot ID,
controller invocation ID, and pointer. With synchronized UTC, retry after exit
time plus 86,400 seconds media freshness, 60 seconds document freshness, and
300 seconds clock margin. A reboot, controller restart, or pointer change starts
a new window. A retained path without independent commit proof remains blocked
past the timer until evidence or legitimate release retirement. Never use a
tombstone to waive ordinary uncertainty. The mutation pass quarantines only
unprotected paths outside the static volume and writes versioned
`.media-commit-history/history.json` plus a migration marker. Back up history,
pointer, releases, shared media, and `.media-tombstones` together.

On **later** deployments, preflight and full-role reaudit use finalized
publication receipts, not retained manifests or mtimes, to keep A's shared URL
after B is published. `current` is the commit witness; a durable intent enables
recovery if the controller stops on either side of pointer replacement. Exact
markers allow intentionally missing active or retained bytes and continue
denying both origins. Unmarked missing or corrupt committed bytes stop the
deployment; repair from a verified committed copy and retry frozen. The freeze
record changes only after successful audit and binds pointer, manifest,
history digest/version, marker inventory, and a versioned active origin probe.
The audit verifies every unmarked current WebP against manifest size, filename
fingerprint, and committed hash before selecting one media URL. If no unmarked
current image exists, including an empty release or one whose only image was
exactly marked and deleted, it selects `/manifest.json` instead. Both public
HTTPS and preview origins must return 200 for that selected path and 404 for
every exact marker; a missing or corrupt unmarked image never falls back to
the document. A missing history after migration
is fatal. Do not bypass a failed preflight with an unfrozen variable.

The full role requires the protected freeze record and rechecks old PID and
container absence before copying Caddyfile. It starts and verifies the reviewed
controller image, source, and health before restarting Caddy with managed
exact-path `503` routes. A failed new Caddy route restores the saved old frozen
Caddyfile; any failed step keeps the freeze record and does not claim release
completion. Retry repeats the preflight and explicit frozen deploy. A fresh VM
records bootstrap-frozen state, but first-release seeding still requires a
valid `current` release before the existing full-deploy static gate can pass.

Only after production health and the host's public and preview origin checks
pass does the workflow call `publish-unfreeze.yml` separately. It repeats the
image/release/source, current static, frozen POST, Caddy validation, and
disposable marker checks. Verify-only and mutating unfreeze consume the same
audited media-or-document probe, compare pointer/manifest/history/marker
digests against the freeze record, and prove persistent exact marker denial at
both origins. The mutating call keeps its disposable marker through the Caddy
restart, removes only the managed gate, and repeats selected-path 200 and
marked-path 404 afterward. It also
requires unauthenticated publish POSTs to reach `401`/`403` without starting a
job before deleting the freeze record. A failed postcheck restores the frozen
file and retains the record. The
release completion gate requires this unfreeze step to succeed. Record gate-on,
old-process exit, quarantine audit, new-controller readiness, route readiness,
and gate-off times and commands in Wave 8 evidence. Resume credentialed smoke
only after explicit unfreeze. Controller-only rollback to a digest that has not
passed this pointer-first readiness gate is refused while shared-media routing
is active.

For **static release rollback**, keep the publish gate, stop the controller,
and verify its unit, container, and PID exited. Run the protected
`static_release_rollback.py --static-root <static-volume-mount>
--target-release-id <committed-release-id> --frozen-exit-proved` helper as root.
It checks receipts, manifest, WebP hashes, and markers, copies only verified
unmarked media, journals the transition, and repoints `current` atomically.
A direct `ln -s`/`mv` repoint is unrecorded and blocks the next publish or
audit. Restart only the reviewed controller, verify active and old URL origin
misses on public HTTPS and preview `:8081` while frozen, then explicitly
unfreeze.

### Exact emergency media takedown

This is a one-way operator checkpoint. Decide the exact disposable or real URL
before beginning; do not create a marker as part of normal smoke cleanup. A
marker and CDN purge intentionally break old documents. Publish an unpublish
first if possible. On the VM, identify the named static volume and compare both
container mounts before any mutation:

```bash
STATIC_ROOT="$(sudo podman volume inspect autographs-static --format '{{.Mountpoint}}')"
STATIC_ROOT="$(realpath -e "$STATIC_ROOT")"
sudo podman inspect --format '{{range .Mounts}}{{println .Name .Source .Destination .RW}}{{end}}' autographs-controller
sudo podman inspect --format '{{range .Mounts}}{{println .Name .Source .Destination .RW}}{{end}}' autographs-caddy
```

Require the `autographs-static` mount source to resolve to `STATIC_ROOT` at
`/var/lib/autographs/static` with `RW=true` for the controller and at
`/srv/autographs/static` with `RW=false` for Caddy. Abort if any name, source,
destination, or writeability differs. Reconcile historically removed URLs into
markers before the first publish with the revised controller. Validate the
exact URL before creating its marker:

```bash
MEDIA_URL='/media/<item-slug>/<image-slug>-detail-<16-hex-fingerprint>.webp'
[[ "$MEDIA_URL" =~ ^/media/[a-z0-9-]+/[a-z0-9-]+-(thumbnail|detail)-[0-9a-f]{16}\.webp$ ]] || exit 1
sudo install -D -m 0644 /dev/null "$STATIC_ROOT/.media-tombstones/${MEDIA_URL#/}"
sudo podman exec autographs-caddy test -f "/srv/autographs/static/.media-tombstones/${MEDIA_URL#/}"
curl --silent --output /dev/null --write-out '%{http_code}\n' \
  --resolve "${AUTOGRAPHS_DOMAIN}:443:127.0.0.1" "https://${AUTOGRAPHS_DOMAIN}${MEDIA_URL}"
curl --silent --output /dev/null --write-out '%{http_code}\n' \
  "http://127.0.0.1:8081${MEDIA_URL}"
```

The marker readback and **both** origin `404` responses are the effective
origin-denial evidence. If either fails, stop and repair the mount identity,
visibility, or routing before deleting files or purging CDN. Once both pass,
remove the exact relative media path from shared media and every retained
release, including the active release if necessary; a failed deletion can be
retried while the marker continues denying it. Purge only that exact Cloudflare
URL and verify edge `404`. A failed CDN purge leaves cached copies until purge
or TTL expiry and is an incomplete takedown. Keep and back up markers with the
static volume. Never clear one to make rollback work; reinstatement needs an
explicit decision and new media fingerprint.

## Phase 8 CDN/cache contract

The Phase 8 CDN/cache rule, purge, rollback, and production verification
contract is maintained in [cdn-cache-contract.md](cdn-cache-contract.md). Keep
the Caddy origin headers above aligned with that contract before implementing
or verifying admin image adjustment cache behavior.

### Phase 8 adjusted media publish verification

Run the credential-gated static smoke against a deployed candidate controller
and Caddy static preview before enabling Cloudflare proxying. Complete the
Oracle adjustment schema migration and confirm the candidate logs its Oracle
schema preflight pass. Use the candidate controller and one-shot smoke image
procedure above with a copied Oracle wallet, a private Podman network, and the
VM instance principal for the disposable private Object Storage original.
Supply the protected environment values `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`,
`AUTOGRAPHS_CONTROLLER_BASE_URL`, `AUTOGRAPHS_STATIC_PREVIEW_BASE_URL`,
`AUTOGRAPHS_STATIC_RELEASE_ROOT`, `AUTOGRAPHS_ADMIN_PASSWORD`,
`AUTOGRAPHS_PUBLIC_ORIGIN` (or `AUTOGRAPHS_DOMAIN`), `ORACLE_DB_USER`,
`ORACLE_DB_PASSWORD`, `ORACLE_DB_CONNECT_STRING`, `OCI_MEDIA_NAMESPACE`, and
`OCI_MEDIA_BUCKET_NAME`. The smoke logs in for a session cookie and sends the
matching Origin for admin mutations; it must not print the cookie or password.

Run the exact gated target in the one-shot image or a trusted environment with
the same Oracle wallet, controller/Caddy reachability, and instance-principal
media permissions:

```bash
AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true \
  cargo test --manifest-path controller/Cargo.toml \
  --features live-persistence --test live_static_publish_smoke \
  -- --ignored --nocapture
```

Record the candidate commit, successful schema preflight, and complete test
result. The smoke creates a unique item, saves a non-identity crop using the
review response's `mediaRevision`, and proves the private preview changes while
the current public JSON and WebP URLs stay unchanged. After incremental publish
it reports both old and new thumbnail/detail URLs, checks each new URL against
the SHA-256 fingerprint of its final WebP bytes, verifies collection JSON and
detail HTML/JSON references, compares corrected pixels and dimensions with the
private preview, checks the OCI original bytes, and scans for private fields.
It then drafts the temporary item, republishes removal, and deletes the OCI
object and Oracle rows. Its already published WebP URLs remain in the shared
origin namespace until retention cleanup; this is expected after routine
unpublish. If interrupted, use the existing cleanup guard and
live persistence cleanup mode above, then verify no temporary item or private
object remains; restore the prior release pointer if a candidate publish was
not fully validated.

With `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` unset, the ignored local run only
proves compilation and safe gating. It is not live evidence. The new media URLs
make routine Cloudflare purges unnecessary under
[cdn-cache-contract.md](cdn-cache-contract.md). Wave 8 records the real edge
cache observations before enabling the CDN, as required by D-08-19.

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
   item pages and JSON no longer resolve. Recently published media URLs remain
   reachable for cached documents until retention cleanup.
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

### Private media-revision live smoke

Run this only against a disposable private draft in the live-smoke tenancy. It
requires both opt-ins; the second flag alone must never exercise Oracle or OCI:

```bash
AUTOGRAPHS_LIVE_PERSISTENCE_SMOKE=true \
AUTOGRAPHS_LIVE_MEDIA_REVISION_SMOKE=true \
  cargo test --manifest-path controller/Cargo.toml \
  --features live-persistence --test live_persistence_smoke \
  -- --ignored --nocapture
```

Prerequisites are the normal Oracle wallet/database and OCI instance-principal
media settings, a bucket intended for disposable private smoke data, and an
operator who can immediately perform the cleanup recovery below. The smoke
creates a uniquely identified draft, writes small known A and B payloads with
checksums held locally, obtains revision A, and replaces A with B. It proves
stale-A source, draft preview, assist, Save, and Reset requests all return the
typed redacted conflict, reloads revision B, proves current-B Reset succeeds,
reads B through the provider boundary, and then verifies that both objects plus
all Oracle item, image, and edit-event rows are absent. It never publishes the
fixture.

Expected evidence contains the item UUID, `stale_save=409`,
`current_reset=200`, and final cleanup completion. It must not contain object
keys, media checksums, credentials, or wallet material. This production-safe
mode does not inject delete ambiguity, warning persistence failure, transport
failure, or partial provider outage; those remain deterministic CI coverage.

Abort immediately on an unexpected non-conflict result, unreadable B object,
privacy-bearing output, or cleanup error. Do not publish or retry mutations on
the fixture. If interrupted, copy only the printed item UUID and run the
existing `AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_ITEM_IDS` cleanup mode. It
discovers current image and cleanup-event keys before deleting Oracle rows. If
protected operator-side inventory identifies an additional candidate, supply
it through `AUTOGRAPHS_LIVE_PERSISTENCE_CLEANUP_OBJECT_KEYS`; the tool consumes
but never echoes that value. Verify zero Oracle rows, object absence, and only
fingerprint-based cleanup evidence before closing the recovery record; never
paste private keys or checksums into an issue, PR, or CI log.

## Full Rebuild
