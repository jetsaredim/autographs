# Terraform State Strategy

Phase 1 uses a bootstrap-plus-runtime state flow:

1. Local state only long enough to create or import the remote backend bucket.
2. OCI Object Storage as the steady-state backend for the tenancy bootstrap root.
3. OCI Object Storage as the steady-state backend for the runtime/app root.

## Backend Contract

Each committed Terraform root keeps a partial backend block:

```hcl
terraform {
  required_version = ">= 1.12.0, < 1.16.0"
  backend "oci" {}
}
```

The native OCI backend requires Terraform v1.12.0 or greater. Oracle recommends
this backend for Object Storage state files; the older S3-compatible Object
Storage path is deprecated for new use when Terraform can be upgraded.

Populate the non-sensitive backend coordinates from
`infra/terraform/bootstrap/backend.hcl.example`, then keep credentials out of
version control. Use separate state object keys for the two roots:

- tenancy bootstrap: `envs/prod/tenancy-bootstrap.tfstate`
- runtime/app: `envs/prod/terraform.tfstate`

Recommended local-only backend inputs:

- `bucket`
- `namespace`
- `region`
- `key`
- `workspace_key_prefix`
- `auth`

Prefer environment variables or interactive prompts for API-key credentials.
HashiCorp documents that backend settings can be written into local
`.terraform/` metadata, so keep `.terraform/`, plan files, and ad hoc backend
files out of Git.

## Migration Command

Once the bucket exists and any manual bucket creation has been imported, migrate
state with the root and key you intend to initialize:

```bash
terraform -chdir=infra/terraform init \
  -migrate-state \
  -backend-config=bootstrap/backend.hcl
```

Use `-reconfigure` later if you change backend coordinates and do not intend to
migrate state again.

## Regional Vault Ownership Migration

The Vault, software key, and three runtime secrets were originally created by
the tenancy root. Their OCI identities and secret versions must be preserved
when the regional deployment root assumes ownership. Terraform `moved` blocks
cannot transfer resources between separate backends, so this is a controlled
import-and-remove operation.

Do not run either root's normal apply concurrently with this procedure. Finish
the prerequisite tenancy IAM apply first, then run the guarded import script
from the repository root:

```bash
bash scripts/migrate-vault-state.sh
```

The script prompts silently for the existing Porkbun provider credentials
because that provider already has resources in the shared deployment state and
Terraform initializes every provider referenced by that state. The credentials
remain in the script process environment only. The script does not mutate DNS.

The script initializes both remote backends, writes permission-restricted state
snapshots under `/tmp`, imports or safely resumes all five resources, verifies
their OCIDs, and saves a deployment plan. It fails closed if that plan proposes
any Vault, key, or secret change. It deliberately leaves all source addresses
in tenancy state for review. If a non-default Terraform executable is needed,
set `TERRAFORM_BIN` for this command only.

Copy the migration artifact directory printed by the script before running the
remaining commands:

```bash
migration_dir=/tmp/autographs-vault-state-migration.replace_me
```

Only after all five imports and that plan succeed, remove the old addresses
from tenancy state. This changes Terraform ownership only; it does not delete
the OCI resources:

```bash
terraform -chdir=infra/terraform/tenancy state rm \
  'oci_vault_secret.runtime["admin_password_hash"]' \
  'oci_vault_secret.runtime["oracle_db_password"]' \
  'oci_vault_secret.runtime["oracle_db_wallet_password"]' \
  oci_kms_key.runtime_secrets \
  oci_kms_vault.runtime_secrets
```

Plan and apply the tenancy root once more. The plan may update the runtime
bundle policy from OCID conditions to stable secret-name conditions and remove
obsolete outputs, but it must not contain Vault, key, or secret destruction:

```bash
terraform -chdir=infra/terraform/tenancy plan \
  -var-file=environments/prod/terraform.tfvars \
  -out="${migration_dir}/tenancy-after-transfer.tfplan"

terraform -chdir=infra/terraform/tenancy show \
  "${migration_dir}/tenancy-after-transfer.tfplan"

terraform -chdir=infra/terraform/tenancy apply \
  "${migration_dir}/tenancy-after-transfer.tfplan"
```

Finish with another deployment-root plan. It must retain all five imported
OCIDs and show no Vault-resource drift. Preserve the snapshot directory until
the PR is merged and the merge-triggered deployment succeeds.

```bash
terraform -chdir=infra/terraform plan \
  -var-file=environments/prod/terraform.tfvars
```

## Bucket Versioning

The state bucket module enables Object Storage versioning by default. Keep that
enabled so you have a recovery path if state is overwritten or deleted
accidentally.

## Manual Bucket Path

If the backend bucket must exist before the first `terraform apply`, create it
manually with:

- private access only
- versioning enabled
- the final bucket name you intend Terraform to manage

Then import it with the instructions in
[imports.md](../infra/terraform/bootstrap/imports.md)
before running `terraform init -migrate-state`.

The historical single-state-to-split-state migration for this project has
already been completed. Current operators should initialize the tenancy and
runtime roots independently with the state keys above.

## Operator Checklist

Before treating remote state as ready:

1. `terraform plan` is clean against local state.
2. The bucket is versioned and private.
3. `terraform init -migrate-state` completes successfully.
4. A follow-up `terraform plan` against the OCI backend is also clean.
