# Spike Manifest

## Current Idea

Audit and simplify the Autographs controller ecosystem before Phase 8 Wave 5.
Inventory repository and VM cruft, consolidate environment/configuration
contracts, move persistent plaintext secrets toward OCI Vault, assess Rust and
SQL consistency with measured efficiency evidence, and produce enforceable
standards plus a prioritized cleanup plan.

## Prior Validated Idea

Validate Oracle's new pure-Rust `oracledb` driver against the existing Oracle
Autonomous Database through a hand-carried, one-shot container. The controller
continues to use `oracle` until this spike has a recorded live verdict.

## Requirements

### Ecosystem Cleanup

- Inventory repository, VM, Podman, systemd, deployment, OCI, and GitHub
  configuration surfaces before recommending deletion or consolidation.
- Inventory outputs may contain variable names, paths, permissions, ownership,
  and resource names, but must never collect secret values.
- Persistent production files may retain non-secret configuration; passwords,
  tokens, private keys, and equivalent secrets should move to OCI Vault where
  feasible.
- Wallet material that a driver must read as files should be treated as secret
  credential material and materialized with restrictive, preferably ephemeral,
  storage.
- Every cleanup candidate must receive one disposition: Remove, Consolidate,
  Move to Vault, Keep and document, Guard in CI, or Defer with owner and reason.
- Efficiency recommendations require evidence; stylistic preferences alone do
  not justify a broad rewrite.

### Oracle Driver Spike

- Build the spike image locally and copy it to the OCI VM for execution, matching the established smoke-test workflow.
- Start with a read-only Oracle interaction using the existing ADB wallet and database credentials.
- Only after a successful read-only result may an explicit write gate create a temporary item and image metadata; it must verify and remove them.
- The image, logs, and committed artifacts must not contain the wallet, database password, connection descriptor, catalog content, or Object Storage credentials.
- A validated result triggers a follow-up controller migration/image update; a failed or partial result records evidence and leaves the controller unchanged.

## Spikes

| # | Name | Type | Validates | Verdict | Tags |
|---|------|------|-----------|---------|------|
| 001 | oracledb-container-smoke | standard | Given the existing ADB wallet and a locally built image copied to the VM, when explicit read-only then gated temporary-write probes run, then `oracledb` can connect, bind, commit, query, and clean up against the live catalog without Oracle Instant Client. | VALIDATED | oracle, rust, container, smoke, adb |
| 002 | oracledb-oci-persistence-smoke | standard | Given the same VM, wallet, instance principal, bucket, and 16-byte payload as the existing persistence smoke, when `oracledb` drives the database lifecycle and the existing OCI adapter handles the original, then the item and object both round-trip and are deleted with per-phase timings. | VALIDATED | oracle, rust, oci, container, smoke, adb |
| 003 | ecosystem-inventory | standard | Given the repository and production VM, when a bounded redacted inventory runs, then configuration drift, persistent secrets, stale runtime artifacts, and cleanup candidates are visible without collecting secret values. | PARTIAL | inventory, configuration, secrets, podman, systemd, hygiene |
