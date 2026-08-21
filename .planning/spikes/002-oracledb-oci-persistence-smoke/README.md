---
spike: 002
name: oracledb-oci-persistence-smoke
type: standard
validates: "Given the same VM, wallet, instance principal, bucket, and 16-byte payload as the existing persistence smoke, when oracledb drives the database lifecycle and the existing OCI adapter handles the original, then the item and object both round-trip and are deleted with per-phase timings."
verdict: PENDING
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

The test prints phase timings, but one execution is not a benchmark. Compare
multiple alternating runs of this image and `Dockerfile.smoke` on the same VM;
report medians and retain any failure output.

## How to Run

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

Use the existing protected live-persistence smoke environment file, which
already supplies the Oracle wallet/database variables and OCI instance-principal
media coordinates. On the VM:

```bash
SPIKE_VERSION="<git-short-sha-used-during-build>"
SPIKE_IMAGE="localhost/autographs-live-oracledb-persistence-smoke:${SPIKE_VERSION}"
SPIKE_WALLET_DIR="/tmp/autographs-oracledb-persistence-wallet"

sudo cp -a /opt/autographs/wallet "${SPIKE_WALLET_DIR}"
sudo podman load --input /tmp/autographs-live-oracledb-persistence-smoke.tar
sudo podman run --rm \
  --env-file /opt/autographs/env/live-persistence-smoke.env \
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
- Pending: run the one-shot image on the OCI VM and compare several alternating runs against the established persistence smoke.

## Results

Pending VM execution.

## Decision Rule

- **VALIDATED:** the image runs on the VM, completes database plus Object Storage round-trip and cleanup, and repeated timing samples reveal no concerning operational regression. Begin the controller driver migration.
- **PARTIAL or INVALIDATED:** document the exact failure or regression; keep `oracle` and Instant Client in production.
