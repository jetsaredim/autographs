---
spike: 001
name: oracledb-container-smoke
type: standard
validates: "Given the existing ADB wallet and a locally built image copied to the VM, when explicit read-only then gated temporary-write probes run, then oracledb can connect, bind, commit, query, and clean up against the live catalog without Oracle Instant Client."
verdict: VALIDATED
related: []
tags: [oracle, rust, container, smoke, adb]
---

# Spike 001: Oracledb Container Smoke

## What This Validates

The new Oracle-maintained `oracledb` thin driver can use the deployed ADB
wallet, perform the catalog operations modeled by the existing persistence
smoke, and run in a minimal container that has no Oracle Instant Client.

The controller's `oracle` dependency and production image stay unchanged during
this experiment. A successful live write smoke is evidence for a separate
controller migration; it is not itself a production-driver replacement.

## Research

| Approach | Tool | Pros | Cons | Status |
|---|---|---|---|---|
| Current native path | `oracle 0.6.3` | Proven controller and smoke behavior | Requires Instant Client and ODPI-C | Baseline |
| New thin path | `oracledb 26.0.0-beta.2` | Oracle-maintained, pure Rust, reads TNS aliases and mTLS wallet PEM files | Pre-release API; must explicitly set config and wallet paths | Chosen |

The [driver guide](https://docs.rs/oracledb/latest/oracledb/guide/index.html)
documents `Config::set_config_dir`, `Config::set_wallet_location`, and
`Config::set_wallet_password` for ADB mTLS. It also documents positional binds,
explicit commits, and rollbacks. The [official repository](https://github.com/oracle/rust-oracledb)
labels the crate as a pre-release, pure-Rust driver; treat this experiment as a
compatibility and operational test rather than an automatic upgrade.

## How to Run

Build and export on a trusted Linux `amd64` workstation:

```bash
SPIKE_VERSION="$(git rev-parse --short HEAD)"
SPIKE_IMAGE="localhost/autographs-oracledb-spike:${SPIKE_VERSION}"

docker build \
  --file .planning/spikes/001-oracledb-container-smoke/Dockerfile \
  --tag "${SPIKE_IMAGE}" \
  .planning/spikes/001-oracledb-container-smoke
docker save --output /tmp/autographs-oracledb-spike.tar "${SPIKE_IMAGE}"
scp /tmp/autographs-oracledb-spike.tar \
  opc@"${VM_PUBLIC_IP}":/tmp/autographs-oracledb-spike.tar
```

On the VM, create `/opt/autographs/env/oracledb-spike.env` with mode `0600`.
It must contain the existing `ORACLE_DB_USER`, `ORACLE_DB_PASSWORD`,
`ORACLE_DB_CONNECT_STRING`, `ORACLE_DB_WALLET_DIR=/opt/autographs/wallet`, and
the optional `ORACLE_DB_WALLET_PASSWORD`. Do not commit it.

Run the read-only gate first:

```bash
SPIKE_VERSION="<git-short-sha-used-during-build>"
SPIKE_IMAGE="localhost/autographs-oracledb-spike:${SPIKE_VERSION}"
SPIKE_WALLET_DIR="/tmp/autographs-oracledb-spike-wallet"

sudo cp -a /opt/autographs/wallet "${SPIKE_WALLET_DIR}"
sudo podman load --input /tmp/autographs-oracledb-spike.tar
sudo podman run --rm \
  --env-file /opt/autographs/env/oracledb-spike.env \
  --env AUTOGRAPHS_ORACLEDB_SPIKE_READ_ONLY=true \
  --volume "${SPIKE_WALLET_DIR}":/opt/autographs/wallet:ro,Z \
  "${SPIKE_IMAGE}"
```

Only after it reports `"schema":"ready"`, add
`AUTOGRAPHS_ORACLEDB_SPIKE_WRITE_SMOKE=true` to the run command. That phase
creates, reads, then deletes a draft item plus a fake-image metadata row. It
does not upload an Object Storage object. The direct-driver proof intentionally
isolates database behavior; the existing OCI media smoke remains the proof for
Object Storage and instance-principal access.

For a terminated write run, supply the printed item ID through the protected
environment file as `AUTOGRAPHS_ORACLEDB_SPIKE_CLEANUP_ITEM_ID=<uuid>` and rerun
the same image. Cleanup deletes the temporary child rows and item, commits, and
checks that no item row remains.

## What to Expect

- The runtime image contains only the binary and CA certificates: no Instant Client package or wallet.
- Read-only mode emits a single redacted JSON record with the crate version, schema-preflight status, and a `0` or `1` representative-read result.
- Write mode emits `created-verified-cleaned`; it never prints an item ID, object key, database user, password, connect string, wallet location, or catalog values.
- Any failure exits non-zero with an operation category but no secret value.

## Investigation Trail

- 2026-08-20: Scoped the spike to a hand-carried VM image, mirroring `Dockerfile.smoke` rather than modifying the deploy path.
- 2026-08-20: Added a read-only gate before any DML. The write gate models the existing smoke's `autograph_items` and `autograph_images` insert/read/cleanup sequence with fake image metadata only.
- 2026-08-20: `cargo fmt --check` and `cargo check` passed with `oracledb 26.0.0-beta.2`.
- 2026-08-20: The local container image built successfully (`45,262,489` bytes) and returned its expected skipped JSON record with no gate set. Its RPM inventory contains no Oracle Instant Client package.
- 2026-08-20: The OCI VM read-only gate passed with `{"mode":"read-only","driver":"oracledb","driver_version":"26.0.0-beta.2","schema":"ready","representative_read":1}`.
- 2026-08-20: The OCI VM write gate passed with `{"mode":"write-smoke","status":"created-verified-cleaned"}`. The driver created, committed, read, and removed temporary item and image-metadata rows.

## Results

**VALIDATED.** Local preflight, the VM read-only verification, and the gated
write/cleanup verification all passed. The `oracledb` image was approximately
45 MB and contains no Oracle Instant Client.

## Decision Rule

- **VALIDATED:** local image build succeeds, the VM read-only gate succeeds, the gated write smoke returns `created-verified-cleaned`, and cleanup/recovery is confirmed. **Met on 2026-08-20.** Migrate the controller from `oracle` and remove Instant Client from its image, then rerun both established persistence smokes.
- **PARTIAL or INVALIDATED:** record the exact redacted failure and environment-independent reproduction steps here; retain the current controller driver and image unchanged.
