# Spike Manifest

## Idea

Validate Oracle's new pure-Rust `oracledb` driver against the existing Oracle
Autonomous Database through a hand-carried, one-shot container. The controller
continues to use `oracle` until this spike has a recorded live verdict.

## Requirements

- Build the spike image locally and copy it to the OCI VM for execution, matching the established smoke-test workflow.
- Start with a read-only Oracle interaction using the existing ADB wallet and database credentials.
- Only after a successful read-only result may an explicit write gate create a temporary item and image metadata; it must verify and remove them.
- The image, logs, and committed artifacts must not contain the wallet, database password, connection descriptor, catalog content, or Object Storage credentials.
- A validated result triggers a follow-up controller migration/image update; a failed or partial result records evidence and leaves the controller unchanged.

## Spikes

| # | Name | Type | Validates | Verdict | Tags |
|---|------|------|-----------|---------|------|
| 001 | oracledb-container-smoke | standard | Given the existing ADB wallet and a locally built image copied to the VM, when explicit read-only then gated temporary-write probes run, then `oracledb` can connect, bind, commit, query, and clean up against the live catalog without Oracle Instant Client. | VALIDATED | oracle, rust, container, smoke, adb |
