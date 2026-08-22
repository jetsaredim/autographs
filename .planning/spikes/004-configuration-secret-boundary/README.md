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
| Direct controller retrieval plus tmpfs wallet | Reuses existing Rust instance-principal code; scalar secrets stay off disk; one runtime owner | Requires extracting a generic OCI signer/client; startup depends on Vault; wallet still needs a file | Chosen |

**Chosen approach:** Direct controller retrieval for scalar secrets, with only
the driver-required wallet/config files materialized in a dedicated container
tmpfs. Keep a single durable env file containing non-secret configuration and
secret OCIDs. Do not add OCI CLI or a second long-lived service to the VM.

## How to Run

```bash
python3 .planning/spikes/004-configuration-secret-boundary/test_secret_delivery_probe.py
python3 .planning/spikes/004-configuration-secret-boundary/secret_delivery_probe.py
```

## What to Expect

The persistent-env model reports one persistent secret-bearing file and three
process-environment secrets. Startup materialization reports no persistent
secret file but four ephemeral files. Direct application retrieval reports only
the required ephemeral wallet file; scalar values remain application memory.

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

## Results

**Verdict: PARTIAL.** Four local tests validate exposure and redaction behavior,
and official OCI/Podman contracts support the design. A live read of a disposable
Vault secret using the production VM's exact dynamic group and a secret-ID-bound
policy is still required.

Recommended sequence:

1. Remove the runtime OCI API key and its unused env coordinates independently.
2. Consolidate `app.env` and `controller.env` into one non-secret runtime config.
3. Provision a default Vault, software-protected symmetric key, secret slots,
   and exact secret-bundle read policy. Do not put secret content in Terraform
   state.
4. Extract the existing instance-principal session/signing code into a generic
   OCI client and add a Vault secret provider.
5. Add a bounded tmpfs wallet directory to the Quadlet and fetch only the files
   required by the Rust driver.
6. Remove plaintext fallback variables and fail readiness closed.
7. Prove restart, rotation-by-restart, Vault denial, redaction, and Oracle/OCI
   smoke behavior before deleting old host files.

See `CONFIGURATION-BOUNDARY.md` for the proposed value-by-value boundary.
