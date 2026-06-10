# Phase 5 Resume Note: Native OCI Object Storage With Instance Principals

Date: 2026-06-09

## Current Situation

PR 98 was moving the Rust admin/controller toward Vault-backed S3-compatible Object Storage credentials. A review finding correctly identified that the current Vault lookup path authenticated with the deploy API key inherited from `app.env`, which bypasses the runtime dynamic-group policy intended for the VM instance principal.

After discussion, the preferred direction is to pivot away from storing admin S3-compatible access/secret keys in Vault for the controller. Instead, the Rust admin/controller should use the OCI VM instance principal directly for Object Storage access.

This gets closer to the desired end state:

- The admin runtime does not need long-lived S3-compatible access/secret keys.
- The controller does not need Vault just to bootstrap media credentials.
- Runtime authorization is controlled by OCI dynamic-group policy for the VM.
- Object Storage permissions can be scoped to the media bucket.
- Vault remains available later for true app secrets rather than credential indirection.

## Target Crate Layout

Create a small Rust workspace around the controller:

```text
crates/
  oci-instance-principal/
    # IMDS discovery, Auth federation token, OCI request signing

  oci-object-storage/
    # Native OCI Object Storage calls using oci-instance-principal

controller/
  # Admin/controller app uses oci-object-storage for private media
```

Dependency direction:

```text
controller
  -> oci-object-storage
      -> oci-instance-principal
```

Keep these crates internal for now. Do not publish them externally until the implementation is proven against the production VM.

## Instance Principal Auth Crate

The `oci-instance-principal` crate should implement the OCI instance principal flow:

1. Read IMDS from `http://169.254.169.254/opc/v2` using `Authorization: Bearer Oracle`.
2. Fetch:
   - region
   - leaf certificate
   - leaf private key
   - intermediate certificate
3. Parse the leaf certificate:
   - extract tenancy OCID from `opc-tenant:` or `opc-identity:` subject value
   - compute SHA-1 certificate fingerprint
4. Generate a temporary RSA session keypair.
5. Call OCI Auth federation:
   - `POST https://auth.<region>.oraclecloud.com/v1/x509`
   - include sanitized cert chain and session public key
   - sign the federation request with the IMDS leaf private key
6. Receive a short-lived security token.
7. Sign OCI service requests with:
   - key id `ST$<token>`
   - the generated session private key

Use a real crypto/X.509 dependency rather than manual parsing. `openssl` is acceptable because the production controller already depends on native Oracle client libraries; avoiding brittle hand-rolled certificate and RSA code is more important than keeping this purely Rust right now.

The crate should expose a small surface, roughly:

```rust
let signer = OciInstancePrincipalSigner::from_imds()?;
let signed_headers = signer.sign("PUT", &url, &body)?;
```

or:

```rust
let signer = OciInstancePrincipalSigner::from_imds()?;
let response = signer.request(client, method, url, body)?;
```

Include token caching/refresh after the first working version. A first pass can fetch once at startup/request time if that keeps implementation smaller, but the code should be structured so refresh is obvious.

## Object Storage Crate

The `oci-object-storage` crate should use native OCI Object Storage APIs, not the S3 compatibility endpoint.

Initial minimum surface:

- `put_object(namespace, bucket, object_name, content_type, bytes)`
- `get_object(namespace, bucket, object_name)`
- `delete_object(namespace, bucket, object_name)`
- optionally `head_object(...)` if needed for smoke tests or cleanup

The controller media adapter should become a thin wrapper over this crate. It should preserve the existing privacy boundary:

- no direct public Object Storage URLs
- no bucket/namespace/object key leakage in public DTOs
- UUID-based object keys remain acceptable

## Infra And IAM Changes

If this pivot is adopted, the admin runtime IAM/Vault S3 credential path should be simplified.

Likely changes:

- Tenancy/runtime IAM should grant the runtime dynamic group bucket-scoped Object Storage permissions directly.
- Existing dynamic-group secret-bundle policy may no longer be needed for media credentials.
- The dedicated `autographs-admin-runtime` IAM user and customer secret key pair may no longer be needed for the controller.
- Vault secrets named like `autographs-admin-access-key` and `autographs-admin-secret-key` should be treated as transitional/deletable once the native object storage path is working.
- Ansible validation should stop requiring Vault coordinates for controller media persistence.
- Controller env should require native Object Storage coordinates, likely:
  - `OCI_REGION`
  - `OCI_MEDIA_NAMESPACE`
  - `OCI_MEDIA_BUCKET_NAME`
  - possibly no `OCI_S3_ENDPOINT`

Do not delete existing Terraform resources in the same first implementation PR unless the replacement is proven and the plan is reviewed. Prefer a clean migration:

1. Add native instance-principal object storage support.
2. Deploy and smoke test it.
3. Then remove now-unused Vault/S3-key scaffolding in a follow-up cleanup PR.

## PR 98 Implication

PR 98 currently contains work that tried to fetch admin S3-compatible keys from Vault. If we continue with this new direction, PR 98 should pivot:

- remove or abandon the Vault-backed S3 credential lookup path for controller media
- add the internal crates above
- update controller media wiring to use native OCI Object Storage with instance principal auth
- update Ansible/docs/workflow validation accordingly

Also fix the unrelated workflow lint finding:

```text
.github/workflows/deploy.yml: shellcheck SC2129
```

The Terraform apply step should group the multiple `echo ... >> "$GITHUB_OUTPUT"` writes:

```bash
{
  echo "runtime_public_ip=$(terraform output -raw runtime_public_ip)"
  echo "admin_vault_id=$(terraform output -raw admin_vault_id)"
  echo "admin_access_key_secret_name=$(terraform output -raw admin_access_key_secret_name)"
  echo "admin_secret_key_secret_name=$(terraform output -raw admin_secret_key_secret_name)"
} >> "$GITHUB_OUTPUT"
```

If the Vault outputs are removed by the pivot, adjust the grouped output block accordingly.

## Verification Plan

Local/CI:

- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- `git diff --check`

Live proof on VM:

- deploy controller with instance-principal object storage enabled
- run the live persistence/static publish smoke from the VM
- verify upload writes object to OCI Object Storage
- verify publish still generates static artifacts without leaking private object identifiers
- verify delete/unpublish behavior still removes generated public artifacts as expected

## Open Questions

- Should the initial workspace be rooted at repository root, or should `controller/` remain the workspace root with `controller/crates/...`? Root workspace is cleaner long term, but may touch more CI/build paths.
- Should the object storage crate support only the exact API calls needed by Autographs, or slightly more generic request helpers? Prefer exact calls first.
- Should Vault secret-bundle policy and admin runtime IAM user be removed immediately after native media works, or left for one follow-up cleanup PR? Prefer follow-up cleanup.
- Should the live smoke verify IAM denial for the old S3-key path? Probably not necessary in Phase 5; record as possible hardening.

## Resume Command Context

Resume from branch:

```bash
git status --short --branch
gh pr view 98 --json title,headRefName,baseRefName,url
```

Expected branch at time of note:

```text
fix/deploy-runtime-after-skipped-image-build
```

Do not merge PR 98 automatically. The user will merge after review.
