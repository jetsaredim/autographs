---
spike: 004
name: configuration-secret-boundary
type: comparison
validates: "Given the runtime's existing instance principal and file-based Oracle wallet requirement, when persistent env, startup materialization, and direct application retrieval are compared, then a lower-persistence secret boundary can be selected without adding OCI user credentials to the VM."
verdict: PARTIAL
related: [003]
tags: [configuration, env, oci-vault, secrets, instance-principal, podman]
---

# Spike 004: Configuration and Secret Boundary

## What This Validates

Given the runtime's existing instance principal and file-based Oracle wallet
requirement, when persistent env, startup materialization, and direct
application retrieval are compared, then a lower-persistence secret boundary
can be selected without adding OCI user credentials to the VM.

The design and local exposure probe are validated. A real Vault secret-bundle
read remains required before production implementation.

## Research

OCI instance principals let an application call OCI services without a user
credential/config file, and OCI Audit records the instance identity making the
call. The VM already uses this mechanism successfully for private Object
Storage. [OCI instance principals](https://docs.oracle.com/en-us/iaas/Content/Identity/Tasks/callingservicesfrominstances.htm)

OCI Secret Retrieval exposes `GetSecretBundle` and
`GetSecretBundleByName`; bundles are base64 content and currently have a 25 KB
maximum. [Secret retrieval](https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/get-secrets-contents.htm),
[secret size](https://docs.oracle.com/en-us/iaas/Content/secret-management/Tasks/update-secret-new-version.htm)

The Always Free contract includes 150 Vault secrets; software-protected key
versions and the Secrets service are free. A default vault is appropriate here,
not an hourly private vault. [Always Free Vault limits](https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm),
[OCI pricing](https://www.oracle.com/cloud/price-list/)

Podman supports tmpfs mounts for sensitive temporary data and Quadlet supports
container secrets. Podman secrets alone do not solve source-of-truth or rotation;
something still has to populate the host secret store. [Podman tmpfs](https://docs.podman.io/en/latest/markdown/podman-run.1.html),
[Quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)

| Approach | Pros | Cons | Status |
|----------|------|------|--------|
| Persistent mode-0600 env files | Simple; survives Vault outage during restart | Secrets persist on disk and enter the container environment; rotation remains deploy-coupled | Baseline only |
| Host/service startup materialization | Keeps secrets out of durable env and app secret-fetch logic | Requires OCI CLI/helper and refresh orchestration on the host; all scalars become files | Viable fallback |
| Direct controller retrieval plus tmpfs wallet | Reuses existing Rust instance-principal code; avoids application-managed persistent secret files; one runtime owner | Process and tmpfs pages remain pageable; requires swap/core mitigations, a generic OCI client, and Vault at startup | Chosen, conditional on production gates |

**Chosen approach:** Direct controller retrieval for scalar secrets, with only
the driver-required wallet/config files materialized in a dedicated container
tmpfs. This removes application-managed durable secret files; it does not by
itself guarantee that bytes never reach storage. The current production VM has
a persistent `/.swapfile`, and ordinary process and tmpfs pages may be paged to
it. Core dumps are another persistence path. C4 therefore cannot cut over until
swap and core-dump gates below are proven on the VM. Keep a single durable env
file containing non-secret configuration and secret OCIDs. Do not add OCI CLI
or a second long-lived service to the steady-state runtime.

## How to Run

```bash
python3 .planning/spikes/004-configuration-secret-boundary/test_secret_delivery_probe.py
python3 .planning/spikes/004-configuration-secret-boundary/secret_delivery_probe.py
```

## What to Expect

The persistent-env model reports one application-managed persistent
secret-bearing file and three process-environment secrets. Startup
materialization reports no application-managed persistent secret file but four
runtime files. Direct application retrieval reports only the required runtime
wallet file; scalar values remain application memory. The report explicitly
lists swap, process/tmpfs paging, and core dumps as excluded surfaces. It does
not prove those kernel-managed persistence paths are absent.

## Investigation Trail

1. The current controller quadlet loads both `app.env` and `controller.env`; the
   split does not express a durable ownership rule.
2. `controller.env` selects instance-principal authentication, while `app.env`
   still carries OCI API-key coordinates and the quadlet mounts
   `/opt/autographs/secrets/oci_api_key.pem`.
3. GitHub still needs the deploy-user API key for Terraform. Copying that same
   key onto the runtime VM is a separate and now-unnecessary exposure.
4. Documentation says plaintext `AUTOGRAPHS_ADMIN_PASSWORD` must not deploy, but
   Ansible retains backward-compatible resolution and emits the key into
   `app.env`. This contract should fail closed instead of accepting the local
   shortcut in production.
5. The existing instance-principal federation and signing implementation lives
   inside `OciInstancePrincipalMediaStore`. Direct Vault retrieval first needs a
   generic, tested OCI instance-principal session/signer abstraction.
6. The thin Oracle driver requires `ewallet.pem` plus its password, and an alias
   connect string also needs `tnsnames.ora`. Secret management reduces durable
   exposure but cannot make this file interface disappear.
7. Production deliberately configures persistent swap. Both controller memory
   and wallet tmpfs pages can be written there, so tmpfs is a lifecycle and file
   ownership improvement rather than a complete off-disk guarantee.

## Results

**Verdict: PARTIAL.** Five local tests validate the bounded
application-managed exposure and redaction behavior, and official OCI/Podman
contracts support the design. A live read of a disposable Vault secret using
the production VM's exact dynamic group and a secret-ID-bound policy is still
required. The VM swap/core gates are also unproven.

Recommended sequence:

1. Remove the runtime OCI API key and its unused env coordinates independently.
2. Consolidate `app.env` and `controller.env` into one non-secret runtime config.
3. Provision a default Vault, software-protected symmetric key, secret slots,
   and exact secret-bundle read policy. Do not put secret content in Terraform
   state.
4. Extract the existing instance-principal session/signing code into a generic
   OCI client and add a Vault secret provider.
5. Disable controller core dumps with the systemd service limit and verify the
   running process reports a zero core-file limit. Confirm the host's kernel
   crash-dump policy cannot retain controller memory.
6. Replace persistent plaintext swap with a reviewed non-persistent strategy:
   either no swap, or encrypted swap whose random boot-only key is not stored
   durably. Reboot and verify the chosen contract before secret cutover.
7. Add a bounded tmpfs wallet directory to the Quadlet and fetch only the files
   required by the Rust driver. Treat tmpfs as pageable unless the VM proof
   demonstrates otherwise.
8. Remove automatic plaintext fallback variables and fail readiness closed.
   Retain only the explicit, bounded rollback helper described in C4.
9. Prove restart, rotation-by-restart, Vault denial, redaction, core limits,
   swap behavior, and Oracle/OCI smoke behavior before deleting old host files.

See `CONFIGURATION-BOUNDARY.md` for the proposed value-by-value boundary.
