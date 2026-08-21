---
spike: 002
name: oracledb-oci-persistence-smoke
type: standard
validates: "Given the same VM, wallet, instance principal, bucket, and 16-byte payload as the existing persistence smoke, when oracledb drives the database lifecycle and the existing OCI adapter handles the original, then the item and object both round-trip and are deleted with per-phase timings."
verdict: VALIDATED
related: [001-oracledb-container-smoke]
tags: [oracle, rust, oci, container, smoke, adb]
---

# Spike 002: Oracledb OCI Persistence Smoke

## What This Validates

This is the closer comparison gate before production migration. It retains the
current `OciInstancePrincipalMediaStore`, UUID-only original-object key, 16-byte
test payload, private object upload/read/delete, database child-first cleanup,
and recovery pattern from `live_persistence_smoke`. Only the database driver is
different: it uses `oracledb` rather than `oracle`.

The temporary test printed phase timings, but one execution was not treated as
a benchmark. Its original comparison method was to alternate this image and
the then-current `Dockerfile.smoke` on the same VM and report medians.

## Historical Execution Record

The commands below record exactly how Spike 002 was run and are intentionally
not runnable after the production migration. PR #211 removed the temporary
`Dockerfile.oracledb-smoke`, `oracledb-live-smoke` feature, and
`live_oracledb_persistence_smoke` test when the validated driver became the
production implementation. Current operator runs must use the surviving
`controller/Dockerfile.smoke`, `live-persistence` feature, and procedures in
`docs/static-runtime-runbook.md`.

Build and transfer the one-shot image from a trusted Linux `amd64` workstation:

```bash
SPIKE_VERSION="$(git rev-parse --short HEAD)"
SPIKE_IMAGE="localhost/autographs-live-oracledb-persistence-smoke:${SPIKE_VERSION}"

docker build \
  --file controller/Dockerfile.oracledb-smoke \
  --tag "${SPIKE_IMAGE}" \
  .
docker save --output /tmp/autographs-live-oracledb-persistence-smoke.tar "${SPIKE_IMAGE}"
scp /tmp/autographs-live-oracledb-persistence-smoke.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-live-oracledb-persistence-smoke.tar
```

Layer the existing protected live-persistence smoke environment file, which
supplies the OCI instance-principal media coordinates, with the protected
`oracledb-spike.env` file from Spike 001, which supplies the thin driver's
verified Oracle wallet/database settings. On the VM:

```bash
SPIKE_VERSION="<git-short-sha-used-during-build>"
SPIKE_IMAGE="localhost/autographs-live-oracledb-persistence-smoke:${SPIKE_VERSION}"
SPIKE_WALLET_DIR="/tmp/autographs-oracledb-persistence-wallet"

sudo cp -a /opt/autographs/wallet "${SPIKE_WALLET_DIR}"
sudo podman load --input /tmp/autographs-live-oracledb-persistence-smoke.tar
sudo podman run --rm \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
  --env-file /opt/autographs/env/oracledb-spike.env \
  --env AUTOGRAPHS_LIVE_ORACLEDB_PERSISTENCE_SMOKE=true \
  --volume "${SPIKE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
  "${SPIKE_IMAGE}"
```

Expected output includes a temporary item ID/object key, a line beginning
`oracledb persistence timings:`, and a `cleanup_ms=` line. The process exits
zero only after it reads the private object and both Oracle records, then
removes the object and all temporary rows.

For a terminated run, set the corresponding comma-separated values in the
protected environment file and rerun the same image:

```text
AUTOGRAPHS_LIVE_ORACLEDB_PERSISTENCE_CLEANUP_ITEM_IDS=<item-uuid>
AUTOGRAPHS_LIVE_ORACLEDB_PERSISTENCE_CLEANUP_OBJECT_KEYS=originals/<item-uuid>/<image-uuid>
```

## Investigation Trail

- 2026-08-20: Spike 001 validated the direct ADB read/write/cleanup lifecycle with the pure-Rust driver, but intentionally did not use OCI Object Storage.
- 2026-08-20: Added a controller integration smoke with `oracledb-live-smoke`, which enables the existing OCI media adapter without enabling the current `oracle` driver feature.
- 2026-08-20: `cargo test --features oracledb-live-smoke --test live_oracledb_persistence_smoke --no-run` passed locally.
- 2026-08-20: The `Dockerfile.oracledb-smoke` image built locally (`46,865,671` bytes) and safely skipped without its explicit live gate. `cargo check --features production-persistence` also passed after the feature split.
- 2026-08-20: The first VM attempt failed before connecting or mutating data because `live-persistence-smoke.env` did not supply the valid `ORACLE_DB_WALLET_PASSWORD` used by Spike 001. Updated the runbook to layer the known-good `oracledb-spike.env` over the OCI smoke environment and changed future builds to fail immediately when the wallet password is absent.
- 2026-08-20: After supplying the verified thin-driver wallet settings plus the OCI media settings in `oracledb-spike.env`, the VM smoke completed successfully in `2.07s`: `connect_ms=268`, `item_ms=3`, `upload_ms=1408`, `image_ms=4`, `verify_ms=57`, and `cleanup_ms=180`.

## Results

**VALIDATED.** The live VM smoke created and committed a temporary autograph
item, uploaded the 16-byte private original through the existing OCI
instance-principal adapter, committed its image metadata, read all three pieces
back, then deleted the Oracle rows and OCI object and verified cleanup. The test
exited zero in `2.07s`.

## Decision Rule

- **VALIDATED:** the image runs on the VM and completes the database plus Object Storage round-trip and cleanup without a concerning operational result. **Met on 2026-08-20.** Begin the controller driver migration; use repeated alternating runs only if comparative performance numbers are needed.
- **PARTIAL or INVALIDATED:** document the exact failure or regression; keep `oracle` and Instant Client in production.
