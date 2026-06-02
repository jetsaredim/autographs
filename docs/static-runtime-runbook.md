# Static Runtime Foundation Runbook

## Local Controller Seed Path

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

podman load --input /tmp/autographs-live-persistence-smoke.tar
podman run --rm \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --volume /opt/autographs/wallet:/opt/autographs/wallet:ro \
  localhost/autographs-live-persistence-smoke:phase-05
```

The image contains the compiled smoke-test executable, CA certificates, and
Oracle Instant Client. It does not contain the Oracle wallet, database password,
or OCI Customer Secret credentials.
