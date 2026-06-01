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

The pure-Rust Oracle probe requires `AUTOGRAPHS_ORACLE_HOST`,
`AUTOGRAPHS_ORACLE_PORT`, and `AUTOGRAPHS_ORACLE_SERVICE_NAME` alongside the
existing wallet, user, and password variables. OCI S3 compatibility requires
`OCI_S3_ENDPOINT`, `OCI_S3_ACCESS_KEY`, `OCI_S3_SECRET_KEY`,
`OCI_MEDIA_NAMESPACE`, and `OCI_MEDIA_BUCKET_NAME`.
