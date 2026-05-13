# Terraform State Strategy

Phase 1 uses a two-step state flow:

1. Local state only long enough to create or import the remote backend bucket.
2. OCI Object Storage as the steady-state backend for all later runs.

## Backend Contract

The committed Terraform root keeps a partial backend block:

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
version control.

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
state with:

```bash
.tools/terraform/terraform -chdir=infra/terraform init \
  -migrate-state \
  -backend-config=bootstrap/backend.hcl
```

Use `-reconfigure` later if you change backend coordinates and do not intend to
migrate state again.

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

## Operator Checklist

Before treating remote state as ready:

1. `terraform plan` is clean against local state.
2. The bucket is versioned and private.
3. `terraform init -migrate-state` completes successfully.
4. A follow-up `terraform plan` against the OCI backend is also clean.
